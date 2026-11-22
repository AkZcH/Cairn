//! lib.rs only declares modules. This pins the public surface so changing a
//! `pub mod` to `mod` breaks the build instead of silently breaking callers.

use cairn_ingest::{chunker, db, embedder, parser};
use std::path::Path;

#[test]
fn all_modules_are_public_and_compose() {
    let chunks = chunker::chunk_sections(&parser::parse_markdown("# T\n\nbody\n"));
    assert_eq!(chunks.len(), 1);

    let _client = db::ApiClient::new("http://localhost".into(), "k".into());
    let _load: fn(&Path) -> anyhow::Result<embedder::Embedder> = embedder::Embedder::load;
}
