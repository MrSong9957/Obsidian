"""Tests for vault-checker.py — metadata completeness detection."""

import textwrap
from pathlib import Path

import pytest

from vault_checker import check_metadata, fix_metadata, md_files, parse_frontmatter, strip_frontmatter


# ── md_files ─────────────────────────────────────────────


def test_md_files_finds_all(tmp_path: Path) -> None:
    """递归找到所有 .md 文件。"""
    (tmp_path / "wiki").mkdir()
    (tmp_path / "wiki" / "a.md").write_text("")
    (tmp_path / "wiki" / "sub").mkdir()
    (tmp_path / "wiki" / "sub" / "b.md").write_text("")
    result = md_files(tmp_path, ["wiki"])
    names = {p.name for p in result}
    assert names == {"a.md", "b.md"}


def test_md_files_skips_index(tmp_path: Path) -> None:
    """跳过 _index.md。"""
    (tmp_path / "wiki").mkdir()
    (tmp_path / "wiki" / "_index.md").write_text("")
    (tmp_path / "wiki" / "real.md").write_text("")
    result = md_files(tmp_path, ["wiki"])
    names = {p.name for p in result}
    assert "_index.md" not in names
    assert names == {"real.md"}


def test_md_files_multiple_dirs(tmp_path: Path) -> None:
    """多目录扫描。"""
    (tmp_path / "wiki").mkdir()
    (tmp_path / "sources").mkdir()
    (tmp_path / "wiki" / "a.md").write_text("")
    (tmp_path / "sources" / "b.md").write_text("")
    result = md_files(tmp_path, ["wiki", "sources"])
    names = {p.name for p in result}
    assert names == {"a.md", "b.md"}


# ── parse_frontmatter ────────────────────────────────────


def test_parse_frontmatter_flat() -> None:
    """解析扁平 frontmatter。"""
    text = textwrap.dedent("""\
        ---
        name: test
        type: concept
        tags: [a, b]
        ---
        # Content
    """)
    fm = parse_frontmatter(text)
    assert fm["name"] == "test"
    assert fm["type"] == "concept"
    assert fm["tags"] == "a, b"


def test_parse_frontmatter_nested_metadata() -> None:
    """解析嵌套 metadata 块中的字段。"""
    text = textwrap.dedent("""\
        ---
        name: cognitive-coding-method
        description: 认知学习编程法
        metadata:
          node_type: memory
          type: reference
          updated: 2026-05-30
          tags:
            - 编程方法
            - 学习
        ---
    """)
    fm = parse_frontmatter(text)
    assert fm["name"] == "cognitive-coding-method"
    assert fm["type"] == "reference"  # from metadata block
    assert "编程方法" in fm["tags"]
    assert "学习" in fm["tags"]


def test_parse_frontmatter_no_frontmatter() -> None:
    """没有 frontmatter 时返回空 dict。"""
    fm = parse_frontmatter("# Just content\nNo frontmatter here.")
    assert fm == {}


def test_parse_frontmatter_empty_tags() -> None:
    """空 tags 列表。"""
    text = textwrap.dedent("""\
        ---
        name: test
        tags: []
        ---
    """)
    fm = parse_frontmatter(text)
    assert fm.get("tags", "") == ""


# ── check_metadata ───────────────────────────────────────


def test_check_metadata_complete(tmp_path: Path) -> None:
    """所有必需字段齐全时无缺失。"""
    p = tmp_path / "good.md"
    p.write_text(textwrap.dedent("""\
        ---
        name: test
        description: a test file
        type: concept
        updated: 2026-05-30
        tags: [test]
        source: https://example.com
        ---
    """))
    result = check_metadata(p)
    assert result["missing"] == []


def test_check_metadata_missing_tags(tmp_path: Path) -> None:
    """缺少 tags 时报告。"""
    p = tmp_path / "no_tags.md"
    p.write_text(textwrap.dedent("""\
        ---
        name: test
        description: a test file
        type: concept
        updated: 2026-05-30
        source: https://example.com
        ---
    """))
    result = check_metadata(p)
    assert "tags" in result["missing"]


