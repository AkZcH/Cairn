use cairn_ingest::embedder::Embedder;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let model_dir = Path::new("models/bge-small-en-v1.5");
    println!("Loading model from {}...", model_dir.display());
    let embedder = Embedder::load(model_dir)?;

    let text = "Rust ownership and borrowing are the core of memory safety.";
    let vec = embedder.embed(text)?;

    println!("Embedding length: {}", vec.len());
    println!("First 5 values: {:?}", &vec[..5]);

    Ok(())
}
