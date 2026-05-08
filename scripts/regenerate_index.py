#!/usr/bin/env python3
"""Regenerate _index.md for wiki/ and sources/ from actual directory contents."""

from __future__ import annotations

import re
import sys
from datetime import date
from pathlib import Path

VAULT = Path("/home/app/obsidian")
WIKI_DIR = VAULT / "wiki"
SOURCES_DIR = VAULT / "sources"
WIKI_INDEX = WIKI_DIR / "_index.md"
SOURCES_INDEX = SOURCES_DIR / "_index.md"
TODAY = date.today().isoformat()


def parse_frontmatter(text: str) -> dict[str, str]:
    """Extract YAML frontmatter key-value pairs from markdown text."""
    m = re.match(r"^---\n(.*?)\n---", text, re.DOTALL)
    if not m:
        return {}
    body = m.group(1)
    result: dict[str, str] = {}
    for line in body.splitlines():
        # Match "key: value" or "key: [a, b, c]"
        kv = re.match(r"^(\w+):\s*(.*)", line)
        if kv:
            val = kv.group(2).strip()
            # Strip surrounding quotes
            if len(val) >= 2 and val[0] == val[-1] and val[0] in ('"', "'"):
                val = val[1:-1]
            # Multi-line literal "description: |" → empty
            if val == "|":
                val = ""
            result[kv.group(1)] = val
    return result


def format_tags(raw: str) -> str:
    """Format tags value like '[a, b, c]' to 'a, b, c'."""
    return raw.strip("[]").strip()


def scan_wiki() -> list[dict]:
    """Scan wiki/*.md and return list of {file, name, description, tags}."""
    entries = []
    for f in sorted(WIKI_DIR.glob("*.md")):
        if f.name == "_index.md":
            continue
        fm = parse_frontmatter(f.read_text(encoding="utf-8"))
        entries.append({
            "file": f.stem,
            "name": fm.get("name", f.stem),
            "description": fm.get("description", ""),
            "tags": format_tags(fm.get("tags", "")),
        })
    return entries


def scan_sources_files() -> list[dict]:
    """Scan sources/*.md (top-level only) and return list of {file, name, description}."""
    entries = []
    for f in sorted(SOURCES_DIR.glob("*.md")):
        if f.name == "_index.md":
            continue
        fm = parse_frontmatter(f.read_text(encoding="utf-8"))
        entries.append({
            "file": f.stem,
            "name": fm.get("name", f.stem),
            "description": fm.get("description", ""),
        })
    return entries


def _clean_collection_desc(d: Path, md_files: list[Path]) -> str:
    """Derive a readable description for a source collection directory."""
    # 1. Try README.md frontmatter
    readme = d / "README.md"
    if readme.exists():
        text = readme.read_text(encoding="utf-8")
        fm = parse_frontmatter(text)
        if fm.get("name"):
            return fm["name"]
        if fm.get("description"):
            return fm["description"]
        # Look for first H1 heading, then fallback to any heading or text
        after_fm = re.sub(r"^---\n.*?\n---\n?", "", text, count=1, flags=re.DOTALL)
        h1 = first_text = ""
        for line in after_fm.splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("<"):
                continue
            if stripped.startswith("# ") and not h1:
                h1 = stripped.lstrip("# ").rstrip(".")
            elif not first_text:
                first_text = stripped.rstrip(".")
        if h1:
            return h1
    # 2. Try first .md file's description
    for f in sorted(md_files):
        if f.name == "README.md":
            continue
        fm = parse_frontmatter(f.read_text(encoding="utf-8"))
        if fm.get("description"):
            return fm["description"]
    # 3. Fallback to directory name
    return d.name


def scan_sources_collections() -> list[dict]:
    """Scan sources/ subdirectories and return list of {dir, count, description}."""
    entries = []
    for d in sorted(SOURCES_DIR.iterdir()):
        if not d.is_dir():
            continue
        md_files = [f for f in d.rglob("*.md") if f.name != "README.md"]
        entries.append({
            "dir": d.name,
            "count": len(md_files),
            "description": _clean_collection_desc(d, md_files),
        })
    return entries


def generate_wiki_index(entries: list[dict]) -> str:
    """Generate wiki/_index.md content."""
    lines = [
        "---",
        "type: index",
        f"updated: {TODAY}",
        "---",
        "",
        "# Wiki 索引",
        "",
        "| 页面 | 说明 | 标签 |",
        "|------|------|------|",
    ]
    for e in entries:
        link = f"[[wiki/{e['file']}\\|{e['name']}]]"
        desc = e["description"].replace("|", "\\|")
        # Truncate long descriptions
        if len(desc) > 60:
            desc = desc[:57] + "..."
        lines.append(f"| {link} | {desc} | {e['tags']} |")
    lines.append("")
    return "\n".join(lines)


def generate_sources_index(files: list[dict], collections: list[dict]) -> str:
    """Generate sources/_index.md content."""
    lines = [
        "---",
        "type: index",
        f"updated: {TODAY}",
        "---",
        "",
        "# 原始资料索引",
        "",
        "## 独立文件",
        "",
        "| 文件 | 说明 |",
        "|------|------|",
    ]
    for e in files:
        link = f"[[sources/{e['file']}\\|{e['name']}]]"
        desc = e["description"].replace("|", "\\|")
        if len(desc) > 80:
            desc = desc[:77] + "..."
        lines.append(f"| {link} | {desc} |")

    lines.extend([
        "",
        "## 集合",
        "",
        "| 路径 | 文件数 | 说明 |",
        "|------|--------|------|",
    ])
    for e in collections:
        link = f"[[sources/{e['dir']}/\\|{e['dir']}]]"
        desc = e["description"].replace("|", "\\|")
        lines.append(f"| {link} | {e['count']} | {desc} |")

    lines.append("")
    return "\n".join(lines)


def main() -> None:
    errors = []

    # Generate wiki index
    wiki_entries = scan_wiki()
    wiki_content = generate_wiki_index(wiki_entries)
    WIKI_INDEX.write_text(wiki_content, encoding="utf-8")
    print(f"wiki/_index.md: {len(wiki_entries)} pages indexed")

    # Generate sources index
    source_files = scan_sources_files()
    source_collections = scan_sources_collections()
    sources_content = generate_sources_index(source_files, source_collections)
    SOURCES_INDEX.write_text(sources_content, encoding="utf-8")
    print(f"sources/_index.md: {len(source_files)} files + {len(source_collections)} collections indexed")

    if errors:
        print(f"\nWarnings:\n" + "\n".join(errors), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
