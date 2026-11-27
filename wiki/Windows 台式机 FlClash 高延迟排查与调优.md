---
name: Windows 台式机 FlClash 高延迟排查与调优
description: 台式机使用 FlClash 延迟偏高（400 ms 起）的分层排查命令、界面覆写路径与低延迟配置方案
type: guide
updated: 2026-11-22
tags: [windows, networking, flclash, clash, vpn, ipv6, troubleshooting]
source: self
---

# Windows 台式机 FlClash 高延迟排查与调优

台式机开启 `FlClash` 后延迟长期显示 `400 ms` 起，且高于同局域网 Wi-Fi 设备（约 `100 ms`）时，通常不是网线或路由器故障，而是**节点链路、测速统计口径、双模式叠加与 IPv6 配置**共同导致。

---

## 1. 分层排查步骤（先硬件，后代理）

按从底层到应用层的顺序执行只读检查，每次只验证一个环节：

1. **检查物理网卡速率与丢包**：
   ```powershell
   Get-NetAdapter | Format-Table Name, Status, LinkSpeed, FullDuplex
   Get-NetAdapterStatistics -Name "以太网" | Select-Object OutboundPacketErrors, OutboundDiscardedPackets, ReceivedPacketErrors, ReceivedDiscardedPackets
   ```
   - 正常标准：千兆网口为 `1 Gbps` 全双工，收发错误与丢包均为 `0`。

2. **检查局域网与宽带直连延迟**：
   ```powershell
   ping -n 10 192.168.1.1
   ping -n 6 223.5.5.5
   ```
   - 正常标准：网关平均 `1 ms`（`0%` 丢包），国内公共 DNS IP 平均 `20–40 ms`（`0%` 丢包）。

3. **排查本机 DNS 阻塞**：
   ```powershell
   $null = Resolve-DnsName localhost
   Measure-Command { Resolve-DnsName www.taobao.com } | Select-Object TotalMilliseconds
   ```
   - 提示：PowerShell 首次加载 `DnsClient` 模块约耗时 `250 ms`，需先用 `localhost` 预热再测。命中 Fake-IP（`198.18.0.x`）时正常耗时应低于 `5 ms`。

4. **分解代理连接各阶段耗时**：
   ```powershell
   curl.exe -s -o NUL -w "HTTP: dns=%{time_namelookup} conn=%{time_connect} tls=%{time_appconnect} ttfb=%{time_starttransfer} total=%{time_total}\n" -x http://127.0.0.1:7890 https://www.gstatic.com/generate_204
   curl.exe -s -o NUL -w "TUN:  dns=%{time_namelookup} conn=%{time_connect} tls=%{time_appconnect} ttfb=%{time_starttransfer} total=%{time_total}\n" --noproxy "*" https://www.gstatic.com/generate_204
   ```
   - 判读方法：
     - `ttfb - tls` 反映握手后的单次 HTTP 往返延迟（对应开启 `unified-delay` 后的面板延迟）。
     - 若 `TUN` 明显慢于 `127.0.0.1:7890` 或频繁出现 `800 ms` 以上毛刺，说明 TUN 栈或双模式叠加存在额外开销。

---

## 2. 常见原因与 `FlClash` 配置路径

### 2.1 原因拆解

| 原因 | 表现特征 | 解决动作 |
|---|---|---|
| 选中高延迟直连节点（如 `VMess + WebSocket`） | `127.0.0.1:7890` 冷连接即达 `360–400 ms`，`tls` 占 `270–300 ms` | 改用低延迟中转节点 |
| `unified-delay` 未在运行配置生效 | 面板测速将 TCP、WebSocket、TLS 握手全部算入，显示 `400 ms` 起 | 在内核覆写中显式开启 `统一延迟` |
| `ipv6` 在运行配置仍为 `true` | 虚拟网卡 IPv6 DNS 查询 `A` 记录偏慢（实测 `122.4 ms`） | 在内核覆写中显式添加 `IPv6` 并关闭 |
| 系统代理与 TUN 双开，且 TUN 栈为 `mips` | `TUN` 请求偶发 `800–1415 ms` 毛刺 | 仅保留其一；开 TUN 时将栈改为 `system` |

### 2.2 `FlClash` 界面配置路径

1. **开启内核覆写项**：
   `工具` → `常规` → `内核` → `添加覆写项` → 添加以下三项：
   - `IPv6`：保持**关闭**（`ipv6: false`）。
   - `统一延迟`：设为**开启**（`unified-delay: true`）。
   - `TCP并发`：设为**开启**（`tcp-concurrent: true`）。
2. **避免双模式叠加与调整 TUN 栈**：
   - 在网络设置中关闭「系统代理」与「虚拟网卡（TUN）」的重复开启，日常建议只开其中一种。
   - 若使用虚拟网卡（TUN），将 TUN 栈从 `mips` 调整为 `system`（`tun.stack: "system"`）。
3. **切换低延迟节点**：
   - 关闭 IPv6 前，必须先将代理组从纯 IPv6 节点切换为 IPv4 中转节点，否则纯 IPv6 节点会无法连通。

---

## 3. 配置文件核对路径

`FlClash` 桌面端运行配置与数据位于 `%APPDATA%\com.follow\clash\`：

- `config.yaml`：核对内核实际生效的 `unified-delay`、`ipv6`、`tcp-concurrent`、`tun.stack`。
- `shared_preferences.json`：核对 `networkProps.systemProxy` 与 `vpnProps.enable`。
- `database.sqlite`（表 `profiles` 列 `selected_map`）：核对当前选中的代理节点。

---

## 4. 实例记录（2026-11-22，Windows 11 台式机）

### 4.1 硬件与直连基线

- 网卡：`Realtek PCIe GbE Family Controller`，`1 Gbps` 全双工，收发错误与丢包均为 `0`。
- 网关 `192.168.1.1`：`1 ms`（`0%` 丢包）；`223.5.5.5`：`30 ms`（`0%` 丢包）；百度 `183.2.172.177`：`33 ms`（`0%` 丢包）。

### 4.2 修改前后对照

| 配置项 | 修改前 | 修改后 |
|---|---|---|
| `unified-delay` | `false` | `true` |
| `ipv6` | `true` | `false` |
| `tcp-concurrent` | `false` | `true` |
| 系统代理（`networkProps.systemProxy`） | `true` | `false` |
| 虚拟网卡 TUN（`vpnProps.enable`） | `true` | `true` |
| TUN 栈（`tun.stack`） | `mips` | `system` |
| `🔰国外流量` 节点 | `日本-IPV6直连` | `日本A3`（`216.23.123.211`） |

### 4.3 效果验证

- `FlClash` 面板延迟：从 `400 ms` 起降至约 `100 ms` 区间。
- 节点基础延迟：`日本-IPV6直连` 20 次 Ping 平均 `96 ms`（`84–132 ms` 波动）；切换为 `日本A3` 后 Ping 平均 `86 ms`（`85–88 ms`，`0%` 丢包）。
- `curl` 分段测速（`https://www.gstatic.com/generate_204`）：握手后单次往返（`ttfb - tls`）为 `135–152 ms`；冷连接完整耗时为 `551–1274 ms`（`tls` 占 `399–954 ms`）。
- 低延迟节点参考：同订阅下 `香港A2`（`45.125.33.15`）20 次 Ping 平均 `53 ms`（`0%` 丢包），`香港A3>HGC原生`、`日本A2`、`新加坡A1` 入口（`39.109.56.160`）Ping 平均 `56 ms`（`0%` 丢包），需要更快的网页冷打开速度时可优先选用。
