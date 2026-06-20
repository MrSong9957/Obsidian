# wiki-query 记忆提炼规则(注入到项目根)

本文件由 wiki-query MCP 的 `wiki_get_project_rules` 工具读取,内容会注入到你当前项目根的 `CLAUDE.md` / `AGENTS.md`,让你在该项目里知道如何使用 wiki-query 沉淀记忆。

## vault 信息

- **vault 根**:见你的 MCP 配置(`WIKI_VAULT_ROOT` 环境变量或启动 cwd)
- **三层结构**:`sources/`(原始资料,可写禁删)/ `wiki/`(结构化知识,可写)/ `templates/`(只读模板)
- **权威源**:Markdown 文件。SQLite 是派生索引,可随时 `rm + sync` 重建。

## frontmatter 约定

你写入 vault 的文档,frontmatter 应有:
- `name`(必有):文档标题,检索展示用
- `description`(必有):一句话说明,检索展示用
- `type`(建议):如 `concept` / `reference` / `summary` / `guide` 等
- `tags`(建议):逗号分隔的标签

缺 `name`/`description` 的文档,检索时元数据不全(不致命,但影响展示)。`type` 不强制白名单。

## 全量提炼工作流(6 步)

当用户说"提炼一下我的记忆"时,按此流程:

1. **读自己的记忆系统**(你自己的文件工具):全量读取你的项目记忆
   - Claude Code:`~/.claude/<项目>/MEMORY.md`(索引页)+ 它链接的关联 `.md`(具体内容)
   - Codex:`~/.codex/memories/*.md`
   - Cursor:`.cursor/rules/*.mdc`
   - OpenCode:`.opencode/memory/*`(格式依插件而定)
2. **逐篇查重**(`wiki_search`):对每篇记忆内容,调 `wiki_search` 用关键词查,看 vault 里有没有相似的
   - 相似:用文件工具读 vault 文档全文 → 决定合并/追加
   - 无相似:用文件工具新建 vault 文档
3. **写 vault**(你自己的文件工具):用文件工具把提炼后的内容写入 vault(frontmatter 按上述约定)
4. **同步索引**(`wiki_sync`):调 `wiki_sync()` 让 SQLite 索引跟上 + 拿查重报告
   - 看报告,若有意外重复,用文件工具处理
5. **删源文件**(你自己的文件工具):删除自己记忆系统里已提炼的源文件(避免下次重复提炼)
6. **告知用户**:报告"已提炼 N 篇,删源 M 个"

## 查重是固定流程

- **主防线(第 2 步)**:每篇记忆都先 `wiki_search`,自己判断要不要新建。智能在你的 LLM。
- **兜底(第 4 步)**:`wiki_sync` 的查重报告会提示相似对。你若第 2 步漏了,sync 还能兜住。
- **合并归你**:sync 报告只提示相似,合不合并、怎么合并是你的智能决策。wiki-query 不自动合并。

## 提炼后删源

记忆会不断增加,不删已提炼的源文档,后面每次提炼都要重新处理旧内容,越来越重。删源让"提炼"始终是对增量记忆的处理。

## 用户说"去重一下"时

调 `wiki_sync(scope="full")`,拿全库相似对报告,用文件工具处理重复(合并/删除)。
