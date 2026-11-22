//! Parses markdown/plaintext into structural sections (split on headings).
//!
//! A "section" is everything under one heading, up to the next heading.
//! We're deliberately not building a full markdown AST — just enough
//! structure to chunk sensibly later.

use pulldown_cmark::{Event, Parser as MdParser, Tag, TagEnd};

#[derive(Debug, Clone)]
pub struct Section {
    pub heading: Option<String>,
    pub content: String,
}

/// Parses raw markdown text into a list of sections.
/// Content with no heading above it becomes a section with `heading: None`.
pub fn parse_markdown(raw: &str) -> Vec<Section> {
    let parser = MdParser::new(raw);

    let mut sections = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_content = String::new();
    let mut in_heading = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // level is HeadingLevel::H1..=H6 — we don't need to
                // distinguish levels for this pass, any heading starts
                // a new section.
                let _ = level;

                if !current_content.trim().is_empty() || current_heading.is_some() {
                    sections.push(Section {
                        heading: current_heading.take(),
                        content: current_content.trim().to_string(),
                    });
                }
                current_content = String::new();
                in_heading = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
            }
            Event::Text(text) | Event::Code(text) => {
                if in_heading {
                    current_heading
                        .get_or_insert_with(String::new)
                        .push_str(&text);
                } else {
                    current_content.push_str(&text);
                    current_content.push(' ');
                }
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Item) => {
                // List items ("tight" lists, per CommonMark) don't emit
                // Paragraph events, only Item ones. Without this, every
                // bullet point in a markdown list runs into the next with
                // just a space, collapsing an entire list (like the PRD's
                // Tech Stack section) into one undifferentiated blob
                // instead of separable, individually-embeddable facts.
                current_content.push('\n');
            }
            Event::SoftBreak | Event::HardBreak => {
                current_content.push('\n');
            }
            _ => {}
        }
    }

    if !current_content.trim().is_empty() || current_heading.is_some() {
        sections.push(Section {
            heading: current_heading,
            content: current_content.trim().to_string(),
        });
    }

    sections
}

