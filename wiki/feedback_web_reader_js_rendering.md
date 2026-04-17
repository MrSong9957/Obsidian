---
name: web-reader 对 JS 渲染页面可能遗漏内容
description: Web reader 工具不执行 JS，Next.js 等 CSR 页面的客户端渲染内容会被遗漏，用 curl 全文搜索更可靠
type: feedback
originSessionId: 4f80973e-007a-4eb7-b13e-fd6c0fed9227
status: deprecated
---
> 内容已迁移至 [[wiki/network-api.md|network-api]]。

Web reader / webReader 工具对 JS 渲染页面（Next.js、SPA）可能遗漏内容。

**Why:** 这些工具只解析服务端返回的初始 HTML，不执行 JavaScript。Next.js 页面的部分内容在客户端组件中渲染，嵌在 `<script>` 标签的 RSC payload 里，reader 解析 DOM 时会跳过这些 `<script>` 块。实测 learn.shareai.run 的 s06 页面，"最小心智模型"段落两次 web reader 都没拿到，curl + grep 才找到。

**How to apply:** 当用户说"页面上有但你没找到"时，不要重复用 web reader，直接 `curl -sL URL | grep 关键词` 对原始 HTML 做文本匹配。怀疑内容被遗漏时也优先用 curl。
