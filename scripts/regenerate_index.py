#!/usr/bin/env python3
"""Wiki 维护脚本。

两个子命令:
- index (默认): 扫描 wiki/ 和 sources/,重新生成 _index.md
- fix-frontmatter: 归一化 sources/ + wiki/ 下文件的 frontmatter 结构

VAULT 路径解析顺序: --vault 参数 > $WIKI_VAULT_ROOT > 脚本所在目录的父目录
(脚本在 <vault>/scripts/ 下,父目录即 vault 根)。
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from datetime import date, datetime
from pathlib import Path


def resolve_vault(cli_vault: str | None) -> Path:
    """解析 vault 根目录。优先级:CLI > 环境变量 > 脚本父目录的父目录。"""
    if cli_vault:
        return Path(cli_vault).resolve()
    env = os.environ.get("WIKI_VAULT_ROOT")
    if env:
        return Path(env).resolve()
    # 脚本在 <vault>/scripts/regenerate_index.py,父目录的父目录即 vault 根
    return Path(__file__).resolve().parent.parent


# 这三个在 main() 里按解析出的 vault 设置,模块级保留供原函数引用
VAULT = resolve_vault(None)
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


# ----------------------------------------------------------------------------
# fix-frontmatter: 归一化 sources/ + wiki/ 下文件的 frontmatter 结构
# ----------------------------------------------------------------------------

# 占位符特征:sources/claude-code/claude-code-system-prompts/ 下大量第三方文件
# 套了未填的模板 frontmatter。本脚本只处理用户自家 sources/ + wiki/ 层。
# 系统占位符文本(匹配这些值的字段会被标记为"需重写"或回填)。
PLACEHOLDER_DESC = "一句话说明"
PLACEHOLDER_TAGS = "[tag1, tag2]"
PLACEHOLDER_SOURCE = "https://example.com"

# sync 读取的标准字段顺序(frontmatter.rs / serialize 输出一致)
FIELD_ORDER = ["name", "description", "type", "updated", "tags", "source"]

# 第三方资料目录,fix-frontmatter 默认跳过(避免动 251 个 system-prompts 文件)
SKIP_DIRS = {"claude-code-system-prompts"}


def split_frontmatter(text: str) -> tuple[str, str, str] | None:
    """拆分 (frontmatter_body, body, full_fmdelim)。

    返回 (fm_block_content_without_delimiters, body_after_second_delim, raw_fm_with_delimiters)
    无 frontmatter 返回 None。
    """
    m = re.match(r"^---\n(.*?)\n---\n?(.*)$", text, re.DOTALL)
    if not m:
        return None
    return m.group(1), m.group(2), text[: m.end() - len(m.group(2))]


def parse_fm_lines(block: str) -> tuple[dict[str, str], list[str], dict[str, str]]:
    """解析 frontmatter 块,支持平铺和嵌套 metadata: 块。

    返回 (flat_fields, original_order, nested_under_metadata)。
    flat_fields: 顶层 key: value 字段
    nested_under_metadata: metadata: 下的缩进字段(yazi.md 那种)
    """
    flat: dict[str, str] = {}
    order: list[str] = []
    nested: dict[str, str] = {}
    in_metadata = False
    for line in block.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        # key: value(对 stripped 行匹配,兼容缩进的嵌套字段)
        m = re.match(r"^([A-Za-z_][\w]*)\s*:\s*(.*)$", stripped)
        if not m:
            continue
        key, val = m.group(1), m.group(2).strip()
        # 去引号
        if len(val) >= 2 and val[0] == val[-1] and val[0] in ('"', "'"):
            val = val[1:-1]
        if key == "metadata":
            in_metadata = True
            continue
        if in_metadata:
            # 缩进行属于 metadata 块
            nested[key] = val
            continue
        flat[key] = val
        if key not in order:
            order.append(key)
    return flat, order, nested


def normalize_frontmatter(text: str, file_mtime: datetime) -> tuple[str, list[str]] | None:
    """归一化单个文件的 frontmatter。

    做的事(仅当有实质语义改动时才返回结果):
    1. 平铺嵌套 metadata: 块(yazi.md 那种)
    2. 把字段重排为标准顺序 name→description→type→updated→tags→source
    3. updated 缺失时用文件 mtime 回填
    4. 去掉行内注释(type: foo # bar → type: foo)

    不做的事:
    - 不改 name/description/tags/source 的内容(占位符原样保留,留给 AI 填)
    - 不动正文

    返回 (new_full_text, changed_fields_list) 或 None(无需改动)。
    """
    split = split_frontmatter(text)
    if split is None:
        return None  # 无 frontmatter,本次不处理(交给 AI 单独加)
    block, body, _ = split
    flat, _, nested = parse_fm_lines(block)

    # 合并嵌套字段到 flat(嵌套优先,因为它往往是更完整的真实值)
    for k, v in nested.items():
        flat[k] = v

    # 去行内注释:value 里 " # ..." 之后的部分丢掉
    cleaned: dict[str, str] = {}
    for k, v in flat.items():
        # 只对看起来像 "value  # comment" 的处理,URL 里的 # 不动
        if " #" in v and not v.startswith("http"):
            v = v.split(" #")[0].rstrip()
        cleaned[k] = v

    # updated 回填:占位/缺失时用 mtime
    updated_val = cleaned.get("updated", "").strip()
    if not updated_val:
        cleaned["updated"] = file_mtime.strftime("%Y-%m-%d")

    # 按 FIELD_ORDER 重排,多余字段追加在后
    ordered: list[tuple[str, str]] = []
    seen = set()
    for k in FIELD_ORDER:
        if k in cleaned:
            ordered.append((k, cleaned[k]))
            seen.add(k)
    for k, v in cleaned.items():
        if k not in seen:
            ordered.append((k, v))

    # 判断有无实质语义改动:字段值、字段集、嵌套块、顺序(对照 FIELD_ORDER)
    old_flat, _, old_nested = parse_fm_lines(block)
    old_merged = {**old_nested, **old_flat}
    new_dict = dict(ordered)

    changed: list[str] = []
    # 嵌套平铺
    if nested:
        changed.append("(flattened nested metadata block)")
    # 字段值变化(包括新增字段)
    all_keys = set(old_merged) | set(new_dict)
    for k in sorted(all_keys):
        old_v = old_merged.get(k, "<missing>")
        new_v = new_dict.get(k, "<removed>")
        if old_v != new_v:
            changed.append(f"{k}: {old_v!r} -> {new_v!r}")

    if not changed:
        return None  # 无实质改动,不动文件(避免纯空白差异误改)

    # 重建 frontmatter 块
    new_fm = "---\n" + "\n".join(f"{k}: {v}" for k, v in ordered) + "\n---\n\n"
    new_text = new_fm + body
    return new_text, changed


def iter_target_md(vault: Path) -> list[Path]:
    """遍历 sources/ + wiki/ 下的 .md,跳过 _index.md 和 SKIP_DIRS。"""
    targets: list[Path] = []
    for root_dir in [vault / "sources", vault / "wiki"]:
        if not root_dir.is_dir():
            continue
        for p in sorted(root_dir.rglob("*.md")):
            if p.name == "_index.md":
                continue
            # 跳过第三方资料目录
            if any(part in SKIP_DIRS for part in p.parts):
                continue
            targets.append(p)
    return targets


def cmd_fix_frontmatter(vault: Path, apply: bool) -> int:
    """执行 frontmatter 归一化。apply=False 时只 dry-run 报告。"""
    targets = iter_target_md(vault)
    print(f"扫描 {len(targets)} 个候选文件(sources/ + wiki/,跳过 {SKIP_DIRS})")
    changed_count = 0
    for p in targets:
        text = p.read_text(encoding="utf-8")
        mtime = datetime.fromtimestamp(p.stat().st_mtime)
        result = normalize_frontmatter(text, mtime)
        if result is None:
            continue
        new_text, changes = result
        changed_count += 1
        rel = p.relative_to(vault)
        print(f"\n=== {rel} ===")
        for c in changes:
            print(f"  {c}")
        if apply:
            p.write_text(new_text, encoding="utf-8")
            print("  [已写入]")
        else:
            print("  [dry-run,未写入]")
    mode = "已修改" if apply else "需修改(dry-run)"
    print(f"\n共 {mode} {changed_count} 个文件")
    return 0


def cmd_index(vault: Path) -> int:
    """原始功能:生成 _index.md。"""
    global WIKI_DIR, SOURCES_DIR, WIKI_INDEX, SOURCES_INDEX
    WIKI_DIR = vault / "wiki"
    SOURCES_DIR = vault / "sources"
    WIKI_INDEX = WIKI_DIR / "_index.md"
    SOURCES_INDEX = SOURCES_DIR / "_index.md"

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
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(description="Wiki 维护脚本")
    parser.add_argument("--vault", help="vault 根目录(默认 $WIKI_VAULT_ROOT 或脚本父目录)")
    sub = parser.add_subparsers(dest="cmd")

    sub.add_parser("index", help="重新生成 wiki/_index.md 和 sources/_index.md")
    ff = sub.add_parser("fix-frontmatter", help="归一化 sources/ + wiki/ 的 frontmatter 结构")
    ff.add_argument("--apply", action="store_true", help="实际写入(默认 dry-run)")

    args = parser.parse_args()
    vault = resolve_vault(args.vault)

    if args.cmd == "fix-frontmatter":
        sys.exit(cmd_fix_frontmatter(vault, apply=args.apply))
    else:
        # 默认(无子命令或 index):生成 _index
        sys.exit(cmd_index(vault))


if __name__ == "__main__":
    main()
