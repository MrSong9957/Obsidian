//! path policy:三层目录权限矩阵。
//!
//! 对应 AGENTS.md 的权限矩阵:
//! - sources/ :读 ✅ 写 ✅(append 优先) 删 ❌
//! - wiki/    :读 ✅ 写 ✅ 删(仅 deprecated 标记)
//! - templates/:读 ✅ 写 ❌ 删 ❌
//!
//! 另做路径穿越防御(含 .. 或绝对路径)。

use crate::error::WikiError;
use std::path::{Component, Path};

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
///
/// 路径穿越防御用 `Path::components()` 而非字符串 split,
/// 因为 Windows 上 `\` 和 `/` 都是分隔符,纯字符串 `split('/')`
/// 会漏掉 `wiki/sub\..\..\..` 这类混合斜杠穿越。
pub fn check(path: &str, op: Op) -> Result<(), WikiError> {
    // 路径穿越防御:用 components() 解析,任何 ParentDir/RootDir/Prefix 都拒绝。
    // 这覆盖了正斜杠、反斜杠、混合斜杠、绝对路径(含 Windows 盘符 C:\)。
    let p = Path::new(path);
    for comp in p.components() {
        match comp {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(WikiError::PathTraversal(path.into()));
            }
            _ => {}
        }
    }

    // 规范化为正斜杠前缀再判断,兼容 Windows 反斜杠输入
    let normalized = path.replace('\\', "/");
    let is_sources = normalized.starts_with("sources/");
    let is_wiki = normalized.starts_with("wiki/");
    let is_templates = normalized.starts_with("templates/");

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
                // sources / wiki 允许写(v5 后写入归 agent 文件工具,policy 只管 sync 时路径校验)
                Ok(())
            }
        }
        Op::Delete => {
            if is_sources {
                Err(WikiError::PolicyViolation("sources 禁止删除".into()))
            } else if is_wiki {
                // wiki 删除实际是 deprecated 标记,允许(实际 deprecate 逻辑在 core::delete)
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

    #[test]
    fn blocks_mixed_slash_traversal() {
        // review Critical:Windows 混合斜杠穿越,split('/') 会漏掉
        // wiki/sub\..\..\..\etc —— components() 能正确识别
        assert!(check("wiki/sub\\..\\..\\..\\etc", Op::Read).is_err());
        assert!(check("wiki\\..\\..\\etc", Op::Read).is_err());
    }

    #[test]
    fn blocks_windows_drive_prefix() {
        // Windows 绝对路径 C:\... 应被 Prefix 组件拒绝
        assert!(check("C:\\Windows\\system32", Op::Read).is_err());
    }

    #[test]
    fn allows_backslash_paths_inside_vault() {
        // 合法:Windows 反斜杠路径在 vault 目录内应放行
        assert!(check("wiki\\sub\\x.md", Op::Read).is_ok());
        assert!(check("sources\\dev\\x.md", Op::Write).is_ok());
    }

    #[test]
    fn blocks_prefix_lookalike() {
        // sources-evil/ 不应匹配 sources/(尾部斜杠是字面量)
        assert!(check("sources-evil/x.md", Op::Read).is_err());
        assert!(check("wikix.md", Op::Read).is_err());
    }
}
