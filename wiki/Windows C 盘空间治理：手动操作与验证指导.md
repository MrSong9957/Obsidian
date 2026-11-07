# Windows C 盘空间治理：手动操作与验证指导

**文档标准**：ASD-STE100 简化技术英语原则（中文执行版）
**版本**：1.0　**日期**：2026-11-07　**验证环境**：Windows 11，WizTree 4.33

---

## 0. 本文档遵循的写作规则

1. 每句不超过 20 个词。步骤句保持短句。
2. 每句只含一个指令。禁止用"然后""并且"连接两个动作。
3. 使用主动语态、现在时、祈使句。
4. 每个术语只有一个含义。术语见第 1 节。
5. 不使用模糊词（"一些""大概""可能地"）。数值、路径、阈值全部给出。
6. 义务分两级：**必须** = 不做则操作失败；**可** = 可选操作。
7. 数字使用阿拉伯数字。
8. 问句只用于提问，不用于陈述。

---

## 1. 术语表

| 术语 | 唯一含义 |
|---|---|
| C 盘 | Windows 系统盘，盘符 `C:` |
| 基线 | 操作开始前记录的磁盘占用数值 |
| 扫描工具 | WizTree 4.x，用于生成目录大小清单 |
| 传递优化缓存 | 路径 `C:\Windows\ServiceProfiles\NetworkService\AppData\Local\Microsoft\Windows\DeliveryOptimization\Cache` |
| 服务转储 | 路径 `C:\Windows\ServiceProfiles\LocalService\AppData\Local\Temp` 下的 `.dmp` 文件 |
| 更新缓存 | 路径 `C:\Windows\SoftwareDistribution\Download` |
| 组件存储 | 目录 `C:\Windows\WinSxS` |
| 还原点 | 卷影副本，存储在 `C:\System Volume Information` |
| 休眠文件 | 文件 `C:\hiberfil.sys` |
| 策略键 | 注册表路径 `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization` |
| P2P | 电脑之间互传更新数据的机制，英文名传递优化 |

---

## 2. 前提

1. 必须具备管理员权限的 PowerShell。
2. 必须安装扫描工具 WizTree。
3. 操作前必须执行以下命令，记录基线：

```powershell
Get-PSDrive C | Select-Object @{n='UsedGB';e={[math]::Round($_.Used/1GB,1)}}, @{n='FreeGB';e={[math]::Round($_.Free/1GB,1)}}
```

4. 把基线数值写入第 8 节的记录表。

---

## 3. 阶段 1：诊断

1. 打开扫描工具，扫描 C 盘。
2. 记录根目录各项数值。
3. 展开目录 `C:\Windows`。
4. 对照下表判断异常项。

| 检查项 | 正常范围 | 超出范围的处置 |
|---|---|---|
| `WinSxS` | 8 ~ 15 GB | 不手删。仅用 DISM（见第 6 节） |
| `ServiceProfiles` | < 1 GB | 执行阶段 2.1 |
| `SoftwareDistribution` | < 3 GB | 执行阶段 2.2 |
| `System Volume Information` | ≤ C 盘的 3% | 执行阶段 2.3 第 2 步 |
| 根目录 `hiberfil.sys` | 0 或 ≤ 内存的 1/2 | 执行阶段 2.3 第 1 步 |

5. 执行目录测量命令，定位异常路径：

```powershell
$dirs = "C:\Windows\ServiceProfiles","C:\Windows\SoftwareDistribution","C:\Windows\Temp","C:\Windows\LastGood.Tmp"
foreach ($d in $dirs) {
  $size = (Get-ChildItem $d -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum
  "{0,8:N2} GB  {1}" -f ($size/1GB), $d
}
```

> 注意：`System Volume Information` 无管理员读取权限，测量结果为 0 属正常。该目录用 `vssadmin list shadowstorage` 测量。

6. 展开 `ServiceProfiles`，确认大头属于哪一类：
   - 属于 `...\DeliveryOptimization\Cache` → 执行 2.1 第 1 步。
   - 属于 `LocalService\AppData\Local\Temp` 下的 `.dmp` → 执行 2.1 第 2 步。
   - 两者都有 → 两步都执行。

---

## 4. 阶段 2：清理操作（必须按顺序执行）

### 2.1 清理传递优化缓存与服务转储

1. 清理传递优化缓存，执行：

```powershell
Delete-DeliveryOptimizationCache -Force
```

2. 清理服务转储，执行：

```powershell
Remove-Item "C:\Windows\ServiceProfiles\LocalService\AppData\Local\Temp\*.dmp" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "C:\Windows\ServiceProfiles\NetworkService\AppData\Local\Temp\*.dmp" -Recurse -Force -ErrorAction SilentlyContinue
```

3. 出现"文件正在使用"提示时，跳过该项。此提示不影响结果。

### 2.2 清理更新缓存

1. 打开管理员 CMD。
2. 逐行执行下列命令。不得合并执行。

```cmd
net stop wuauserv
net stop bits
net stop cryptsvc
del /f /s /q C:\Windows\SoftwareDistribution\Download
net start cryptsvc
net start bits
net start wuauserv
```

