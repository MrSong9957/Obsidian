#!/usr/bin/env python3
"""Detect missing tags and metadata in Obsidian vault .md files."""

from __future__ import annotations

import argparse
import re
import sys
from datetime import date
from pathlib import Path

VAULT_PATH = Path("/home/app/obsidian")
SCAN_DIRS = ["wiki", "sources"]
SKIP_FILES = ["_index.md"]
REQUIRED_FIELDS = ["name", "description", "type", "updated", "tags", "source"]


_FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n?", re.DOTALL)


def md_files(vault: Path, dirs: list[str]) -> list[Path]:
    """Recursively find .md files in given dirs, skipping SKIP_FILES."""
    result: list[Path] = []
    for d in dirs:
        base = vault / d
        if not base.is_dir():
            continue
        for p in sorted(base.rglob("*.md")):
            if p.name not in SKIP_FILES:
                result.append(p)
    return result


def parse_frontmatter(text: str) -> dict[str, str]:
    """Extract YAML frontmatter key-value pairs.

    Handles:
    - Flat keys: ``key: value``
    - Inline lists: ``key: [a, b, c]`` → ``"a, b, c"``
    - Nested ``metadata:`` block with indented keys
    - Multi-line list items under metadata: ``- item`` → ``"item1, item2"``
    """
    m = _FRONTMATTER_RE.match(text)
    if not m:
        return {}

    body = m.group(1)
    result: dict[str, str] = {}
    in_metadata = False
    meta_tags: list[str] = []

    for line in body.splitlines():
        # Detect start of metadata block
        if re.match(r"^metadata:\s*$", line):
            in_metadata = True
            continue

        if in_metadata:
            # Indented key-value inside metadata block
            kv = re.match(r"^  (\w+):\s*(.*)", line)
            if kv:
                key, val = kv.group(1), kv.group(2).strip()
                if val == "|" or val == "":
                    # Multi-line value — tags will come as "- item" lines
                    continue
                if len(val) >= 2 and val[0] == val[-1] and val[0] in ('"', "'"):
                    val = val[1:-1]
                result[key] = val
                continue
            # Multi-line list item (e.g. "  - tag1")
            item = re.match(r"^\s+-\s+(.*)", line)
            if item:
                meta_tags.append(item.group(1).strip())
                continue
            # Unindented line — metadata block ended
            in_metadata = False

        # Top-level key-value
        kv = re.match(r"^(\w+):\s*(.*)", line)
        if kv:
            key, val = kv.group(1), kv.group(2).strip()
            if len(val) >= 2 and val[0] == val[-1] and val[0] in ('"', "'"):
                val = val[1:-1]
            if val == "|":
                val = ""
            # Strip brackets from inline lists: [a, b] → a, b
            if val.startswith("[") and val.endswith("]"):
                val = val[1:-1].strip()
            result[key] = val

    # Flush collected metadata tags
    if meta_tags:
        result["tags"] = ", ".join(meta_tags)

    return result


def strip_frontmatter(text: str) -> str:
    """Remove the leading YAML frontmatter block, return remaining content."""
    return _FRONTMATTER_RE.sub("", text, count=1)


def fix_metadata(filepath: Path) -> bool:
    """Strip old frontmatter, merge old values with defaults, write back. Returns True."""
    text = filepath.read_text(encoding="utf-8")
    old = parse_frontmatter(text)
    body = strip_frontmatter(text)

    defaults = {
        "name": filepath.stem,
        "description": "一句话说明",
        "type": "concept",
        "updated": date.today().isoformat(),
        "tags": "[tag1, tag2]",
        "source": "https://example.com",
    }

    # Map legacy field names to canonical ones
    if "source" not in old and "sources" in old:
        old["source"] = old.pop("sources")

    merged = {**defaults, **old}

    lines = ["---"]
    for key in REQUIRED_FIELDS:
        lines.append(f"{key}: {merged[key]}")
    lines.append("---")

    filepath.write_text("\n".join(lines) + "\n" + body, encoding="utf-8")
    return True


def check_metadata(filepath: Path) -> dict:
    """Check a single file for required metadata fields.

    Returns ``{"file": Path, "missing": list[str]}``.
    """
    text = filepath.read_text(encoding="utf-8")
    fm = parse_frontmatter(text)
    missing = [f for f in REQUIRED_FIELDS if not fm.get(f)]
    return {"file": filepath, "missing": missing}


def print_report(results: list[dict]) -> None:
    """Print a terminal report of missing metadata."""
    issues = [r for r in results if r["missing"]]
    if not issues:
        print("All files have complete metadata.")
        return
    print(f"Found {len(issues)} file(s) with missing metadata:\n")
    for r in issues:
        rel = r["file"]
        missing = ", ".join(r["missing"])
        print(f"  {rel}: missing [{missing}]")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Detect and optionally fix missing metadata in Obsidian vault .md files."
    )
    parser.add_argument(
        "--fix",
        action="store_true",
        help="Fix missing metadata by inserting defaults for absent fields.",
    )
    args = parser.parse_args()

    files = md_files(VAULT_PATH, SCAN_DIRS)
    results = [check_metadata(f) for f in files]

    if args.fix:
        issues = [r for r in results if r["missing"]]
        if not issues:
            print("All files have complete metadata. Nothing to fix.")
            return
        fixed = 0
        for r in issues:
            fix_metadata(r["file"])
            fixed += 1
            missing = ", ".join(r["missing"])
            print(f"  Fixed {r['file']}: added [{missing}]")
        print(f"\nFixed {fixed} file(s).")
    else:
        print_report(results)


if __name__ == "__main__":
    main()
