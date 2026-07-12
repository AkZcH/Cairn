import re

MAX_CHARS = 1000
HEADING_RE = re.compile(r"^#{1,6}\s+(.*)$")


def parse_markdown(raw: str) -> list[dict]:
    """Splits on markdown headings, mirroring the Rust parser's approach.
    Simpler than a full CommonMark parse (no list-item handling like the
    fix we just shipped in ingest/), good enough for casual web pastes,
    can converge with the Rust logic later if it proves worth it."""
    lines = raw.splitlines()
    sections = []
    current_heading = None
    current_lines: list[str] = []

    def flush():
        content = "\n".join(current_lines).strip()
        if content or current_heading:
            sections.append({"heading": current_heading, "content": content})

    for line in lines:
        m = HEADING_RE.match(line)
        if m:
            flush()
            current_heading = m.group(1).strip()
            current_lines = []
        else:
            current_lines.append(line)
    flush()

    return sections or [{"heading": None, "content": raw.strip()}]


def chunk_sections(sections: list[dict]) -> list[dict]:
    """Same greedy paragraph-packing strategy as ingest/src/chunker.rs:
    a section that fits stays whole, an oversized one splits on blank-line
    paragraph boundaries, never mid-sentence."""
    chunks = []
    for section in sections:
        content = section["content"]
        if len(content) <= MAX_CHARS:
            if content.strip():
                chunks.append({"heading": section["heading"], "content": content})
            continue

        paragraphs = [p.strip() for p in content.split("\n\n") if p.strip()]
        current = ""
        for para in paragraphs:
            would_be = len(current) + len(para) + 1
            if current and would_be > MAX_CHARS:
                chunks.append({"heading": section["heading"], "content": current.strip()})
                current = ""
            current = f"{current}\n{para}" if current else para
        if current.strip():
            chunks.append({"heading": section["heading"], "content": current.strip()})

    return chunks