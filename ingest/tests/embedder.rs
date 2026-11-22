use cairn_ingest::embedder::Embedder;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const FILES: [&str; 3] = ["config.json", "tokenizer.json", "model.safetensors"];

fn load_error(dir: &Path) -> String {
    match Embedder::load(dir) {
        Ok(_) => panic!("expected load to fail for {dir:?}"),
        Err(e) => format!("{e:#}"),
    }
}

// ---------- no model needed: load() error handling ----------

#[test]
fn a_missing_directory_gives_an_actionable_error() {
    let msg = load_error(Path::new("definitely/not/here"));
    assert!(msg.contains("missing model file"), "{msg}");
    assert!(msg.contains("config.json"), "{msg}");
    assert!(
        msg.contains("huggingface.co/BAAI/bge-small-en-v1.5"),
        "{msg}"
    );
}

#[test]
fn an_empty_directory_is_reported_as_missing_files() {
    let tmp = TempDir::new().unwrap();
    assert!(load_error(tmp.path()).contains("missing model file"));
}

#[test]
fn each_missing_file_is_named_in_the_error() {
    for missing in FILES {
        let tmp = TempDir::new().unwrap();
        for f in FILES.iter().filter(|f| **f != missing) {
            fs::write(tmp.path().join(f), "x").unwrap();
        }
        let msg = load_error(tmp.path());
        assert!(
            msg.contains("missing model file") && msg.contains(missing),
            "{missing}: {msg}"
        );
    }
}

#[test]
fn present_but_corrupt_files_fail_with_a_parse_error_not_a_missing_file_error() {
    let tmp = TempDir::new().unwrap();
    for f in FILES {
        fs::write(tmp.path().join(f), "this is not valid").unwrap();
    }
    let msg = load_error(tmp.path());
    assert!(!msg.contains("missing model file"), "{msg}");
}

// ---------- needs the real model ----------

fn embedder() -> Embedder {
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

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum() // vectors are unit length
}

#[test]
#[ignore = "needs embedding model files"]
fn model_embeddings_have_384_finite_dimensions() {
    let v = embedder().embed("Rust ownership and borrowing").unwrap();
    assert_eq!(v.len(), 384);
    assert!(v.iter().all(|x| x.is_finite()));
}

#[test]
#[ignore = "needs embedding model files"]
fn model_embeddings_are_unit_length() {
    let v = embedder()
        .embed("normalisation matters for cosine similarity")
        .unwrap();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-3, "norm was {norm}");
}

#[test]
#[ignore = "needs embedding model files"]
fn model_embedding_is_deterministic() {
    let e = embedder();
    let a = e.embed("same input, same output").unwrap();
    let b = e.embed("same input, same output").unwrap();
    assert!(a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-5));
}

#[test]
#[ignore = "needs embedding model files"]
fn model_different_texts_give_different_vectors() {
    let e = embedder();
    assert_ne!(e.embed("alpha").unwrap(), e.embed("omega").unwrap());
}

#[test]
#[ignore = "needs embedding model files"]
fn model_related_texts_are_closer_than_unrelated_ones() {
    let e = embedder();
    let pg = e.embed("PostgreSQL is a relational database").unwrap();
    let close = e.embed("Postgres relational database system").unwrap();
    let far = e.embed("Recipe for banana bread with walnuts").unwrap();
    assert!(cosine(&pg, &close) > cosine(&pg, &far));
}

#[test]
#[ignore = "needs embedding model files"]
fn model_odd_inputs_do_not_panic() {
    let e = embedder();
    for text in [
        "",
        "   ",
        "\n\n",
        "日本語のテキスト",
        "emoji ☕🚀",
        "<<<>>>",
    ] {
        if let Ok(v) = e.embed(text) {
            assert_eq!(v.len(), 384, "{text:?}");
        }
    }
}

#[test]
#[ignore = "needs embedding model files"]
fn model_very_long_input_reports_instead_of_panicking() {
    // The chunker never splits inside a paragraph, so one huge paragraph reaches
    // embed() whole. BERT-style models cap at 512 tokens. Run with --nocapture
    // to see whether this errors or is truncated.
    let long = "word ".repeat(5000);
    let r = embedder().embed(&long);
    eprintln!(
        "long input result: {:?}",
        r.as_ref().map(|v| v.len()).map_err(|e| e.to_string())
    );
    if let Ok(v) = r {
        assert_eq!(v.len(), 384);
    }
}
