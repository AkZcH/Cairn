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
            Event::End(TagEnd::Paragraph) => {
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
    raw.split("\n\n")
        .map(|block| block.trim())
        .filter(|block| !block.is_empty())
        .map(|block| Section {
            heading: None,
            content: block.to_string(),
        })
        .collect()
}
