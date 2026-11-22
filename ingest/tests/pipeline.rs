use cairn_ingest::{chunker, parser};
use std::collections::BTreeSet;

const DOC: &str = "\
# Tech Stack

**Core**:
- **Python** (primary) with FastAPI
- **Embeddings**: local models
- **Vector Store**: PostgreSQL plus pgvector
- **Knowledge Graph**: Neo4j in Docker
- **LLM**: Groq hosted models

## Deployment

Docker Compose runs the database and the API.

## Notes

Nothing else here.
";

fn tokens(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn vector_store_fact_sits_on_its_own_line_after_parsing_and_chunking() {
    let chunks = chunker::chunk_sections(&parser::parse_markdown(DOC));
    let chunk = chunks
        .iter()
        .find(|c| c.content.contains("Vector Store"))
        .expect("no chunk mentions the vector store");
    let line = chunk
        .content
        .lines()
        .find(|l| l.contains("Vector Store"))
        .unwrap();
    assert!(line.contains("pgvector"));
    assert!(!line.contains("Neo4j"));
    assert!(!line.contains("Embeddings"));
}

#[test]
fn no_words_are_lost_between_markdown_and_chunks() {
    let chunks = chunker::chunk_sections(&parser::parse_markdown(DOC));
    let mut out = String::new();
    for c in &chunks {
        if let Some(h) = &c.heading {
            out.push_str(h);
            out.push(' ');
        }
        out.push_str(&c.content);
        out.push(' ');
    }
    let missing: Vec<_> = tokens(DOC).difference(&tokens(&out)).cloned().collect();
    assert!(missing.is_empty(), "lost words: {missing:?}");
}

#[test]
fn a_large_document_stays_within_chunk_limits() {
    let md: String = (0..300)
        .map(|i| {
            let items: String = (0..6)
                .map(|j| format!("- item {i} number {j} with some filler text\n"))
                .collect();
            format!("## Section {i}\n\n{items}\n")
        })
        .collect();
    let chunks = chunker::chunk_sections(&parser::parse_markdown(&md));
    assert!(chunks.len() >= 300);
    for c in &chunks {
        assert!(c.content.chars().count() <= 1000); // mirrors chunker::MAX_CHARS
        assert!(c.heading.is_some());
    }
}
