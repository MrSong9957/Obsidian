---
name: C 盘空间治理补充篇：用户目录、根目录与休眠处理
description: C 盘治理增量篇——用户目录 AppData 诊断与清理判读、根目录系统文件判读、休眠关闭决策与结算
type: reference
updated: 2026-11-07
tags: [windows, disk-cleanup, powershell, hibernation]
source: self
---

# C 盘空间治理补充篇：用户目录、根目录与休眠处理

**文档标准**：ASD-STE100 简化技术英语原则（中文执行版）
**版本**：1.1　**日期**：2026-11-07　**定位**：本文只写增量内容。基础流程见前篇[[wiki/Windows C 盘空间治理：手动操作与验证指导|《Windows C 盘空间治理：手动操作与验证指导》]]。
**验证环境**：Windows 11，WizTree 4.33，管理员账户 `Administrator`

---

## 0. 写作规则

同前篇第 0 节：短句；一句一步；主动语态；术语单一含义；禁用模糊词；义务分"必须/可"两级；数字用阿拉伯数字。

---

## 1. 补充术语

| 术语 | 唯一含义 |
|---|---|
| 用户目录 | `C:\Users\Administrator` |
| 配置目录 | `AppData\Local` 与 `AppData\Roaming` 的合称 |
| 模型文件 | Chrome 的 `OptGuideOnDeviceModel` 目录，端侧 AI 模型 |
| 更新残留 | 应用的 `update_downloads` 目录，升级安装包副本 |
| 用户数据 | Electron 应用的 `Default` 目录，含聊天记录与会话 |
| 休眠文件 | `C:\hiberfil.sys`，已随休眠功能关闭而删除 |
| 结算值 | 本篇全部操作完成后的 C 盘已用数值 |

---

## 2. 阶段 5：用户目录诊断（必须先诊断，后清理）

1. 执行基线命令，记录 `UsedGB`：

```powershell
Get-PSDrive C | Select-Object @{n='UsedGB';e={[math]::Round($_.Used/1GB,1)}}, @{n='FreeGB';e={[math]::Round($_.Free/1GB,1)}}
```

2. 执行目录测量命令：

```powershell
foreach ($base in @("C:\Users\Administrator\AppData\Local","C:\Users\Administrator\AppData\Roaming")) {
  "=== $base ==="
  Get-ChildItem $base -Directory -Force -ErrorAction SilentlyContinue | ForEach-Object {
    $s = (Get-ChildItem $_.FullName -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum
    [PSCustomObject]@{ GB = [math]::Round($s/1GB,2); Dir = $_.Name }
  } | Sort-Object GB -Descending | Select-Object -First 12 | Format-Table -AutoSize
}
```

3. 输出中出现 `Measure-Object` 报错时，判为正常。该报错表示个别空目录。数值缺失不影响判读。
4. 对照第 3 节的判读表，标出可删项与保留项。

---

## 3. 判读表（本机实测值，作为后续对照基线）

### 3.1 配置目录·可删项

| 路径（`AppData\Local` 下） | 实测值 | 处置 |
|---|---|---|
| `Temp` | 0.74 GB | 必须清 |
| `@deepseek-aidsh-desktop-updater` | 0.27 GB | 必须清 |
| `@zcodedesktop-updater` | 0.22 GB | 必须清 |
| `CrashDumps` | 0.06 GB | 必须清 |
| `Google\Chrome\User Data\OptGuideOnDeviceModel` | 3.98 GB | 必须清（见 5.1） |
| `Doubao\User Data\update_downloads` | 0.78 GB | 必须清（见 5.2） |
| `Doubao\User Data\gecko_cache` | 0.21 GB | 必须清（见 5.2） |

### 3.2 配置目录·保留项

| 路径 | 实测值 | 保留原因 |
|---|---|---|
| `Google\Chrome\User Data\Default` | 0.54 GB | 用户数据 |
| `Doubao\User Data\Default` | 0.31 GB | 聊天记录 |
| `Doubao\User Data\sdk_storage` | 0.02 GB | 应用状态 |
| `Doubao\User Data\sandbox_runtime` | 0.57 GB | 按用户判定保留 |
| `OpenAI` | 0.74 GB | Codex 登录态 |
| `Microsoft` | 1.08 GB | 系统组件 |
| `Packages` | 0.36 GB | UWP 数据 |
| `Roaming\Tencent` | 0.86 GB | 聊天数据 |
| `Roaming\Notion` | 0.60 GB | 用户数据 |
| `Roaming\Code` | 0.27 GB | VS Code 状态 |

### 3.3 用户目录·其他项

| 路径 | 实测值 | 处置 |
|---|---|---|
| `.cache\codex-runtimes` | 1.35 GB | 保留。删除后工具会重新下载。按用户判定不删 |
| `.cache\claude` | 0 GB | 空目录，不动 |
| `.codex` `.vscode` `.claude` `.cc-switch` 等 | 合计约 1.9 GB | 保留。工具配置与登录态 |

---

## 4. 阶段 6：清理命令（逐条执行，不得合并）

前提：先退出相关进程。Chrome、豆包、Codex、Claude Code 全部关闭。任务管理器确认无残留进程。

1. 执行第 1 批命令（4 项，约 1.3 GB）：

```powershell
Remove-Item "C:\Users\Administrator\AppData\Local\Temp\*" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Users\Administrator\AppData\Local\@deepseek-aidsh-desktop-updater\*" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Users\Administrator\AppData\Local\@zcodedesktop-updater\*" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Users\Administrator\AppData\Local\CrashDumps\*" -Recurse -Force -ErrorAction SilentlyContinue
```

2. 执行第 2 批命令（Chrome 模型文件，约 3.98 GB）：