/// Fallback for plain text (no markdown structure): blank-line-separated
/// blocks become sections, all with `heading: None`.
pub fn parse_plaintext(raw: &str) -> Vec<Section> {
    let normalized = raw.replace("\r\n", "\n");
    normalized
        .split("\n\n")
        .map(|block| block.trim())
        .filter(|block| !block.is_empty())
        .map(|block| Section {
            heading: None,
            content: block.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trimmed, non-empty lines of a section.
    /// The parser appends a space after every text event, so asserting on
    /// whole-line strings would be brittle.
    fn lines(section: &Section) -> Vec<String> {
        section
            .content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    fn headings(sections: &[Section]) -> Vec<Option<&str>> {
        sections.iter().map(|s| s.heading.as_deref()).collect()
    }

    // A tight list with no blank lines between items. This specifically
    // exercises the Item events that the old implementation failed to split.
    const TIGHT_LIST: &str = "\
# Gadgets

## Shopping list
- zephyrquartz widget for the garage
- obsidian-lamp for the study
- plain notebook

## Misc
The marker phrase is flibbertigibbet-7731.
";

    // ---------- section structure ----------

    #[test]
    fn empty_and_blank_input_yield_no_sections() {
        assert!(parse_markdown("").is_empty());
        assert!(parse_markdown("   \n\n \t\n").is_empty());
    }

    #[test]
    fn splits_on_every_heading() {
        let s = parse_markdown("# Title\n\nIntro.\n\n## A\n\nBody A.\n\n## B\n\nBody B\n");
        assert_eq!(headings(&s), vec![Some("Title"), Some("A"), Some("B")]);

        let bodies: Vec<&str> = s.iter().map(|x| x.content.as_str()).collect();
        assert_eq!(bodies, vec!["Intro.", "Body A.", "Body B"]);
    }

    #[test]
    fn all_six_heading_levels_start_a_section() {
        let md = (1..=6)
            .map(|n| format!("{} H{n}\n\nbody {n}\n", "#".repeat(n)))
            .collect::<Vec<_>>()
            .join("\n");

        let s = parse_markdown(&md);

        assert_eq!(s.len(), 6);

        for (i, sec) in s.iter().enumerate() {
            let want = format!("H{}", i + 1);
            assert_eq!(sec.heading.as_deref(), Some(want.as_str()));
        }
    }

    #[test]
    fn setext_headings_work_like_atx_headings() {
        let s = parse_markdown("Title\n=====\n\nBody\n");

        assert_eq!(s.len(), 1);
        assert_eq!(s[0].heading.as_deref(), Some("Title"));
        assert_eq!(s[0].content, "Body");
    }

    #[test]
    fn text_before_the_first_heading_becomes_a_headingless_section() {
        let s = parse_markdown("Preamble text.\n\n# Heading\n\nBody.\n");

        assert_eq!(s.len(), 2);
        assert_eq!(s[0].heading, None);
        assert_eq!(s[0].content, "Preamble text.");
        assert_eq!(s[1].heading.as_deref(), Some("Heading"));
        assert_eq!(s[1].content, "Body.");
    }

    #[test]
    fn heading_with_no_body_is_kept() {
        let s = parse_markdown("# Lonely\n");

        assert_eq!(s.len(), 1);
        assert_eq!(s[0].heading.as_deref(), Some("Lonely"));
        assert_eq!(s[0].content, "");
    }

    #[test]
    fn consecutive_headings_each_get_a_section() {
        let s = parse_markdown("# One\n\n## Two\n\nBody\n");

        assert_eq!(headings(&s), vec![Some("One"), Some("Two")]);
        assert_eq!(s[0].content, "");
        assert_eq!(s[1].content, "Body");
    }

    #[test]
    fn empty_heading_marker_does_not_invent_a_heading() {
        let s = parse_markdown("#\n\nbody\n");

        assert_eq!(s.len(), 1);
        assert_eq!(s[0].heading, None);
        assert_eq!(s[0].content, "body");
    }

    #[test]
    fn inline_formatting_in_headings_is_flattened_to_plain_text() {
        let s = parse_markdown("# Hello **world**\n\n# Use `cargo`\n");

        assert_eq!(headings(&s), vec![Some("Hello world"), Some("Use cargo")]);
    }

    #[test]
    fn many_sections_keep_their_order() {
        let md: String = (0..100)
            .map(|i| format!("## H{i}\n\nbody {i}\n\n"))
            .collect();

        let s = parse_markdown(&md);

        assert_eq!(s.len(), 100);

        for (i, sec) in s.iter().enumerate() {
            let want = format!("H{i}");
            assert_eq!(sec.heading.as_deref(), Some(want.as_str()));
            assert_eq!(sec.content, format!("body {i}"));
        }
    }

    // ---------- paragraph and line handling ----------

    #[test]
    fn paragraphs_end_up_on_separate_lines() {
        let s = parse_markdown("# T\n\nFirst paragraph.\n\nSecond paragraph.\n");

        assert_eq!(lines(&s[0]), vec!["First paragraph.", "Second paragraph."]);
    }

    #[test]
    fn soft_and_hard_breaks_become_newlines() {
        let soft = parse_markdown("alpha\nbeta\n");
        assert_eq!(lines(&soft[0]), vec!["alpha", "beta"]);

        let hard = parse_markdown("alpha  \nbeta\n");
        assert_eq!(lines(&hard[0]), vec!["alpha", "beta"]);
    }

    #[test]
    fn inline_code_text_is_kept() {
        let s = parse_markdown("Run `cargo test` now.\n");

        assert!(s[0].content.contains("cargo test"));
    }

    #[test]
    fn code_blocks_and_blockquotes_keep_their_text() {
        let s = parse_markdown("## Code\n\n```\nlet x = 1;\n```\n\n> quoted text\n");

        assert!(s[0].content.contains("let x = 1;"));
        assert!(s[0].content.contains("quoted text"));
    }

    // ---------- lists ----------

    #[test]
    fn tight_list_items_stay_on_separate_lines() {
        let s = parse_markdown(TIGHT_LIST);

        let shopping = s
            .iter()
            .find(|section| section.content.contains("zephyrquartz"))
            .expect("shopping list section should exist");

        let list_lines: Vec<&str> = shopping.content.lines().map(str::trim).collect();

        assert!(list_lines.contains(&"zephyrquartz widget for the garage"));
        assert!(list_lines.contains(&"obsidian-lamp for the study"));
        assert!(list_lines.contains(&"plain notebook"));
    }

    #[test]
    fn tight_list_items_stay_on_separate_lines_generic() {
        let s = parse_markdown("## Stack\n\n- Alpha item\n- Beta item\n- Gamma item\n");

        assert_eq!(s.len(), 1);
        assert_eq!(lines(&s[0]), vec!["Alpha item", "Beta item", "Gamma item"]);
    }

    #[test]
    fn ordered_list_items_stay_on_separate_lines() {
        let s = parse_markdown("## Steps\n\n1. First\n2. Second\n3. Third\n");

        assert_eq!(lines(&s[0]), vec!["First", "Second", "Third"]);
    }

    #[test]
    fn loose_list_items_stay_on_separate_lines() {
        let s = parse_markdown("## Loose\n\n- One\n\n- Two\n\n- Three\n");

        assert_eq!(lines(&s[0]), vec!["One", "Two", "Three"]);
    }

    #[test]
    fn nested_lists_do_not_lose_content() {
        let s = parse_markdown("## N\n\n- Parent\n  - Child\n- Sibling\n");

        let l = lines(&s[0]);
        let all = l.join(" ");

        for word in ["Parent", "Child", "Sibling"] {
            assert!(all.contains(word), "missing {word} in {all:?}");
        }

        assert!(l.iter().any(|x| x == "Sibling"));
    }

    #[test]
    fn prd_style_bold_prefixed_list_keeps_each_fact_on_its_own_line() {
        let md = "## Tech Stack\n\n**Core**:\n\
                  - **Python** (primary)\n\
                  - **Vector Store**: PostgreSQL + pgvector\n\
                  - **Knowledge Graph**: Neo4j\n";

        let s = parse_markdown(md);
        let l = lines(&s[0]);

        let vector_line = l
            .iter()
            .find(|x| x.contains("Vector Store"))
            .expect("vector store line missing");

        assert!(vector_line.contains("pgvector"));
        assert!(!vector_line.contains("Neo4j"));
        assert!(!vector_line.contains("Python"));
    }

    // ---------- robustness ----------

    #[test]
    fn crlf_and_lf_documents_parse_identically() {
        let lf = "# Title\n\nIntro line.\n\n## List\n\n- one\n- two\n";
        let crlf = lf.replace('\n', "\r\n");

        let a = parse_markdown(lf);
        let b = parse_markdown(&crlf);

        assert_eq!(a.len(), b.len());

        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x.heading, y.heading);
            assert_eq!(x.content, y.content);
        }
    }

    #[test]
    fn unicode_survives() {
        let s = parse_markdown("# Café ☕\n\nNaïve résumé 日本語.\n");

        assert_eq!(s[0].heading.as_deref(), Some("Café ☕"));
        assert_eq!(s[0].content, "Naïve résumé 日本語.");
    }

    #[test]
    fn odd_inputs_never_panic_and_never_yield_empty_sections() {
        let mut inputs: Vec<String> = [
            "\0",
            "\u{FFFF}",
            "<<<>>>",
            "[[]]((()))",
            "**__``",
            "# \n# \n# ",
            "- \n- \n-",
            "> > > >",
            "```",
            "~~~\n~~~",
            "&amp; &lt; &#0;",
            "\r\r\r",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        inputs.push("#".repeat(50));
        inputs.push("- ".repeat(500));
        inputs.push("> ".repeat(100));

        for input in inputs {
            for s in parse_markdown(&input) {
                assert!(
                    s.heading.is_some() || !s.content.is_empty(),
                    "empty section from {input:?}"
                );

                assert_eq!(s.content, s.content.trim());
            }
        }
    }

    // ---------- parse_plaintext ----------

    #[test]
    fn plaintext_empty_and_blank_yield_nothing() {
        assert!(parse_plaintext("").is_empty());
        assert!(parse_plaintext(" \n\n \n\n ").is_empty());
    }

    #[test]
    fn plaintext_splits_on_blank_lines_and_trims() {
        let s = parse_plaintext("  first  \n\n\n\n second \n\nthird");

        let c: Vec<&str> = s.iter().map(|x| x.content.as_str()).collect();

        assert_eq!(c, vec!["first", "second", "third"]);
        assert!(s.iter().all(|x| x.heading.is_none()));
    }

    #[test]
    fn plaintext_single_newlines_stay_inside_one_section() {
        let s = parse_plaintext("line one\nline two");

        assert_eq!(s.len(), 1);
        assert_eq!(s[0].content, "line one\nline two");
    }

    #[test]
    fn gap_plaintext_crlf_paragraphs_split() {
        assert_eq!(parse_plaintext("a\r\n\r\nb").len(), 2);
    }
}
