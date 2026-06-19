use jieba_rs::Jieba;
use std::sync::OnceLock;

static JIEBA: OnceLock<Jieba> = OnceLock::new();

fn jieba() -> &'static Jieba {
    JIEBA.get_or_init(Jieba::new)
}

/// 把输入文本用 jieba 分词,空格连接(FTS5 unicode61 tokenizer 友好)。
/// 中文 "记忆系统" → "记忆 系统";英文按原样保留(已被空格分隔)。
///
/// 用途:写入 FTS5 前对字段做预分词,使 FTS5 的 MATCH 查询能命中中文词。
pub fn tokenize_for_fts(text: &str) -> String {
    let words = jieba().cut(text, true);
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_chinese() {
        let result = tokenize_for_fts("记忆系统设计");
        assert!(result.contains("记忆"));
        assert!(result.contains("系统"));
        assert!(result.contains("设计"));
    }

    #[test]
    fn tokenize_mixed() {
        let result = tokenize_for_fts("MCP 协议");
        assert!(result.contains("MCP"));
        assert!(result.contains("协议"));
    }

    #[test]
    fn tokenize_english_passthrough() {
        let result = tokenize_for_fts("rust async tokio");
        assert!(result.contains("rust"));
        assert!(result.contains("tokio"));
    }
}
