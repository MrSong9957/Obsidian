use jieba_rs::Jieba;
use std::sync::OnceLock;

static JIEBA: OnceLock<Jieba> = OnceLock::new();

fn jieba() -> &'static Jieba {
    JIEBA.get_or_init(Jieba::new)
}

/// 把输入文本用 jieba 搜索引擎模式分词,空格连接(FTS5 unicode61 tokenizer 友好)。
///
/// 用 cut_for_search 而非 cut:搜索引擎模式会把长词再细分,同时保留整词。
/// 例:"记忆系统" → "记忆 系统 记忆系统"。这样搜"记忆"和搜"记忆系统"都能命中,
/// 避免整词入库导致子串查询失效。
///
/// 用途:写入 FTS5 前对字段做预分词,使 FTS5 的 MATCH 查询能命中中文词。
pub fn tokenize_for_fts(text: &str) -> String {
    // HMM=true:启用隐马模型,提升未登录词(新词/专有名词)切分质量
    let words = jieba().cut_for_search(text, true);
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助断言:验证某个词作为独立 token 出现(而非子串)。
    /// 防止 jieba 退化为"不切词"时测试误判通过。
    fn assert_has_word(result: &str, word: &str) {
        assert!(
            result.split_whitespace().any(|w| w == word),
            "期望分词结果含独立词 '{}',实际: {:?}",
            word,
            result
        );
    }

    #[test]
    fn tokenize_chinese() {
        let result = tokenize_for_fts("记忆系统设计");
        // 若 jieba 不切词(返回原文 "记忆系统设计"),这些断言会失败,
        // 因为 split_whitespace 找不到独立的 "记忆" token
        assert_has_word(&result, "记忆");
        assert_has_word(&result, "系统");
        assert_has_word(&result, "设计");
    }

    #[test]
    fn tokenize_mixed_with_space() {
        let result = tokenize_for_fts("MCP 协议");
        assert_has_word(&result, "MCP");
        assert_has_word(&result, "协议");
    }

    #[test]
    fn tokenize_mixed_no_space() {
        // 中英文紧贴无空格(如 "MCP协议"),验证 jieba 能在 CJK/ASCII 边界切分
        let result = tokenize_for_fts("MCP协议");
        assert_has_word(&result, "MCP");
        assert_has_word(&result, "协议");
    }

    #[test]
    fn tokenize_english_passthrough() {
        let result = tokenize_for_fts("rust async tokio");
        assert_has_word(&result, "rust");
        assert_has_word(&result, "tokio");
    }

    #[test]
    fn tokenize_empty() {
        // 契约:空输入返回空串
        assert_eq!(tokenize_for_fts(""), "");
    }
}