```powershell
Remove-Item "C:\Users\Administrator\AppData\Local\Google\Chrome\User Data\OptGuideOnDeviceModel\*" -Recurse -Force -ErrorAction SilentlyContinue
```

3. 执行第 3 批命令（豆包两项，约 1 GB）：

```powershell
Remove-Item "C:\Users\Administrator\AppData\Local\Doubao\User Data\update_downloads\*" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Users\Administrator\AppData\Local\Doubao\User Data\gecko_cache\*" -Recurse -Force -ErrorAction SilentlyContinue
```

4. 删除 `Temp` 时出现"文件正在使用"提示时，跳过该项。该提示不影响结果。
5. 禁止执行下列命令：
   - 禁止删除 `Doubao\User Data\Default`
   - 禁止删除 `Doubao\User Data\sandbox_runtime`
   - 禁止删除 `.cache` 目录整体

---

## 5. 各清理项的再生行为

1. Chrome 模型文件：下次打开 Chrome 时可能重新下载约 4 GB。属正常。
2. 豆包 `update_downloads`：下次升级时重新生成。
3. 豆包 `gecko_cache`：下次使用时重新生成。
4. `Temp`：随使用重新增长。
5. 再生不判为失败。再生后的处理方法：达到判读表数值时，重复执行第 4 节命令。

---

## 6. 阶段 7：根目录文件判读

1. 执行根目录测量命令：

```powershell
Get-ChildItem C:\ -Force -File | Where-Object {$_.Length -gt 100MB} |
  Select-Object Name, @{n='GB';e={[math]::Round($_.Length/1GB,2)}} | Sort-Object GB -Descending
```

2. 对照下表：

| 文件 | 判定 |
|---|---|
| `hiberfil.sys` 6.4 GB | 可删。处置见第 7 节 |
| `pagefile.sys` 2.9 GB | 保留。虚拟内存。禁止迁移，禁止手删 |
| `$MFT` 340.8 MB | 保留。NTFS 主文件表。禁止手删 |
| `$LogFile` 64 MB | 保留。NTFS 日志。禁止手删 |
| `swapfile.sys` 16 MB | 保留。禁止手删 |
| 其他 `$` 开头文件 | 保留。全部为 NTFS 元数据。禁止手删 |
| `Output.txt` `bootTel.dat` | 忽略。小于 200 字节 |

3. `$MFT` 随磁盘使用量增长。340 MB 属正常值。禁止压缩，禁止清理。

---

## 7. 阶段 8：休眠决策与执行

1. 先判断设备类型与使用方式，对照决策表：

| 条件 | 决策 |
|---|---|
| 台式机，每天关机 | 关闭休眠。执行第 2 步 |
| 笔记本，经常合盖带走 | 保留休眠。不执行任何命令 |
| 想保留快速启动，想省一半空间 | 执行第 3 步 |

2. 关闭休眠，执行第 1 条命令。回收 6.4 GB：

```powershell
powercfg /h off
```

3. 体积折半方案，执行第 1 条命令。保留快速启动，失去休眠功能：

```powershell
powercfg /h /type reduced
```

4. 关闭休眠的副作用：快速启动同时关闭。下次开机慢 2~5 秒。该副作用不影响任何功能。
5. 验证休眠文件已删除：

```powershell
Test-Path C:\hiberfil.sys
```

6. 通过标准：输出 `False`。

---

## 8. 最终验证与结算

1. 执行基线命令（同第 2 节第 1 步）。
2. 记录 `UsedGB` 与 `FreeGB`。
3. 结算表（本次实测）：

| 时点 | 已用 | 本步回收 |
|---|---|---|
| 起点 | 101.9 GB | — |
| 前篇：传递优化缓存 + 更新缓存 | 75.2 GB | 26.7 GB |
| 本篇：第 4 批第 1、2 步 | 70.9 GB | 4.3 GB |
| 本篇：第 4 批第 3 步 | 69.9 GB | 1.0 GB |
| 本篇：关休眠 | 63.5 GB | 6.4 GB |
| **合计** | | **38.4 GB** |

4. 通过标准：最终 `UsedGB` ≈ 63.5，`FreeGB` ≈ 174.7。
5. 差值超过 ±1 GB 时，重跑第 3 节测量命令，定位差值来源。

---

## 9. 本篇禁止操作汇总

1. 禁止删除 `Doubao\User Data\Default`（聊天记录）。
2. 禁止删除 `Doubao\User Data\sdk_storage`。
3. 禁止删除 `.cache\codex-runtimes`（按用户判定保留）。
4. 禁止删除根目录 `$` 开头的任何文件。
5. 禁止迁移 `pagefile.sys`。
6. 禁止在休眠决策为"保留"的机器上执行 `powercfg /h off`。

---

## 10. 记录表（本篇增量填写）

| 日期 | 基线 | 本篇操作 | 结算值 | 验证结果 |
|---|---|---|---|---|
| 2026-11-07 | 70.9 GB | 豆包两项 + 关休眠 | 63.5 GB | 通过 |
| | | | | |

---

## 11. 维护周期（本篇增量）

1. 每月检查 `AppData\Local\Temp`，超过 1 GB 时执行第 4 节第 1 步。
2. Chrome 模型文件重新达到 3 GB 以上时，执行第 4 节第 2 步。
3. 每季度跑一次第 2 节测量命令，与第 3 节判读表对照。
4. 数值超过基线 2 GB 以上时，判为异常，按前篇阶段 1 流程重新诊断。

---

**本篇效果**：回收 7.4 GB（101.9 → 63.5 中的本篇份额 5.3 GB + 前篇已结算部分的衔接确认）。总账与前篇合并后为 38.4 GB。休眠关闭后，`hiberfil.sys` 不再存在，C 盘根目录无增长项。