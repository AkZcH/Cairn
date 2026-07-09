use cairn_ingest::{chunker, db, embedder::Embedder, parser};
use clap::Parser as ClapParser;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(ClapParser)]
#[command(about = "Parse, chunk, embed, and ingest a file or a whole folder into Cairn")]
struct Args {
    /// Path to a file OR a directory to ingest
    path: PathBuf,

    /// Optional title (only applies when ingesting a single file — in
    /// batch mode, titles are always derived from filenames, otherwise
    /// every file in the folder would get the same title)
    #[arg(long)]
    title: Option<String>,

    #[arg(long, default_value = "models/bge-small-en-v1.5")]
    model_dir: PathBuf,
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

/// Skips dotfiles/dot-directories (.git, .obsidian, etc) — common vault
/// noise that isn't real note content.
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

enum Outcome {
    Inserted { chunks: usize },
    Skipped,
    Failed(String),
}

async fn process_file(
    path: &Path,
    title: Option<&str>,
    embedder: &Embedder,
    pool: &sqlx::PgPool,
) -> Outcome {
    let result: anyhow::Result<Outcome> = async {
        let raw = fs::read_to_string(path)?;
        let hash = content_hash(&raw);

        let is_markdown = path
            .extension()
            .map(|ext| ext == "md" || ext == "markdown")
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

        let (document_id, was_new) =
            db::upsert_document(pool, source, resolved_title.as_deref(), &hash).await?;

        if !was_new {
            return Ok(Outcome::Skipped);
        }

        for (i, chunk) in chunks.iter().enumerate() {
            let content = match &chunk.heading {
                Some(h) => format!("{h}\n\n{}", chunk.content),
                None => chunk.content.clone(),
            };
            let embedding = embedder.embed(&content)?;
            db::insert_chunk(pool, document_id, i as i32, &content, &embedding).await?;
        }

        Ok(Outcome::Inserted { chunks: chunks.len() })
    }
    .await;

    match result {
        Ok(outcome) => outcome,
        Err(e) => Outcome::Failed(e.to_string()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (see ingest/.env.example)");

    println!("Loading embedding model...");
    let embedder = Embedder::load(&args.model_dir)?;
    let pool = db::create_pool(&database_url).await?;

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
        println!(
            "No supported files found (.md, .markdown, .txt) at {}",
            args.path.display()
        );
        return Ok(());
    }

    println!("Found {} file(s) to process.", files.len());

    let (mut inserted, mut skipped, mut failed) = (0, 0, 0);

    for (i, path) in files.iter().enumerate() {
        print!("[{}/{}] {}... ", i + 1, files.len(), path.display());

        let title = if files.len() == 1 { args.title.as_deref() } else { None };

        match process_file(path, title, &embedder, &pool).await {
            Outcome::Inserted { chunks } => {
                println!("inserted ({chunks} chunks)");
                inserted += 1;
            }
            Outcome::Skipped => {
                println!("skipped (already ingested)");
                skipped += 1;
            }
            Outcome::Failed(err) => {
                println!("FAILED: {err}");
                failed += 1;
            }
        }
    }

    println!(
        "\nDone. {inserted} inserted, {skipped} skipped, {failed} failed, out of {} file(s).",
        files.len()
    );

    Ok(())
}