---
name: example-hydration-mismatch
framework: next.js
version: "14.x / 15.x"
keywords: [hydration, mismatch, 水合, SSR, 客户端, server, client, 不一致, hydration failed]
created: 2026-04-08
updated: 2026-04-08
---

## 问题
Next.js 应用中，组件在服务端渲染的 HTML 与客户端 hydration 时的输出不一致，导致 hydration mismatch 错误。

## 错误表现
浏览器控制台报错：
```
Hydration failed because the initial UI does not match what was rendered on the server.
```
或
```
Text content does not match server-rendered HTML.
```

## 根因
组件在服务端和客户端渲染时产生了不同的输出。常见原因：
- 使用了 `Date.now()`、`Math.random()` 等非确定性函数
- 依赖 `window`、`localStorage` 等浏览器 API（服务端不存在）
- 条件渲染基于 `typeof window !== 'undefined'`

## 解决方案
将浏览器专属逻辑包裹在 `useEffect` 或 `dynamic import` 中，确保服务端和客户端首次渲染一致：

```tsx
// 方案 1: dynamic import + ssr: false
import dynamic from 'next/dynamic'
const ClientOnly = dynamic(() => import('./ClientOnly'), { ssr: false })

// 方案 2: useEffect + isMounted 状态
const [isMounted, setIsMounted] = useState(false)
useEffect(() => setIsMounted(true), [])
if (!isMounted) return null
```

## 失败路径
- 尝试 1: 直接在组件中使用 `new Date()` → 服务端和客户端时间戳不同导致 mismatch
- 尝试 2: 用 `suppressHydrationWarning` 属性 → 只是隐藏警告，实际问题未解决，子组件仍会报错
- 尝试 3: 强制 `window` 检查 `typeof window !== 'undefined'` → SSR 时走 else 分支，客户端走 if 分支，HTML 结构不同

## 适用条件
- Next.js 14/15 App Router
- 任何依赖浏览器 API 的组件
- 使用 `Date`、`Math.random()`、`localStorage`、`sessionStorage` 的场景
