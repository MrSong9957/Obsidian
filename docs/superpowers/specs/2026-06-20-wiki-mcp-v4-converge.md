# wiki-query MCP 收敛与增强设计(v4)

## 概述

基于 v3 实现后的方案对比与复盘,对 wiki-query MCP 做 5 项调整:收敛到纯 MCP、补 section patch、加 ServerInfo 说明、明确记忆提取工作流。Obsidian 生态对接推迟。

**基线**:`docs/superpowers/specs/2026-06-19-wiki-memory-system-design.md`(v3,已实现)。

**本轮不做**(守住治理定位,避免重造轮子):文件 watcher、向量检索、知识图谱、Obsidian REST API 依赖。

## 已确认决策

| 点 | 决策 | 性质 |
|----|------|------|
| 1. 只保留 MCP | 删 search/show CLI,sync(运维)+ serve(MCP)保留 | 收敛 |
| 2. section patch | 按 Markdown heading 切分替换(抄 cyanheads 思路,不抄 REST 依赖) | 新功能 |
| 3. ServerInfo 说明 | 手写 get_info 告诉 agent 仓库模型 | 增强 |
| 4. Obsidian 生态 | 推迟 | — |
| 5. 记忆提取 | 形态 B:agent 提炼后调 wiki_write,纯工作流文档,零代码 | 文档 |

## 改动详情

### 1. 成品只保留 MCP(收敛)

**删除**:
- `main.rs` 的 `search` / `show` CLI dispatch 分支
- 命令行不再支持 `wiki-query search <q>` / `wiki-query show <path>`

**保留**:
- `wiki-query sync`(运维,git post-commit hook 用)
- `wiki-query serve`(MCP server)
- `db::search` / `db::show` 内部函数(core 复用,无 CLI 入口)

**理由**:查询走 agent(MCP),人不用手敲命令。sync 是运维操作不归 agent 管。

**影响**:`db.rs` 的 `show` 函数当前是 CLI 专用(直接 println),删 CLI 后它无调用方。评估:保留为 core::read 的底层或删除。倾向于:show 逻辑并入 core::read(已存在且更强),删 db::show。

### 2. section patch(吸取点 1)

**core::patch(vault_root, path, heading, new_content)**:
1. policy::check(path, Write)
2. 读文件,parse frontmatter + body
3. 在 body 里定位 heading 行(`#`~`######`,CommonMark 要求 `#` 后跟空格),到下一个同级或更高级 heading 之间的内容为 section
4. heading 不存在 → 返回 `WikiError::NotFound`(业务错误)
5. 替换该 section 内容为 new_content,保留 heading 行本身
6. 重新 serialize(frontmatter + 改后的 body)
7. 写回文件 + 即时 sync_file_by_path

**heading 定位规则**(简化,避免复杂 AST):
- 支持 `#`(h1)~ `######`(h6)全级别(CommonMark:`#` 后须跟空格才算 heading)
- section 范围:从目标 heading 行下一行,到下一个同级或更高级 heading 行前
- 例:`## 配置` 的 section 是它下面到下一个 `##` 或 `#` 之前的内容
- 只匹配第一个同名 heading(文档内 heading 应唯一,否则提示)

**MCP wiki_patch 工具**:
- 参数:path、heading、new_content
- description 明确:按 heading 局部替换,避免整篇重写丢失未改部分;heading 不存在返回错误

**不做**:多 heading 批量替换、跨级 heading 嵌套编辑(MVP 单 heading 替换够用)。

### 3. ServerInfo 说明(吸取点 2)

脱离 `#[tool_router(server_handler)]` 的自动 ServerHandler 实现,手写:

```rust
impl ServerHandler for WikiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Obsidian 知识库 MCP。三层结构:\
                sources/(原始资料,只读为主)/ wiki/(结构化知识)/ templates/(只读模板)。\
                写入走四态决策(ADD/UPDATE/NOOP/CONFLICT),自动查重。\
                查知识用 wiki_search,读全文 wiki_read,记录 wiki_write,\
                列举 wiki_list,局部编辑 wiki_patch。".into()),
            server_info: Implementation { name: "wiki-mcp".into(), version: ... },
            ..Default::default()
        }
    }
}
```

tool_router 宏改为只生成 tool 路由部分,ServerHandler 手写。参考 rmcp counter 示例的混合模式。

### 4. 记忆提取工作流(文档,第 5 点)

README 新增章节,纯工作流约定,零代码:

**触发**:用户主动说"把当前项目记忆提炼到知识库"。

**流程**(agent 执行):
1. agent 读自己的项目记忆(agent 知道存储位置)
2. agent 用 LLM 挑出长期有价值内容(技术决策/配置方法/踩坑)
3. agent 调 wiki_write 写入,frontmatter 带 source 溯源

**分工原则**(写入 AGENTS.md):
- 瞬时偏好/会话进度 → agent 自带记忆(临时)
- 长期技术知识 → wiki-query(永久 + 溯源)
- 提炼是手动触发,不自动(避免噪声)

**为什么零代码**:形态 B(agent 提炼 + wiki_write 接收)下,wiki-query 不需要新工具。agent 已有的 wiki_write 足够接收提炼结果。提炼的智能在 agent(LLM),不在 wiki-query(Rust)。

## 不在本轮范围

- 文件 watcher(重造轮子,obsidian-brain 已有)
- 向量语义检索(场景是中文关键词 + 治理,非语义)
- 知识图谱(wikilink 关系查询,nice-to-have)
- Obsidian REST API / Local REST 插件依赖(违背"独立工具"定位)
- wiki_delete / 矛盾页工作流(沿用 v3 推迟决定)

## 测试

- section patch:heading 存在/不存在/同级边界/h2 与 h3 混合(单元 + 集成)
- ServerInfo:initialize 响应含 instructions(MCP 集成测试断言)
- 记忆提取:无代码测试,文档评审即可

## 复杂度

| 工作 | 估计 |
|------|------|
| 删 CLI search/show + 清理 db::show | 1h |
| core::patch + wiki_patch 工具 + 测试 | 3h |
| ServerInfo instructions + rmcp 混合模式 | 1.5h |
| README 记忆提取章节 + AGENTS.md 分工 | 1h |
| **总计** | **~6.5h** |
