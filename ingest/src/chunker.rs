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

#[cfg(test)]
mod tests {
    use super::*;

    fn section(heading: Option<&str>, content: &str) -> Section {
        Section {
            heading: heading.map(String::from),
            content: content.to_string(),
        }
    }

    /// Fixed-length paragraph that starts and ends with a non-space char, so
    /// the chunker's trimming never changes it.
    fn para(i: usize, len: usize) -> String {
        let head = format!("P{i:02}");
        format!("{head}{}", "x".repeat(len - head.len()))
    }

    fn rejoin(chunks: &[Chunk]) -> String {
        chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn empty_input_yields_no_chunks() {
        assert!(chunk_sections(&[]).is_empty());
    }

    #[test]
    fn small_section_is_exactly_one_chunk() {
        let chunks = chunk_sections(&[section(Some("Intro"), "Short body.")]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading.as_deref(), Some("Intro"));
        assert_eq!(chunks[0].content, "Short body.");
    }

    #[test]
    fn headingless_sections_stay_headingless() {
        let chunks = chunk_sections(&[section(None, "text")]);
        assert_eq!(chunks[0].heading, None);
    }

    #[test]
    fn sections_are_never_merged_and_keep_their_order() {
        let chunks = chunk_sections(&[
            section(Some("A"), "one"),
            section(Some("B"), "two"),
            section(Some("C"), "three"),
        ]);
        let h: Vec<_> = chunks.iter().map(|c| c.heading.as_deref()).collect();
        assert_eq!(h, vec![Some("A"), Some("B"), Some("C")]);
    }

    #[test]
    fn heading_only_section_becomes_an_empty_content_chunk() {
        // Pins current behaviour: main.rs will embed "{heading}\n\n" for these.
        let chunks = chunk_sections(&[section(Some("Lonely"), "")]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "");
    }

    // ---------- the MAX_CHARS boundary ----------

    #[test]
    fn content_of_exactly_max_chars_is_not_split() {
        let content = format!("{}\n{}", "a".repeat(499), "b".repeat(500)); // 499 + 1 + 500
        assert_eq!(content.chars().count(), MAX_CHARS);
        assert_eq!(chunk_sections(&[section(None, &content)]).len(), 1);
    }

    #[test]
    fn one_char_over_max_splits_at_the_paragraph_boundary() {
        let content = format!("{}\n{}", "a".repeat(499), "b".repeat(501)); // 1001
        let chunks = chunk_sections(&[section(None, &content)]);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content, "a".repeat(499));
        assert_eq!(chunks[1].content, "b".repeat(501));
    }

    #[test]
    fn size_is_measured_in_chars_not_bytes() {
        // 4 paragraphs of 300 multibyte chars: 1203 chars, 2406 bytes.
        // By chars: 3 paragraphs fit (902), the 4th starts a new chunk.
        // Counting bytes would split after every paragraph.
        let p = "é".repeat(300);
        let content = [p.as_str(); 4].join("\n");
        let chunks = chunk_sections(&[section(None, &content)]);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].content.chars().count(), 902);
    }

    // ---------- splitting and packing ----------

    #[test]
    fn oversized_section_packs_whole_paragraphs_greedily() {
        // 12 paragraphs of 80 chars = 12*80 + 11 newlines = 971; a 13th would make 1052.
        let paras: Vec<String> = (0..30).map(|i| para(i, 80)).collect();
        let chunks = chunk_sections(&[section(Some("Big"), &paras.join("\n"))]);

        assert_eq!(chunks.len(), 3); // 12 + 12 + 6
        assert_eq!(chunks[0].content, paras[..12].join("\n"));
        for c in &chunks {
            assert!(c.content.chars().count() <= MAX_CHARS);
            assert_eq!(c.heading.as_deref(), Some("Big"));
            assert_eq!(c.content, c.content.trim());
        }
        assert_eq!(rejoin(&chunks), paras.join("\n")); // nothing lost, nothing duplicated
    }

    #[test]
    fn a_single_oversized_paragraph_is_never_split() {
        let huge = "y".repeat(2500);
        let chunks = chunk_sections(&[section(None, &huge)]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content.chars().count(), 2500);
    }

    #[test]
    fn an_oversized_paragraph_is_isolated_from_its_neighbours() {
        let huge = "H".repeat(1500);
        let content = format!("small one\n{huge}\nsmall two");
        let chunks = chunk_sections(&[section(None, &content)]);
        let c: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        assert_eq!(c, vec!["small one", huge.as_str(), "small two"]);
    }

    #[test]
    fn blank_lines_between_paragraphs_are_dropped_when_splitting() {
        let paras: Vec<String> = (0..15).map(|i| para(i, 80)).collect();
        let content = paras.join("\n\n\n"); // 15*80 + 14*3 = 1242
        let chunks = chunk_sections(&[section(None, &content)]);
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().all(|c| !c.content.contains("\n\n")));
        assert_eq!(rejoin(&chunks), paras.join("\n"));
    }

    #[test]
    fn invariants_hold_for_pseudo_random_paragraph_lengths() {
        // xorshift so the test is deterministic without a rand dependency.
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };

        for case in 0..50 {
            let n = 5 + (next() % 60) as usize;
            let paras: Vec<String> = (0..n)
                .map(|i| {
                    let len = 10 + (next() % 290) as usize; // 10..=299
                    para(i, len)
                })
                .collect();
            let chunks = chunk_sections(&[section(Some("H"), &paras.join("\n"))]);

            assert!(!chunks.is_empty(), "case {case}");
            for c in &chunks {
                assert!(!c.content.is_empty(), "case {case}: empty chunk");
                assert!(
                    c.content.chars().count() <= MAX_CHARS,
                    "case {case}: oversize"
                );
                assert_eq!(c.heading.as_deref(), Some("H"));
            }
            assert_eq!(
                rejoin(&chunks),
                paras.join("\n"),
                "case {case}: content changed"
            );
        }
    }
}
