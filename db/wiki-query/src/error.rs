//! 统一错误类型。供 core/policy/frontmatter/write 等模块共用。
//!
//! 设计:用一个 enum 覆盖所有业务错误,From impl 桥接 io::Error / rusqlite::Error,
//! 让 ? 运算符在调用链自动转换。

use std::fmt;

#[derive(Debug)]
pub enum WikiError {
    /// frontmatter 校验失败(字段缺失/非法)
    InvalidFrontmatter(String),
    /// 路径穿越拒绝(含 .. 或绝对路径)
    PathTraversal(String),
    /// 权限策略违反(sources 禁删 / templates 只读 等)
    PolicyViolation(String),
    /// IO 错误
    Io(String),
    /// 数据库错误
    Db(String),
}

impl fmt::Display for WikiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WikiError::InvalidFrontmatter(msg) => write!(f, "frontmatter 校验失败: {}", msg),
            WikiError::PathTraversal(p) => write!(f, "路径穿越拒绝: {}", p),
            WikiError::PolicyViolation(msg) => write!(f, "权限策略: {}", msg),
            WikiError::Io(msg) => write!(f, "IO 错误: {}", msg),
            WikiError::Db(msg) => write!(f, "数据库错误: {}", msg),
        }
    }
}

impl std::error::Error for WikiError {}

impl From<std::io::Error> for WikiError {
    fn from(e: std::io::Error) -> Self {
        WikiError::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for WikiError {
    fn from(e: rusqlite::Error) -> Self {
        WikiError::Db(e.to_string())
    }
}
