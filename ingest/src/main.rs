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
    matches!(path.extension().and_then(|e| e.to_str()), Some("md") | Some("markdown") | Some("txt"))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()).map(|n| n.starts_with('.')).unwrap_or(false)
}

enum Outcome {
    Inserted { chunks: usize },
    Updated { chunks: usize },
    Skipped,
    Failed(String),
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

        let is_markdown = path.extension().map(|e| e == "md" || e == "markdown").unwrap_or(false);
        let sections = if is_markdown { parser::parse_markdown(&raw) } else { parser::parse_plaintext(&raw) };
        let chunks = chunker::chunk_sections(&sections);

        let resolved_title = title.map(String::from)
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
            .upload_document(source, resolved_title.as_deref(), &canonical_path, &hash, payload_chunks)
            .await?;

        Ok(match response.status.as_str() {
            "inserted" => Outcome::Inserted { chunks: chunks.len() },
            "updated" => Outcome::Updated { chunks: chunks.len() },
            _ => Outcome::Skipped,
        })
    }.await;

    result.unwrap_or_else(|e| Outcome::Failed(e.to_string()))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    let base_url = std::env::var("CAIRN_API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let api_key = std::env::var("CAIRN_API_KEY")
        .expect("CAIRN_API_KEY must be set (see ingest/.env.example, get one from /auth/signup)");

    println!("Loading embedding model...");
    let embedder = Embedder::load(&args.model_dir)?;
    let api = ApiClient::new(base_url, api_key);

    let files: Vec<PathBuf> = if args.path.is_dir() {
        WalkDir::new(&args.path)
            .into_iter()
            .filter_entry(|e| e.path() == args.path || !is_hidden(e.path()))
            .filter_map(|e| e.ok())
            .map(|e| e.into_path())
            .filter(|p| p.is_file() && is_supported(p))
            .collect()
    } else {
        vec![args.path.clone()]
    };

    if files.is_empty() {
        println!("No supported files found at {}", args.path.display());
        return Ok(());
    }

    println!("Found {} file(s) to process.", files.len());
    let (mut inserted, mut updated, mut skipped, mut failed) = (0, 0, 0, 0);

    for (i, path) in files.iter().enumerate() {
        print!("[{}/{}] {}... ", i + 1, files.len(), path.display());
        let title = if files.len() == 1 { args.title.as_deref() } else { None };

        match process_file(path, title, &embedder, &api).await {
            Outcome::Inserted { chunks } => { println!("inserted ({chunks} chunks)"); inserted += 1; }
            Outcome::Updated { chunks } => { println!("updated ({chunks} chunks re-embedded)"); updated += 1; }
            Outcome::Skipped => { println!("skipped (unchanged)"); skipped += 1; }
            Outcome::Failed(err) => { println!("FAILED: {err}"); failed += 1; }
        }
    }

    println!("\nDone. {inserted} inserted, {updated} updated, {skipped} skipped, {failed} failed, out of {} file(s).", files.len());
    Ok(())
}