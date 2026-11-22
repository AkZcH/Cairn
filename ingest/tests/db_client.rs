use cairn_ingest::db::{ApiClient, UploadResponse};
use serde_json::{Value, json};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const DOC_ID: &str = "11111111-1111-1111-1111-111111111111";

fn reply(status: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "document_id": DOC_ID, "status": status }))
}

async fn server_with(response: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .respond_with(response)
        .mount(&server)
        .await;
    server
}

/// 0.5 and 0.25 are exactly representable, so JSON round-trips compare equal.
fn one_chunk() -> Vec<(String, Vec<f32>)> {
    vec![("hello".to_string(), vec![0.5, 0.25])]
}

async fn upload(client: &ApiClient) -> anyhow::Result<UploadResponse> {
    client
        .upload_document(
            "markdown",
            Some("t.md"),
            "/abs/t.md",
            "hash123",
            one_chunk(),
        )
        .await
}

fn chunks_of(req: &Request) -> Vec<Value> {
    serde_json::from_slice::<Value>(&req.body)
        .ok()
        .and_then(|v| v["chunks"].as_array().cloned())
        .unwrap_or_default()
}

// ---------- success paths ----------

#[tokio::test]
async fn parses_a_successful_response() {
    let server = server_with(reply("inserted")).await;
    let client = ApiClient::new(server.uri(), "k".into());
    let resp = upload(&client).await.unwrap();
    assert_eq!(resp.status, "inserted");
    assert_eq!(resp.document_id.to_string(), DOC_ID);
}

#[tokio::test]
async fn parses_every_status_the_backend_can_return() {
    for status in ["inserted", "updated", "unchanged"] {
        let server = server_with(reply(status)).await;
        let client = ApiClient::new(server.uri(), "k".into());
        assert_eq!(upload(&client).await.unwrap().status, status);
    }
}

#[tokio::test]
async fn sends_the_api_key_as_a_bearer_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .and(header("authorization", "Bearer secret-key"))
        .respond_with(reply("inserted"))
        .expect(1)
        .mount(&server)
        .await;
    let client = ApiClient::new(server.uri(), "secret-key".into());
    assert!(upload(&client).await.is_ok());
}

#[tokio::test]
async fn a_wrong_key_does_not_match_and_surfaces_as_an_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .and(header("authorization", "Bearer right-key"))
        .respond_with(reply("inserted"))
        .mount(&server)
        .await;
    let client = ApiClient::new(server.uri(), "wrong-key".into());
    let err = upload(&client).await.unwrap_err().to_string();
    assert!(err.contains("404"), "{err}"); // wiremock answers unmatched requests with 404
}

#[tokio::test]
async fn sends_json_with_the_expected_shape() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .and(header("content-type", "application/json"))
        .and(body_partial_json(json!({
            "source": "markdown",
            "title": "t.md",
            "source_path": "/abs/t.md",
            "content_hash": "hash123",
            "chunks": [{ "index": 0, "content": "hello", "embedding": [0.5, 0.25] }]
        })))
        .respond_with(reply("inserted"))
        .expect(1)
        .mount(&server)
        .await;
    let client = ApiClient::new(server.uri(), "k".into());
    assert!(upload(&client).await.is_ok());
}

#[tokio::test]
async fn a_missing_title_is_sent_as_json_null() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .and(body_partial_json(json!({ "title": null })))
        .respond_with(reply("inserted"))
        .expect(1)
        .mount(&server)
        .await;
    let client = ApiClient::new(server.uri(), "k".into());
    client
        .upload_document("plaintext", None, "/abs/t.txt", "h", one_chunk())
        .await
        .unwrap();
}