def test_check_metadata_no_frontmatter(tmp_path: Path) -> None:
    """没有 frontmatter 时报告所有字段缺失。"""
    p = tmp_path / "bare.md"
    p.write_text("# Just a heading\nSome content.\n")
    result = check_metadata(p)
    assert set(result["missing"]) == {"name", "description", "type", "updated", "tags", "source"}


def test_check_metadata_nested_metadata_block(tmp_path: Path) -> None:
    """嵌套 metadata 块中的字段也能检测到。"""
    p = tmp_path / "nested.md"
    p.write_text(textwrap.dedent("""\
        ---
        name: cognitive-coding-method
        description: 认知学习编程法
        metadata:
          type: reference
          updated: 2026-05-30
          tags:
            - 编程方法
        ---
    """))
    result = check_metadata(p)
    assert result["missing"] == ["source"]


# ── strip_frontmatter ────────────────────────────────────


def test_strip_frontmatter_with_frontmatter() -> None:
    """有 frontmatter 时正确剥离。"""
    text = "---\nname: test\n---\n# Content\n"
    result = strip_frontmatter(text)
    assert result == "# Content\n"
    assert "---" not in result


def test_strip_frontmatter_no_frontmatter() -> None:
    """无 frontmatter 时原样返回。"""
    text = "# Just content\nNo frontmatter here.\n"
    result = strip_frontmatter(text)
    assert result == text


def test_strip_frontmatter_body_has_separator() -> None:
    """正文中有 --- 时不被误删。"""
    text = "---\nname: test\n---\nAbove\n\n---\n\nBelow\n"
    result = strip_frontmatter(text)
    assert result == "Above\n\n---\n\nBelow\n"


# ── fix_metadata ─────────────────────────────────────────


def test_fix_metadata_no_frontmatter(tmp_path: Path) -> None:
    """完全缺失时插入完整模板。"""
    p = tmp_path / "bare.md"
    p.write_text("# Heading\nSome content.\n")
    assert fix_metadata(p) is True
    text = p.read_text()
    assert text.startswith("---\n")
    assert "name: bare" in text
    assert "type: concept" in text
    assert "source: https://example.com" in text
    assert "# Heading" in text


def test_fix_metadata_partial(tmp_path: Path) -> None:
    """部分缺失时保留旧值、补全缺失字段。"""
    p = tmp_path / "partial.md"
    p.write_text(textwrap.dedent("""\
        ---
        name: old-name
        tags: [old]
        ---
        Content here.
    """))
    assert fix_metadata(p) is True
    text = p.read_text()
    assert "name: old-name" in text  # 保留旧值
    assert "tags: old" in text  # 保留旧值
    assert "description: 一句话说明" in text  # 补缺失
    assert "type: concept" in text  # 补缺失
    assert "source: https://example.com" in text  # 补缺失
    assert "Content here." in text


def test_fix_metadata_preserves_content(tmp_path: Path) -> None:
    """正文内容不丢失。"""
    body = "## Section\n\nParagraph with **bold** and *italic*.\n\n- item 1\n- item 2\n"
    p = tmp_path / "preserve.md"
    p.write_text("---\nname: x\n---\n" + body)
    fix_metadata(p)
    text = p.read_text()
    # frontmatter 之后的内容应完整保留
    assert text.endswith(body)


def test_fix_metadata_sources_to_source(tmp_path: Path) -> None:
    """旧字段 sources 映射到 source。"""
    p = tmp_path / "legacy.md"
    p.write_text(textwrap.dedent("""\
        ---
        name: my-doc
        description: test
        type: concept
        updated: 2026-05-30
        tags: [a, b]
        sources: https://old-url.com
        ---
        Body.
    """))
    fix_metadata(p)
    text = p.read_text()
    assert "source: https://old-url.com" in text
    assert "sources:" not in text
    assert check_metadata(p)["missing"] == []
