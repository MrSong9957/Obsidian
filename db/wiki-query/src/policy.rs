//! path policy:三层目录权限矩阵。
//!
//! 对应 AGENTS.md 的权限矩阵:
//! - sources/ :读 ✅ 写 ✅(append 优先) 删 ❌
//! - wiki/    :读 ✅ 写 ✅ 删(仅 deprecated 标记)
//! - templates/:读 ✅ 写 ❌ 删 ❌
//!
//! 另做路径穿越防御(含 .. 或绝对路径)。

use crate::error::WikiError;

/// 操作类型。对应 documents 的读/写/删。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Read,
    Write,
    Delete,
}

/// 检查 path 相对路径的操作权限。
///
/// path 须为相对路径,如 "wiki/agent.md"、"sources/dev/rust.md"。
/// 必须在 sources/ wiki/ templates/ 三个顶层目录之一内。
/// 含 ".." 段或以 "/" 开头 → 路径穿越,拒绝。
pub fn check(path: &str, op: Op) -> Result<(), WikiError> {
    // 路径穿越防御:任一段为 ".." 或整体以 "/" 开头
    for segment in path.split('/') {
        if segment == ".." {
            return Err(WikiError::PathTraversal(path.into()));
        }
    }
    if path.starts_with('/') {
        return Err(WikiError::PathTraversal(path.into()));
    }

    let is_sources = path.starts_with("sources/");
    let is_wiki = path.starts_with("wiki/");
    let is_templates = path.starts_with("templates/");

    if !is_sources && !is_wiki && !is_templates {
        return Err(WikiError::PolicyViolation(format!(
            "路径必须在 sources/ wiki/ templates/ 下: {}",
            path
        )));
    }

    match op {
        Op::Read => Ok(()),
        Op::Write => {
            if is_templates {
                Err(WikiError::PolicyViolation("templates 只读,禁止写入".into()))
            } else {
                // sources / wiki 允许写(sources 的 append 语义由 core::write 体现)
                Ok(())
            }
        }
        Op::Delete => {
            if is_sources {
                Err(WikiError::PolicyViolation("sources 禁止删除".into()))
            } else if is_wiki {
                // wiki 删除实际是 deprecated 标记,允许
                Ok(())
            } else {
                Err(WikiError::PolicyViolation("templates 禁止删除".into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_read_everywhere() {
        assert!(check("wiki/x.md", Op::Read).is_ok());
        assert!(check("sources/x.md", Op::Read).is_ok());
        assert!(check("templates/x.md", Op::Read).is_ok());
    }

    #[test]
    fn allows_write_sources_and_wiki() {
        assert!(check("wiki/x.md", Op::Write).is_ok());
        assert!(check("sources/dev/x.md", Op::Write).is_ok());
    }

    #[test]
    fn blocks_write_templates() {
        assert!(check("templates/x.md", Op::Write).is_err());
    }

    #[test]
    fn blocks_delete_sources() {
        assert!(check("sources/x.md", Op::Delete).is_err());
    }

    #[test]
    fn allows_delete_wiki_as_deprecated() {
        // wiki 删除语义是 deprecated 标记,policy 放行(实际处理在 core)
        assert!(check("wiki/x.md", Op::Delete).is_ok());
    }

    #[test]
    fn blocks_delete_templates() {
        assert!(check("templates/x.md", Op::Delete).is_err());
    }

    #[test]
    fn blocks_path_traversal_dotdot() {
        assert!(check("../etc/passwd", Op::Read).is_err());
        assert!(check("wiki/../../etc", Op::Read).is_err());
    }

    #[test]
    fn blocks_absolute_path() {
        assert!(check("/etc/passwd", Op::Read).is_err());
        assert!(check("/absolute/path", Op::Read).is_err());
    }

    #[test]
    fn blocks_outside_vault_dirs() {
        assert!(check("random/x.md", Op::Read).is_err());
        assert!(check("docs/x.md", Op::Read).is_err());
    }
}