3. 只删除 `Download` 子目录。禁止删除 `DataStore`。

### 2.3 可选清理

1. 不用休眠功能时，执行第 1 条命令，回收约 6 GB：

```cmd
powercfg /h off
```

2. 把还原点上限压到 3%，执行第 1 条命令，回收约 3 GB：

```cmd
vssadmin resize shadowstorage /for=C: /on=C: /maxsize=3%
```

3. 清理上次启动备份，执行第 1 条命令：

```cmd
del /f /q "C:\Windows\LastGood.Tmp\*"
```

---

## 5. 阶段 3：防复发配置（一次执行，长期有效）

1. 打开管理员 PowerShell。
2. 执行下列命令，设置四项策略：

```powershell
New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization" -Force | Out-Null
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization" -Name "DOMaxCacheSize" -Value 2 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization" -Name "DOAbsoluteMaxCacheSize" -Value 5 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization" -Name "DOMaxCacheAge" -Value 86400 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization" -Name "DODownloadMode" -Value 0 -Type DWord
Restart-Service DoSvc
```

3. 各值的含义见下表。

| 值名 | 设定值 | 含义 |
|---|---|---|
| `DOMaxCacheSize` | 2 | 缓存上限为 C 盘的 2% |
| `DOAbsoluteMaxCacheSize` | 5 | 缓存绝对上限 5 GB |
| `DOMaxCacheAge` | 86400 | 缓存 1 天后自动删除 |
| `DODownloadMode` | 0 | 只从微软服务器下载，关闭 P2P |

4. 设置界面执行一次：打开"设置 → Windows 更新 → 高级选项 → 传递优化"。
5. 关闭开关"允许从其他设备下载"。
6. 策略键优先于设置界面。策略键为 0 时，设置界面显示不一致不影响结果。
7. "上传限制"设置只管流量，不管磁盘。可设为 5 GB。该设置与空间无关。

---

## 6. 阶段 4：验证

每项验证必须记录"命令输出"和"是否通过"。

### 4.1 验证基线恢复

1. 执行第 2 节第 3 步的命令。
2. 通过标准：已用值比基线减小。本次实测 101.9 GB → 75.2 GB。

### 4.2 验证策略生效

1. 执行：

```powershell
$p = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization"
if (Test-Path $p) {
  Get-ItemProperty $p | Format-List DOMaxCacheSize, DOAbsoluteMaxCacheSize, DOMaxCacheAge, DODownloadMode
} else {
  "策略键不存在 —— 缓存上限未设置"
}
```

2. 通过标准：四个字段依次输出 `2`、`5`、`86400`、`0`。
3. 输出"策略键不存在"时，判为失败。重新执行第 5 节第 2 步。

### 4.3 验证更新缓存

1. 执行：

```powershell
"{0:N2} GB" -f ((Get-ChildItem C:\Windows\SoftwareDistribution\Download -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum/1GB)
```

2. 通过标准：清理后输出 < 0.10 GB。
3. 日常阈值：数值 ≥ 2.00 GB 时，执行第 4.2 节命令。

### 4.4 运行时验证（可选）

1. 在 Windows 正在下载更新时执行：

```powershell
Get-DeliveryOptimizationStatus | Select-Object DownloadMode, Status
```

2. 通过标准：`DownloadMode` 为 `0`。
3. 空闲时输出警告 `No active Delivery Optimization download or upload jobs` 属正常。

---

## 7. 禁止操作

1. 禁止手删 `C:\Windows\WinSxS` 内的任何文件。
2. 禁止手删 `C:\Windows\Installer`。
3. 禁止删除 `SoftwareDistribution\DataStore`。
4. 禁止删除 `C:\Recovery`、`C:\$Extend`、`C:\ESP-backup`。
5. 禁止删除 `C:\ProgramData` 下的 `Package Cache`。
6. 组件存储超过 15 GB 时，只允许执行 DISM，不允许手删：

```cmd
Dism.exe /online /Cleanup-Image /StartComponentCleanup
```

> 不加 `/ResetBase` 参数。加了会让已装更新无法卸载。

---

## 8. 记录表（每次操作填写）

| 日期 | 基线已用 | 清理后已用 | ServiceProfiles | SoftwareDistribution | 执行的步骤 | 验证结果 |
|---|---|---|---|---|---|---|
| 2026-11-07 | 101.9 GB | 75.2 GB | 13.7→0.05 GB | 7.8→0.07 GB | 2.1、2.2、5 | 4.1~4.3 通过 |
| | | | | | | |

---

## 9. 维护周期

1. 每月执行一次 4.3 节命令。
2. 每次 Windows 功能更新后，执行第 3 节诊断。
3. `ServiceProfiles` 超过 1 GB 时，执行第 4.1 节命令。
4. 策略键被系统重置时，重新执行第 5 节命令。

---

**预期总效果**：阶段 2 + 2.3 全部执行，回收 25 ~ 30 GB。阶段 3 生效后，传递优化缓存不再超过 5 GB。更新缓存按月手工清一次，不复涨。

这份覆盖了 C 盘治理的"诊断 → 清理 → 防复发 → 验证 → 记录"闭环。