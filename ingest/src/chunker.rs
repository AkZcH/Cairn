//! Turns parsed sections into chunks small enough to embed sensibly.
//!
//! Strategy: a section's content is already structurally coherent (it's
//! everything under one heading). If it fits within `max_chars`, it becomes
//! one chunk. If it's too big, we split on paragraph boundaries (blank
//! lines) and greedily pack paragraphs together until we'd exceed the
//! limit, starting a new chunk at that point. We never split *inside* a
//! paragraph — that would cut a sentence in half and hurt retrieval quality
//! more than an oversized chunk would.

use crate::parser::Section;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub heading: Option<String>,
    pub content: String,
}

const MAX_CHARS: usize = 1000;

pub fn chunk_sections(sections: &[Section]) -> Vec<Chunk> {
    let mut chunks = Vec::new();

    for section in sections {
        if section.content.chars().count() <= MAX_CHARS {
            chunks.push(Chunk {
                heading: section.heading.clone(),
                content: section.content.clone(),
            });
            continue;
        }

        let paragraphs: Vec<&str> = section
            .content
            .split('\n')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();

        let mut current = String::new();

        for para in paragraphs {
            let would_be_len = current.chars().count() + para.chars().count() + 1;

            if !current.is_empty() && would_be_len > MAX_CHARS {
                chunks.push(Chunk {
                    heading: section.heading.clone(),
                    content: current.trim().to_string(),
                });
                current = String::new();
            }

            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(para);
        }

        if !current.trim().is_empty() {
            chunks.push(Chunk {
                heading: section.heading.clone(),
                content: current.trim().to_string(),
            });
        }
    }

    chunks
}
