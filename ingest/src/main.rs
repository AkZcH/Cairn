use cairn_ingest::{chunker, db::ApiClient, embedder::Embedder, parser};
use clap::Parser as ClapParser;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(ClapParser)]
#[command(about = "Parse, chunk, embed, and upload a file or folder into Cairn")]
struct Args {
    path: PathBuf,

    #[arg(long)]
    title: Option<String>,

    #[arg(long, default_value_os_t = default_model_dir())]
    model_dir: PathBuf,
}

fn default_model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cairn")
        .join("models")
        .join("bge-small-en-v1.5")
}

fn content_hash(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn is_supported(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md") | Some("markdown") | Some("txt")
    )
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

enum Outcome {
    Inserted { chunks: usize },
    Updated { chunks: usize },
    Skipped,
    Failed(String),
}

fn discover_files(root: &Path) -> Vec<PathBuf> {
    if root.is_dir() {
        WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| e.path() == root || !is_hidden(e.path()))
            .filter_map(|e| e.ok())
            .map(|e| e.into_path())
            .filter(|p| p.is_file() && is_supported(p))
            .collect()
    } else {
        vec![root.to_path_buf()]
    }
}

async fn process_file(
    path: &Path,
    title: Option<&str>,
    embedder: &Embedder,
    api: &ApiClient,
) -> Outcome {
    let result: anyhow::Result<Outcome> = async {
        let raw = fs::read_to_string(path)?;
        let hash = content_hash(&raw);

        let is_markdown = path
            .extension()
            .map(|e| e == "md" || e == "markdown")
            .unwrap_or(false);
        let sections = if is_markdown {
            parser::parse_markdown(&raw)
        } else {
            parser::parse_plaintext(&raw)
        };
        let chunks = chunker::chunk_sections(&sections);

        let resolved_title = title
            .map(String::from)
            .or_else(|| path.file_name().map(|n| n.to_string_lossy().to_string()));
        let source = if is_markdown { "markdown" } else { "plaintext" };
        let canonical_path = std::fs::canonicalize(path)?.to_string_lossy().to_string();

        let mut payload_chunks = Vec::new();
        for chunk in &chunks {
            let content = match &chunk.heading {
                Some(h) => format!("{h}\n\n{}", chunk.content),
                None => chunk.content.clone(),
            };
            let embedding = embedder.embed(&content)?;
            payload_chunks.push((content, embedding));
        }

        let response = api
            .upload_document(
                source,
                resolved_title.as_deref(),
                &canonical_path,
                &hash,
                payload_chunks,
            )
            .await?;

        Ok(match response.status.as_str() {
            "inserted" => Outcome::Inserted {
                chunks: chunks.len(),
            },
            "updated" => Outcome::Updated {
                chunks: chunks.len(),
            },
            _ => Outcome::Skipped,
        })
    }
    .await;

    result.unwrap_or_else(|e| Outcome::Failed(e.to_string()))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    let base_url =
        std::env::var("CAIRN_API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let api_key = std::env::var("CAIRN_API_KEY")
        .expect("CAIRN_API_KEY must be set (see ingest/.env.example, get one from /auth/signup)");

    println!("Loading embedding model...");
    let embedder = Embedder::load(&args.model_dir)?;
    let api = ApiClient::new(base_url, api_key);

    let files = discover_files(&args.path);

    if files.is_empty() {
        println!("No supported files found at {}", args.path.display());
        return Ok(());
    }

    println!("Found {} file(s) to process.", files.len());
    let (mut inserted, mut updated, mut skipped, mut failed) = (0, 0, 0, 0);

    for (i, path) in files.iter().enumerate() {
        print!("[{}/{}] {}... ", i + 1, files.len(), path.display());
        let title = if files.len() == 1 {
            args.title.as_deref()
        } else {
            None
        };

        match process_file(path, title, &embedder, &api).await {
            Outcome::Inserted { chunks } => {
                println!("inserted ({chunks} chunks)");
                inserted += 1;
            }
            Outcome::Updated { chunks } => {
                println!("updated ({chunks} chunks re-embedded)");
                updated += 1;
            }
            Outcome::Skipped => {
                println!("skipped (unchanged)");
                skipped += 1;
            }
            Outcome::Failed(err) => {
                println!("FAILED: {err}");
                failed += 1;
            }
        }
    }

    println!(
        "\nDone. {inserted} inserted, {updated} updated, {skipped} skipped, {failed} failed, out of {} file(s).",
        files.len()
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wiremock::matchers::{header, method, path as url_path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    // ---------- content_hash ----------

    #[test]
    fn content_hash_matches_known_sha256_vectors() {
        assert_eq!(
            content_hash(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            content_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn content_hash_is_64_lowercase_hex() {
        let h = content_hash("anything at all");
        assert_eq!(h.len(), 64);
        assert!(
            h.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        );
    }

    #[test]
    fn content_hash_is_deterministic_and_sensitive_to_every_byte() {
        assert_eq!(content_hash("note"), content_hash("note"));
        assert_ne!(content_hash("note"), content_hash("note "));
        assert_ne!(content_hash("Note"), content_hash("note"));
        assert_ne!(content_hash("a\nb"), content_hash("a\r\nb"));
    }

    // ---------- is_supported / is_hidden ----------

    #[test]
    fn is_supported_accepts_the_three_note_extensions() {
        for name in [
            "a.md",
            "a.markdown",
            "a.txt",
            "dir/deep/a.md",
            "archive.tar.md",
        ] {
            assert!(is_supported(Path::new(name)), "{name}");
        }
    }

    #[test]
    fn is_supported_rejects_everything_else() {
        for name in [
            "a.json",
            "a.rs",
            "noext",
            "md",
            ".md",
            "notes.md.bak",
            "a.mdx",
        ] {
            assert!(!is_supported(Path::new(name)), "{name}");
        }
    }

    #[test]
    #[ignore = "known gap: extension matching is case-sensitive, so NOTES.MD is skipped"]
    fn gap_extension_matching_is_case_insensitive() {
        assert!(is_supported(Path::new("NOTES.MD")));
        assert!(is_supported(Path::new("Readme.Txt")));
    }

    #[test]
    fn is_hidden_flags_dotfiles_and_dot_directories() {
        for name in [
            ".git",
            ".obsidian",
            ".hidden.md",
            "vault/.trash",
            "a/b/.cache",
        ] {
            assert!(is_hidden(Path::new(name)), "{name}");
        }
    }

    #[test]
    fn is_hidden_ignores_normal_names_and_dot_components() {
        for name in ["note.md", "a/b/c.md", "dir.with.dots/file", "", ".", ".."] {
            assert!(!is_hidden(Path::new(name)), "{name:?}");
        }
    }

    // ---------- default_model_dir ----------

    #[test]
    fn default_model_dir_has_the_expected_tail() {
        let p = default_model_dir();
        assert!(
            p.ends_with(Path::new("cairn/models/bge-small-en-v1.5")),
            "{p:?}"
        );
        if dirs::data_dir().is_some() {
            assert!(p.is_absolute(), "{p:?}");
        }
    }

    // ---------- CLI arguments ----------

    #[test]
    fn args_defaults() {
        let a = Args::try_parse_from(["cairn-ingest", "notes"]).unwrap();
        assert_eq!(a.path, PathBuf::from("notes"));
        assert!(a.title.is_none());
        assert_eq!(a.model_dir, default_model_dir());
    }

    #[test]
    fn args_accept_title_and_model_dir() {
        let a = Args::try_parse_from([
            "cairn-ingest",
            "note.md",
            "--title",
            "My Note",
            "--model-dir",
            "/tmp/m",
        ])
        .unwrap();
        assert_eq!(a.title.as_deref(), Some("My Note"));
        assert_eq!(a.model_dir, PathBuf::from("/tmp/m"));
    }

    #[test]
    fn args_reject_bad_invocations() {
        assert!(Args::try_parse_from(["cairn-ingest"]).is_err()); // path is required
        assert!(Args::try_parse_from(["cairn-ingest", "a", "b"]).is_err()); // one path only
        assert!(Args::try_parse_from(["cairn-ingest", "a", "--nope"]).is_err());
        assert!(Args::try_parse_from(["cairn-ingest", "a", "--title"]).is_err()); // missing value
    }

    // ---------- discover_files ----------

    fn touch(root: &Path, rel: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "x").unwrap();
    }

    fn rel_sorted(root: &Path, files: Vec<PathBuf>) -> Vec<String> {
        let mut v: Vec<String> = files
            .iter()
            .map(|p| {
                p.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        v.sort();
        v
    }

    #[test]
    fn discover_finds_supported_files_recursively_and_skips_the_rest() {
        let tmp = TempDir::new().unwrap();
        for f in [
            "a.md",
            "b.txt",
            "c.markdown",
            "d.json",
            "e.rs",
            "noext",
            "sub/f.md",
            "sub/deep/g.txt",
            ".hidden/h.md",
            ".dot.md",
            "sub/.git/i.md",
        ] {
            touch(tmp.path(), f);
        }
        let got = rel_sorted(tmp.path(), discover_files(tmp.path()));
        assert_eq!(
            got,
            vec!["a.md", "b.txt", "c.markdown", "sub/deep/g.txt", "sub/f.md"]
        );
    }

    #[test]
    fn discover_does_not_treat_a_hidden_root_as_hidden() {
        let tmp = tempfile::Builder::new()
            .prefix(".vault-")
            .tempdir()
            .unwrap();
        touch(tmp.path(), "a.md");
        assert_eq!(discover_files(tmp.path()).len(), 1);
    }

    #[test]
    fn discover_on_an_empty_directory_returns_nothing() {
        let tmp = TempDir::new().unwrap();
        assert!(discover_files(tmp.path()).is_empty());
    }

    #[test]
    fn discover_ignores_directories_named_like_notes() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "folder.md/inner.txt");
        let got = rel_sorted(tmp.path(), discover_files(tmp.path()));
        assert_eq!(got, vec!["folder.md/inner.txt"]);
    }

    #[test]
    fn discover_finds_deeply_nested_files() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "a/b/c/d/e/f/g/h/i/j/deep.md");
        assert_eq!(discover_files(tmp.path()).len(), 1);
    }

    #[test]
    fn discover_on_a_single_file_returns_it_even_with_an_unsupported_extension() {
        // An explicitly named file is always attempted; filtering only applies to directory walks.
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "data.json");
        let file = tmp.path().join("data.json");
        assert_eq!(discover_files(&file), vec![file]);
    }

    #[test]
    fn discover_on_a_missing_path_returns_it_unchanged() {
        let p = PathBuf::from("definitely/not/here.md");
        assert_eq!(discover_files(&p), vec![p]);
    }

    // ---------- process_file (needs the model, backend is mocked) ----------

    fn load_embedder() -> Embedder {
        let dir = std::env::var_os("CAIRN_TEST_MODEL_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("models")
                    .join("bge-small-en-v1.5")
            });
        Embedder::load(&dir).expect(
            "model files not found: set CAIRN_TEST_MODEL_DIR or put them in ingest/models/bge-small-en-v1.5",
        )
    }

    fn replying(status: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "document_id": "00000000-0000-0000-0000-000000000001",
            "status": status
        }))
    }

    async fn backend(
        response: ResponseTemplate,
        matcher: impl wiremock::Match + 'static,
    ) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(url_path("/documents/upload"))
            .and(header("authorization", "Bearer test-key"))
            .and(matcher)
            .respond_with(response)
            .expect(1)
            .mount(&server)
            .await;
        server
    }

    fn body_of(req: &Request) -> Option<serde_json::Value> {
        serde_json::from_slice(&req.body).ok()
    }

    #[tokio::test]
    #[ignore = "needs embedding model files"]
    async fn model_process_file_sends_the_expected_payload_and_reports_inserted() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("note.md");
        let text = "# Title\n\nSome body text.\n";
        fs::write(&file, text).unwrap();
        let expected_hash = content_hash(text);

        let matcher = move |req: &Request| -> bool {
            let Some(v) = body_of(req) else { return false };
            v["source"] == "markdown"
                && v["title"] == "note.md"
                && v["content_hash"] == expected_hash.as_str()
                && v["source_path"]
                    .as_str()
                    .is_some_and(|p| p.ends_with("note.md"))
                && v["chunks"].as_array().is_some_and(|c| c.len() == 1)
                && v["chunks"][0]["index"] == 0
                && v["chunks"][0]["content"] == "Title\n\nSome body text."
                && v["chunks"][0]["embedding"]
                    .as_array()
                    .is_some_and(|e| e.len() == 384)
        };
        let server = backend(replying("inserted"), matcher).await;

        let api = ApiClient::new(server.uri(), "test-key".into());
        let outcome = process_file(&file, None, &load_embedder(), &api).await;
        assert!(matches!(outcome, Outcome::Inserted { chunks: 1 }));
    }

    #[tokio::test]
    #[ignore = "needs embedding model files"]
    async fn model_process_file_treats_txt_as_plaintext_and_honours_a_title_override() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("plain.txt");
        fs::write(&file, "First paragraph.\n\nSecond paragraph.\n").unwrap();

        let matcher = |req: &Request| -> bool {
            let Some(v) = body_of(req) else { return false };
            v["source"] == "plaintext"
                && v["title"] == "Custom"
                && v["chunks"][0]["content"] == "First paragraph."
                && v["chunks"][1]["content"] == "Second paragraph."
        };
        let server = backend(replying("updated"), matcher).await;

        let api = ApiClient::new(server.uri(), "test-key".into());
        let outcome = process_file(&file, Some("Custom"), &load_embedder(), &api).await;
        assert!(matches!(outcome, Outcome::Updated { chunks: 2 }));
    }

    #[tokio::test]
    #[ignore = "needs embedding model files"]
    async fn model_process_file_maps_unchanged_to_skipped() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("note.md");
        fs::write(&file, "# T\n\nbody\n").unwrap();
        let server = backend(replying("unchanged"), |_: &Request| true).await;

        let api = ApiClient::new(server.uri(), "test-key".into());
        let outcome = process_file(&file, None, &load_embedder(), &api).await;
        assert!(matches!(outcome, Outcome::Skipped));
    }

    #[tokio::test]
    #[ignore = "needs embedding model files"]
    async fn model_process_file_reports_backend_errors_as_failed() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("note.md");
        fs::write(&file, "# T\n\nbody\n").unwrap();
        let server = backend(
            ResponseTemplate::new(500).set_body_string("boom"),
            |_: &Request| true,
        )
        .await;

        let api = ApiClient::new(server.uri(), "test-key".into());
        match process_file(&file, None, &load_embedder(), &api).await {
            Outcome::Failed(msg) => assert!(msg.contains("500") && msg.contains("boom"), "{msg}"),
            _ => panic!("expected Failed"),
        }
    }

    #[tokio::test]
    #[ignore = "needs embedding model files"]
    async fn model_process_file_reports_a_missing_file_as_failed() {
        let api = ApiClient::new("http://127.0.0.1:1".into(), "k".into());
        let outcome =
            process_file(Path::new("no/such/file.md"), None, &load_embedder(), &api).await;
        assert!(matches!(outcome, Outcome::Failed(_)));
    }

    #[tokio::test]
    #[ignore = "needs embedding model files"]
    async fn model_process_file_uploads_an_empty_file_with_zero_chunks() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("empty.md");
        fs::write(&file, "").unwrap();
        let matcher = |req: &Request| -> bool {
            body_of(req).is_some_and(|v| v["chunks"].as_array().is_some_and(|c| c.is_empty()))
        };
        let server = backend(replying("inserted"), matcher).await;

        let api = ApiClient::new(server.uri(), "test-key".into());
        let outcome = process_file(&file, None, &load_embedder(), &api).await;
        assert!(matches!(outcome, Outcome::Inserted { chunks: 0 }));
    }
}