#[tokio::test]
async fn chunk_indexes_are_sequential_from_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .and(|req: &Request| {
            let c = chunks_of(req);
            c.len() == 3 && c.iter().enumerate().all(|(i, ch)| ch["index"] == i as i64)
        })
        .respond_with(reply("inserted"))
        .expect(1)
        .mount(&server)
        .await;
    let chunks = (0..3)
        .map(|i| (format!("chunk {i}"), vec![0.0f32; 4]))
        .collect();
    let client = ApiClient::new(server.uri(), "k".into());
    client
        .upload_document("markdown", Some("t"), "/p", "h", chunks)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_large_payload_round_trips() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .and(|req: &Request| {
            let c = chunks_of(req);
            c.len() == 200
                && c.iter()
                    .all(|ch| ch["embedding"].as_array().is_some_and(|e| e.len() == 384))
        })
        .respond_with(reply("inserted"))
        .expect(1)
        .mount(&server)
        .await;
    let chunks = (0..200)
        .map(|i| (format!("chunk {i}"), vec![0.0f32; 384]))
        .collect();
    let client = ApiClient::new(server.uri(), "k".into());
    client
        .upload_document("markdown", Some("big"), "/p", "h", chunks)
        .await
        .unwrap();
}

#[tokio::test]
async fn the_client_can_be_reused_for_several_uploads() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/documents/upload"))
        .respond_with(reply("inserted"))
        .expect(2)
        .mount(&server)
        .await;
    let client = ApiClient::new(server.uri(), "k".into());
    upload(&client).await.unwrap();
    upload(&client).await.unwrap();
}

// ---------- failure paths ----------

#[tokio::test]
async fn http_500_includes_status_and_body_in_the_error() {
    let server = server_with(ResponseTemplate::new(500).set_body_string("boom")).await;
    let client = ApiClient::new(server.uri(), "k".into());
    let err = upload(&client).await.unwrap_err().to_string();
    assert!(err.contains("500") && err.contains("boom"), "{err}");
}

#[tokio::test]
async fn http_401_is_an_error() {
    let server = server_with(ResponseTemplate::new(401).set_body_string("Invalid API key")).await;
    let client = ApiClient::new(server.uri(), "k".into());
    let err = upload(&client).await.unwrap_err().to_string();
    assert!(
        err.contains("401") && err.contains("Invalid API key"),
        "{err}"
    );
}

#[tokio::test]
async fn http_422_validation_detail_is_surfaced() {
    let body = json!({ "detail": "field required" }).to_string();
    let server = server_with(ResponseTemplate::new(422).set_body_string(body)).await;
    let client = ApiClient::new(server.uri(), "k".into());
    let err = upload(&client).await.unwrap_err().to_string();
    assert!(
        err.contains("422") && err.contains("field required"),
        "{err}"
    );
}

#[tokio::test]
async fn a_200_with_a_non_json_body_is_an_error() {
    let server = server_with(ResponseTemplate::new(200).set_body_string("not json")).await;
    let client = ApiClient::new(server.uri(), "k".into());
    assert!(upload(&client).await.is_err());
}

#[tokio::test]
async fn a_200_missing_required_fields_is_an_error() {
    let server = server_with(ResponseTemplate::new(200).set_body_json(json!({}))).await;
    let client = ApiClient::new(server.uri(), "k".into());
    assert!(upload(&client).await.is_err());
}

#[tokio::test]
async fn a_200_with_an_invalid_uuid_is_an_error() {
    let body = json!({ "document_id": "not-a-uuid", "status": "inserted" });
    let server = server_with(ResponseTemplate::new(200).set_body_json(body)).await;
    let client = ApiClient::new(server.uri(), "k".into());
    assert!(upload(&client).await.is_err());
}

#[tokio::test]
async fn connection_refused_is_an_error() {
    let client = ApiClient::new("http://127.0.0.1:1".into(), "k".into());
    assert!(upload(&client).await.is_err());
}

#[tokio::test]
async fn gap_base_url_trailing_slash_is_tolerated() {
    let server = server_with(reply("inserted")).await;
    let client = ApiClient::new(format!("{}/", server.uri()), "k".into());
    assert!(upload(&client).await.is_ok());
}
