---
name: rust
description: Rust 语言安装、环境配置、基础工具链使用指南
metadata: 
  node_type: memory
  type: reference
  updated: 2026-05-22
  tags: 
    - rust
    - toolchain
    - installation
    - cargo
  originSessionId: eb3af0e1-2c1a-40de-b846-89abc033dc80
---

# Rust 基础内容

## 1. 安装

### 环境：Windows Docker 容器中的 Linux (WSL2)

- **宿主机**: Windows (WSL2, Linux 6.6.87.2-microsoft-standard-WSL2)
- **容器系统**: Debian GNU/Linux 12 (bookworm)
- **安装用户**: app (uid=1000)，不需要 root 权限

#### 安装步骤

**标准方式（外网通畅时）：**

```bash
# 1. 安装 C 链接器（系统级，需要 root，但通常已预装）
sudo apt-get update && sudo apt-get install -y build-essential

# 2. 下载并安装 rustup（用户级，不需要 root）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 3. 加载环境变量
source "$HOME/.cargo/env"
```

**镜像方式（sh.rustup.rs 不通时，实测 2026-05-25）：**

某些 Docker 容器网络环境下 `sh.rustup.rs` 域名 SSL 握手失败（`SSL_ERROR_SYSCALL`），但 `static.rust-lang.org` 可达。

失败的尝试：设 `RUSTUP_DIST_SERVER` / `RUSTUP_UPDATE_ROOT` 环境变量无效——因为卡在获取安装脚本这一步（连接 `sh.rustup.rs` 本身就 SSL 失败），根本没走到下载工具链。

解决方法：用 USTC 镜像托管的那份安装脚本，完全绕开 `sh.rustup.rs`。

```bash
# 前提：先用 curl -I https://static.rust-lang.org 确认网络通
# 1. 用 USTC 镜像的安装脚本（完全绕开 sh.rustup.rs）
curl -sSf https://mirrors.ustc.edu.cn/misc/rustup-install.sh | sh

# 2. 选择 1（标准安装），然后加载环境
. "$HOME/.cargo/env"

# 3. 验证
rustc --version   # rustc 1.95.0
cargo --version   # cargo 1.95.0
```

> **决策树**：`sh.rustup.rs` 卡住 → `Ctrl+C` → 设镜像环境变量仍 SSL 失败 → 用 USTC 镜像的安装脚本替代

#### 安装结果

```
rustc 1.95.0 (59807616e 2026-04-14)
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
```

- rustup home: `~/.rustup`
- cargo home: `~/.cargo`
- 二进制路径: `~/.cargo/bin`
- 默认工具链: `stable-x86_64-unknown-linux-gnu`

#### 踩坑记录

- **不要用 root 安装**：rustup 装到 `$HOME` 下，root 的 `$HOME` 是 `/root/`（权限 `drwx------`），其他用户无法访问。应直接用目标用户安装
- **build-essential 是系统级依赖**：提供 `cc` 链接器，很多 crate 底层依赖 C 代码（OpenSSL、SQLite 等），缺少会报 `linker 'cc' not found`。它不影响 Rust 工具链的干净度，保留即可
- 容器重启后 PATH 不会自动加载 `cargo`，需要手动 `source "$HOME/.cargo/env"` 或加入 `.bashrc`
- 下载中断时清理 `~/.rustup/downloads/` 后重试即可
- **安装后验证**：`cargo new hello-world && cd hello-world && cargo run`，输出 `Hello, world!` 即表示环境正常

### Windows 原生安装

