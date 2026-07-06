mod chunker;
mod db;
mod parser;

use clap::Parser as ClapParser;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(about = "Parse, chunk, and ingest a document into Cairn")]
struct Args {
    /// Path to the file to ingest
    file: PathBuf,

    /// Optional title (defaults to the filename)
    #[arg(long)]
    title: Option<String>,
}

fn content_hash(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args = Args::parse();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (see ingest/.env.example)");

    let raw = fs::read_to_string(&args.file)?;
    let hash = content_hash(&raw);

    let is_markdown = args
        .file
        .extension()
        .map(|ext| ext == "md" || ext == "markdown")
        .unwrap_or(false);

    let sections = if is_markdown {
        parser::parse_markdown(&raw)
    } else {
        parser::parse_plaintext(&raw)
    };

    let chunks = chunker::chunk_sections(&sections);
    println!("Parsed into {} section(s), {} chunk(s)", sections.len(), chunks.len());

    let pool = db::create_pool(&database_url).await?;

    let title = args
        .title
        .or_else(|| args.file.file_name().map(|n| n.to_string_lossy().to_string()));

    let source = if is_markdown { "markdown" } else { "plaintext" };

    let (document_id, was_new) = db::upsert_document(&pool, source, title.as_deref(), &hash).await?;

    if !was_new {
        println!("Document already ingested (content unchanged) — skipping chunk insert.");
        println!("document_id: {document_id}");
        return Ok(());
    }

    for (i, chunk) in chunks.iter().enumerate() {
        let content = match &chunk.heading {
            Some(h) => format!("{h}\n\n{}", chunk.content),
            None => chunk.content.clone(),
        };
        db::insert_chunk(&pool, document_id, i as i32, &content).await?;
    }

    println!("Inserted document {document_id} with {} chunk(s).", chunks.len());
    Ok(())
}