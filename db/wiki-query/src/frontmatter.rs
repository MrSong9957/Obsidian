//! frontmatter 解析、序列化。
//!
//! sync 建索引时用。v5 删除了强校验(write 工具已删,无拦截点),
//! frontmatter 规范降级为 AGENTS.md 约定 + templates/project-rules.md 注入。
//! 这里只保留 parse/serialize 供 sync 索引时提取字段。

use std::collections::BTreeMap;

/// frontmatter 字段集合。对应 .md 文件 `---` 之间的 YAML 块。
///
/// 标准 5 字段(name/description/type/updated/tags)单独存取;
/// `extras` 透传其他字段(如 source/aliases/confidence/status),
/// 保证 serialize 往返不丢数据。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,
    pub doc_type: String,
    pub updated: String,
    pub tags: String,
    /// 非标准字段,按出现顺序保留(用 BTreeMap 保证 serialize 输出稳定)。
    pub extras: BTreeMap<String, String>,
}

/// 解析 frontmatter + 正文。返回 (Frontmatter, body)。
///
/// 行为约定:
/// - 不以 `---` 开头 → 返回空 Frontmatter + 原文(整个当 body)
/// - 以 `---` 开头但无闭合 `---` → 返回空 Frontmatter + 原文(回退,不误解析)
/// - 正常闭合 → 解析字段 + 第二个 `---` 之后为 body
pub fn parse(content: &str) -> (Frontmatter, String) {
    let mut fm = Frontmatter::default();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return (fm, content.to_string());
    }

    // 用 Option 表达闭合位置:None = 未闭合(回退为原文)。
    let close_idx: Option<usize> = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim() == "---")
        .map(|(i, _)| i);

    let close = match close_idx {
        Some(i) => i,
        None => return (fm, content.to_string()), // 未闭合,回退
    };

    // 解析 frontmatter 区间 (1..close) 的 key: value 行。
    // 标准 5 字段单独存;其余键(source/aliases/confidence/status 等)
    // 收集到 extras,保证 serialize 往返不丢字段。
    for line in &lines[1..close] {
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();
            match key {
                "name" => fm.name = value.to_string(),
                "description" => fm.description = value.to_string(),
                "type" => fm.doc_type = value.to_string(),
                "updated" => fm.updated = value.to_string(),
                "tags" => fm.tags = value.to_string(),
                // 跳过空 key(缩进行如子映射)和注释
                _ if !key.is_empty() && !key.starts_with('#') => {
                    fm.extras.insert(key.to_string(), value.to_string());
                }
                _ => {}
            }
        }
    }

    let body = if close + 1 < lines.len() {
        lines[(close + 1)..].join("\n")
    } else {
        String::new()
    };

    (fm, body)
}

/// 把 Frontmatter + body 重组为完整 Markdown(frontmatter + 正文)。
///
/// updated 为空时自动填今天(chrono)。
/// extras 字段原样写回,保证读-改-写往返不丢数据。
pub fn serialize(fm: &Frontmatter, body: &str) -> String {
    let updated = if fm.updated.trim().is_empty() {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    } else {
        fm.updated.clone()
    };
    let mut out = format!(
        "---\nname: {}\ndescription: {}\ntype: {}\nupdated: {}\ntags: {}",
        fm.name, fm.description, fm.doc_type, updated, fm.tags
    );
    // extras 按 BTreeMap 键序写回(输出稳定)
    for (k, v) in &fm.extras {
        out.push_str(&format!("\n{}: {}", k, v));
    }
    out.push_str(&format!("\n---\n\n{}", body));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_frontmatter() {
        let content = "---\nname: 测试\ndescription: 描述\ntype: concept\nupdated: 2026-06-20\ntags: [a, b]\n---\n\n正文";
        let (fm, body) = parse(content);
        assert_eq!(fm.name, "测试");
        assert_eq!(fm.doc_type, "concept");
        assert!(body.contains("正文"));
    }

    #[test]
    fn parse_no_frontmatter_returns_whole_content() {
        let content = "纯正文无 frontmatter";
        let (fm, body) = parse(content);
        assert_eq!(fm.name, "");
        assert_eq!(body, content);
    }

    #[test]
    fn parse_unclosed_frontmatter_falls_back() {
        // 以 --- 开头但无闭合,应回退为原文(不误解析)
        let content = "---\nname: 未闭合\n这是正文";
        let (fm, body) = parse(content);
        assert_eq!(fm.name, "", "未闭合不应解析字段");
        assert_eq!(body, content);
    }

    #[test]
    fn parse_collects_extras() {
        // source/aliases 等非标准字段应进 extras,不丢
        let content = "---\nname: 测试\ndescription: d\ntype: concept\nupdated: 2026-06-20\ntags: t\nsource: [[doc.md]]\naliases: 别名\n---\n\n正文";
        let (fm, _) = parse(content);
        assert_eq!(fm.extras.get("source").unwrap(), "[[doc.md]]");
        assert_eq!(fm.extras.get("aliases").unwrap(), "别名");
    }

    #[test]
    fn serialize_fills_updated_when_empty() {
        let fm = Frontmatter {
            name: "测试".into(),
            description: "d".into(),
            doc_type: "reference".into(),
            ..Default::default()
        };
        let md = serialize(&fm, "正文");
        assert!(md.starts_with("---\n"));
        assert!(md.contains("name: 测试"));
        assert!(md.contains("正文"));
        assert!(md.contains("updated: 2026-"));
    }

    #[test]
    fn serialize_preserves_updated_when_set() {
        let fm = Frontmatter {
            name: "测试".into(),
            description: "d".into(),
            doc_type: "concept".into(),
            updated: "2025-01-01".into(),
            ..Default::default()
        };
        let md = serialize(&fm, "正文");
        assert!(md.contains("updated: 2025-01-01"));
    }

    #[test]
    fn serialize_roundtrips_extras() {
        // 回应 review Important:source 等 extras 必须在 serialize 后保留
        let mut extras = std::collections::BTreeMap::new();
        extras.insert("source".into(), "[[doc.md]]".into());
        extras.insert("aliases".into(), "别名".into());
        let fm = Frontmatter {
            name: "测试".into(),
            description: "d".into(),
            doc_type: "concept".into(),
            updated: "2026-06-20".into(),
            tags: "t".into(),
            extras,
        };
        let md = serialize(&fm, "正文");
        assert!(md.contains("source: [[doc.md]]"), "source 必须保留");
        assert!(md.contains("aliases: 别名"), "aliases 必须保留");
        // 重新解析确认往返
        let (fm2, _) = parse(&md);
        assert_eq!(fm2.extras.get("source").unwrap(), "[[doc.md]]");
        assert_eq!(fm2.extras.get("aliases").unwrap(), "别名");
    }
}