1. 下载 [rustup-init.exe](https://www.rust-lang.org/zh-CN/tools/install) 并运行
2. 安装 C 编译环境（二选一）：
   - **MSVC（推荐）**：安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，勾选 "C++ 桌面开发" 工作负载
   - **MinGW + GCC**：安装 [MSYS2](https://www.msys2.org/)，然后 `pacman -S mingw-w64-x86_64-gcc`，并将 `gcc` 所在目录加入 PATH
3. 运行 `rustup-init.exe`，按提示完成安装
4. 打开新的终端，验证：`rustc --version`

> [!info] Rust 的 Windows 安装器会自动检测本机的 C 编译环境。MSVC 是官方推荐方案，兼容性最好。

#### GNU 工具链轻量化安装（免 Visual Studio）

> [!tip] 适用场景：不想安装几个 GB 的 Visual Studio，只想轻量化搭建 Rust 开发环境。采用 GNU 工具链（`x86_64-pc-windows-gnu`）替代默认的 MSVC 工具链。

**准备工作：**

1. 访问 [Rust 官网下载页面](https://www.rust-lang.org/zh-CN/tools/install) 下载 `rustup-init.exe`
2. 下载并安装 [Visual Studio Code](https://code.visualstudio.com/Download)（主编辑器）

**核心步骤：自定义安装 Rust（切换 GNU 模式）：**

双击 `rustup-init.exe`，会提示缺少 Visual C++ 依赖。**不要选择默认安装，按以下步骤操作：**

1. **进入自定义设置**：看到 `1) Quick install`、`2) Manually install` 等选项时，输入 `2`（Customize installation）并回车
2. **修改目标平台（关键）**：
   - `Default host triple? [x86_64-pc-windows-msvc]` → 手动输入 `x86_64-pc-windows-gnu`（不要直接回车）
   - `Default toolchain? (stable/beta/nightly/none) [stable]` → 直接回车
   - `Profile? (minimal/default/complete) [default]` → 直接回车
   - `Modify PATH variable? (Y/n)` → 输入 `y` 并回车
3. **确认安装**：确认 `default host triple: x86_64-pc-windows-gnu` 后，输入 `1`（Proceed with selected options）

当看到 `Rust is installed now. Great!` 时，安装成功。

**配置 VS Code：**

安装两个核心插件：
- **rust-analyzer**：代码高亮、实时语法检查、智能补全和类型推导
- **Native Debug**：断点单步调试

安装后彻底重启 VS Code，确保读取到新的环境变量。

**验证：**

```bash
cargo new my_project
cd my_project
cargo run
```

**常见问题：**

> [!question] `cargo run` 报错找不到链接器 `cc` 或 `gcc`？
> Windows 纯净系统缺少 GNU 链接器。**不要装 Visual Studio**。安装 [Git](https://git-scm.com/)（自带轻量链接器）或单独下载 MinGW（不到 100MB），将其 `bin` 目录加入系统环境变量 `Path`，重启 VS Code。

> [!question] 什么是 `cargo check`？
> 只做语法和类型检查，不生成可执行文件，比 `cargo run` 快得多。

> [!question] 编译后的文件在哪里？
> 都在项目根目录的 `target/` 文件夹中。磁盘不够时可运行 `cargo clean` 一键清理。

### macOS

macOS 自带 Clang（Xcode Command Line Tools），无需额外安装 C 编译器：

```bash
xcode-select --install   # 如果尚未安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### IDE 搭建（VSCode）

推荐使用 VSCode + 以下扩展：

| 扩展 | 用途 |
|------|------|
| rust-analyzer | 代码补全、类型推断、跳转定义（官方推荐的 LSP 服务端） |
| Native Debug | 基于 GDB/LLDB 的调试支持（断点、单步、变量查看） |

安装步骤：

1. 安装 VSCode 扩展：在扩展面板搜索 `rust-analyzer` 和 `Native Debug` 安装
2. rust-analyzer 会自动检测 rustup 管理的工具链，无需额外配置
3. 调试配置：在项目 `.vscode/launch.json` 中添加 Cargo launch 配置，或直接按 F5 让 VSCode 自动生成

> [!tip] rust-analyzer 是独立的后台进程，首次打开项目时会建立索引，期间 CPU 占用较高，完成后恢复正常。

## 2. 工具链管理

```bash
# 查看已安装的工具链
rustup toolchain list

# 更新到最新稳定版
rustup update

# 切换工具链
rustup default stable
rustup default nightly

# 查看当前版本
rustc --version
cargo --version
```

## 3. Cargo 是什么

Cargo 是 Rust 的官方构建系统和包管理器，两个核心职责：

- **构建系统**：管理项目的编译、测试、运行，自动处理依赖的编译和链接
- **包管理器**：通过 crates.io 搜索、添加、管理第三方库，处理版本兼容性

关键能力：依赖管理（Cargo.toml）、构建缓存、离线工作、跨平台构建、发布到 crates.io、自定义构建脚本、基准测试（cargo bench）、插件系统、环境变量覆盖

## 4. Cargo 基础命令

```bash
# 创建新项目
cargo new my-project
cargo new --lib my-lib

# 构建
cargo build           # debug
cargo build --release  # release

# 运行
cargo run

# 测试
cargo test

# 格式化
cargo fmt

# 代码检查（比编译更快的 lint）
cargo clippy

# 添加依赖
cargo add <crate-name>

# 检查语法和类型错误（比 build 更快，不生成二进制）
cargo check

# 清理构建产物（target/ 目录）
cargo clean

# 更新依赖到最新兼容版本
cargo update

# 发布到 crates.io
cargo publish

# 基准测试
cargo bench
```

## 5. 项目结构

```
my-project/
├── Cargo.toml       # 项目配置：[package] 元信息 + [dependencies] 依赖声明
├── Cargo.lock       # 依赖版本锁定（自动生成，二进制项目提交 git，库项目不提交）
├── src/
│   ├── main.rs      # 入口文件
│   └── lib.rs       # 库入口（可选）
├── tests/           # 集成测试
└── examples/        # 示例代码
```

## 6. 常用 Crate

| Crate | 用途 |
|-------|------|
| serde | 序列化/反序列化 |
| tokio | 异步运行时 |
| clap | 命令行参数解析 |
| anyhow | 简化错误处理 |
| reqwest | HTTP 客户端 |

## 7. 核心概念理解

### 7.1 编译器架构：rustc 与 LLVM

- rustc = 前端（解析、类型检查、借用检查、单态化），输出 LLVM IR
- LLVM = 通用后端（优化 + 机器码生成），被 C/C++/Swift/Rust 等多种语言共享
- Rust 借此从第一天就拥有工业级优化器

### 7.2 零成本抽象

- 高级语法糖经编译器优化后，生成的机器码与手写底层代码无异
- 成本转移到编译时——编译器多花时间，运行时零代价
- 关键技术：单态化（为每个具体类型生成专用代码，避免运行时动态分发）

### 7.3 零开销 FFI

- Rust 调 C 函数 = 一条 CPU call 指令，和 C 调 C 没有区别
- 零开销的根基：内存布局兼容、调用约定相同、无 GC、无运行时
- 对比：Java JNI 需要数据编组（marshalling），开销显著

### 7.4 内存管理三种方式

| 方式 | 代表 | 谁决定释放 | 运行时开销 |
|------|------|-----------|-----------|
| GC | Java/Python/Go | 运行时自动 | 扫描+标记+释放，可能暂停程序 |
| 手动管理 | C/C++ | 程序员 | 无 |
| 编译时释放 | Rust | 编译器（所有权规则） | 无 |

- 手动管理 ≠ GC。GC 是运行时自动回收，手动是程序员自己 free
- Rust 折中：编译器通过所有权规则自动确定释放位置，插入释放指令

### 7.5 零成本抽象的取舍

- 代价：编译慢 + 语言复杂 + 开发效率低
- 大多数场景（Web、脚本、业务应用）GC 开销可接受，不值得
- 只有系统级领域（OS、游戏引擎、浏览器、嵌入式）才值得
- C++ 是零成本抽象鼻祖，Rust 继承了这一理念

### 7.6 Rust 的核心创新：编译器代理内存管理

- C++ 最难最容易出错的部分是手动管理内存（忘记释放→泄漏，释放两次→崩溃，释放后使用→未定义行为）
- Rust 把这个决策交给编译器，通过所有权规则实现：每个值有且只有一个所有者，所有者离开作用域时自动释放
- 与 GC 的区别：GC 是运行时回收，Rust 是编译时决定释放位置
- 代价：编译器严格检查，无法确定生命周期就拒绝编译——学习曲线陡峭的本质是在和编译器"谈判"，证明内存使用安全

### 7.7 遮蔽（Shadowing）vs 覆盖（Overwriting）

不可变变量（`let`）和常量（`const`）表面都"不能改"，但机制不同：

- **常量**：编译时确定，不能遮蔽，必须声明类型
- **不可变变量**：运行时绑定，可以被遮蔽重新绑定（甚至换类型）

遮蔽与覆盖的本质区别在内存层面：

- **覆盖（mut）**：同一块内存地址，值原地改写，类型必须相同
- **遮蔽（shadow）**：新分配一块内存，名字重新绑定到新地址，旧值不动，类型可以不同

```rust
// 覆盖：同一块内存，123 被 456 替换
let mut a = 123;
a = 456;

// 遮蔽：新内存地址，旧的 123 仍然存在，只是名字 "a" 不再指向它
let a = 123;
let a = 456;
```

遮蔽允许类型变化（覆盖不允许）：

```rust
let a = 123;        // a: i32（4 字节）
let a = "hello";    // a: &str（16 字节），新地址新类型
```

旧值仍在内存中的证据：

```rust
let a = 123;
let b = &a;          // b 指向旧的 a
let a = 456;         // 新 a 在新地址
println!("{}", b);   // 打印 123，旧值毫发无损
```

| | 覆盖（mut） | 遮蔽（shadowing） |
|---|---|---|
| 内存 | 同一块地址，原地改 | 新地址，旧值不动 |
| 类型 | 必须相同 | 可以不同 |
| 旧值 | 真的没了 | 还在，但名字指向新值 |

Rust 区分两者的安全意义：遮蔽不会导致引用悬空（旧值还在，引用仍然有效），而 mut 改值后已有的引用可能指向意外的新值——这就是为什么 Rust 要求 &mut 是独占的

#### 遮蔽后的旧值生命周期

遮蔽只是让名字不再指向旧值，但编译器仍然追踪旧值的生命周期，不会造成内存泄漏：

- 无引用指向旧值时，编译器通过 NLL（Non-Lexical Lifetimes）在最早安全时机自动释放
- 有借用指向旧值时（如 `let b = &a`），编译器延迟释放直到借用结束
- 整个过程完全在编译时确定，零运行时开销

关键认知：编译器跟踪的是**值本身的生命周期**，不是变量名。变量名只是访问值的入口，遮蔽切断的是入口，不是值的存在。

### 7.8 栈与堆

栈和堆是进程内存中两块独立的区域，从两端相向增长：

```
高地址 ┌────────────┐
       │   栈区     │ ← 向下增长，CPU 硬件直接管理（栈指针寄存器 + push/pop）
       │            │
       │   （可用）  │ ← 不是分隔符，只是双方还没占用的空间
       │            │
       │   堆区     │ ← 向上增长，软件分配器管理（追踪空闲空间、分配回收）
低地址 └────────────┘
```

#### 栈（Stack）

- 大小固定，编译时确定，直接存值
- 分配/释放极快：只需移动栈顶指针（一条 CPU 指令）
- 离开作用域自动释放，不需要手动管理
- 硬件原生支持：CPU 有专用栈指针寄存器（x86 的 rsp）、push/pop 指令、函数调用依赖栈

局限：只能存编译时确定大小的数据。运行时才知道大小的数据（如用户输入的字符串）放不下。

#### 堆（Heap）

- 大小不固定，运行时按需向分配器申请
- 分配/释放慢：分配器要找空闲空间、记录分配信息
- 需要管理：C/C++ 手动 `free`/`delete`，Rust 编译器通过所有权自动管理
- 堆分配器内部用数据结构追踪空闲块（空闲链表 freelist 是其中一种方式）

#### 指针：连接栈和堆的桥梁

堆上的数据通过指针访问。指针是固定大小的地址（8 字节），可以存在于任何位置：

```
栈                              堆
┌──────────┐                   ┌─────────────────┐
│ head ───────────────────────→│ 节点1 │ next ────┤──→ 节点2 → ...
└──────────┘                   └─────────────────┘
  指针在栈上                      指针在堆上（next 字段）
```

指针不一定要在栈上。链表中节点之间的 next 指针就是"堆上指向堆上"的。栈上只存 head 一个入口指针。

#### Rust 中栈与堆的体现

```rust
let a = 123;                   // i32，固定 4 字节，直接存栈上
let s = String::from("hello"); // 栈上存：指针 + 长度 + 容量（固定 24 字节）
                               // 堆上存：实际的 "hello" 字节
```

#### 关键认知

- 空闲区域不是栈和堆之间的分隔标记，只是暂未被双方占用的空间。栈和堆碰上了就报错（栈溢出或内存不足）
- 栈是 CPU 硬件直接支持的原始机制；堆是软件层额外引入的管理机制，为了应对运行时才知道大小的数据
- C/C++ 的栈变量也是自动管理的（和 Rust 一样），内存泄漏只发生在堆上——丢失了指向堆内存的指针，那块内存就一直占用直到进程退出

#### 栈上存固定部分，堆上存实际内容

以 `String::from("hello")` 为例：

```
栈上（永远 24 字节）                堆上（大小不固定）
┌───────────────────┐              ┌─────────────────┐
│ ptr ───────────────────────────→ │ h e l l o       │
│ len: 5            │              └─────────────────┘
│ capacity: 5       │
└───────────────────┘
  8字节 + 8字节 + 8字节 = 24字节     5字节
```

不管字符串是 "hi"（2字节）还是 1 万个字，栈上的结构永远是 ptr + len + capacity = 24 字节。变的只是 ptr 指向的堆区域大小。

核心规则：编译时能确定大小 → 默认在栈上；编译时确定不了大小 → 借助堆。固定大小也可以主动放堆上（如 `Box::new(123)`），用于避免栈溢出或处理大对象。

#### 堆分配器的工作方式

堆分配器内部用数据结构（如空闲链表 freelist）追踪哪些空间空闲/已分配。分配器的管理数据不存放在栈上，而是存放在分配器自身的内部结构中（全局变量或嵌入堆的空闲块头部）。栈上存的是程序变量（如 `let head = ...`），这些变量恰好是指针指向堆数据，但分配器如何管理堆与栈无关。

#### 栈分配 vs 堆分配的具体过程

**栈分配（2 条 CPU 指令）：**

CPU 有一个专用寄存器——栈指针（SP），指向栈当前位置。栈从高地址向低地址增长。

```
分配 let a: i32 = 123;（需要 4 字节）
1. sub sp, 4           ← SP 往下移 4 格
2. mov [sp], 123       ← 把 123 写入 SP 指向的地址

释放（离开作用域）：
1. add sp, 4           ← SP 往上移 4 格
不擦除数据，只移动指针。旧数据下次分配时被覆盖。
```

SP 移到哪，就代表栈占用了多少内存，同时也指向最后一个栈变量。

**堆分配（几十到几百条指令）：**

```
malloc(5) 的过程：
1. 进入分配器函数
2. 遍历空闲链表找足够大的块
3. 块太大 → 分割，块刚好 → 直接用
4. 从空闲链表中摘除，标记为"已分配"
5. 更新链表指针
6. 返回块地址

释放（如 String 离开作用域）：
1. 分配器收到地址
2. 查找分配记录，标记为"空闲"
3. 检查相邻块是否空闲 → 是则合并
4. 挂回空闲链表
```

**栈快堆慢的本质：** 栈是硬件级的指针加减法，堆是软件级的查找和管理。

#### 释放不等于擦除

释放内存 = 标记空间"可以用了"，不是擦除数据。旧数据留在原处，下次分配到同一块空间时直接覆写。

这也解释了：
- C 语言未初始化变量的值是"随机的"——读到的是上一次用这块内存时留下的旧数据
- Rust 要求变量必须初始化——正是因为旧数据不会被清除，读到的都是垃圾值，Rust 在编译时禁止这个问题
- 遮蔽后旧值还存在——内存中的数据没有被清零，只是没有名字指向它了，编译器允许那块空间被覆盖
