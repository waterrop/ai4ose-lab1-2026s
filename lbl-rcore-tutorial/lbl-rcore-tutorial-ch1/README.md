# 第一章：应用程序与基本执行环境

> 适配学习者：你具备 Rust 基础（能读不会写）、计算机组成原理知识、操作系统理论背景。本教程将帮助你把理论转化为实践。

## 学习目标

在本章结束时，你将能够：

1. **理解**为什么"Hello, world!"在裸机上并不简单
2. **掌握**如何在没有操作系统的情况下运行 Rust 程序
3. **理解** RISC-V 的启动流程和特权级机制
4. **理解** SBI（Supervisor Binary Interface）的作用

---

## 前置知识对照

如果你对以下概念感到陌生，请先回顾相关教材：

| 你已熟悉的概念 | 本章对应的技术点 | 章节位置 |
|--------------|----------------|----------|
| 计算机组成原理：CPU 上电复位后的第一条指令 | RISC-V 启动流程：`_start` 入口 | 6.3 节 |
| 操作系统：系统调用（如 `sys_write`） | SBI 调用（`ecall` 指令） | 6.4 节 |
| 计算机组成原理：CPU 特权级（Ring 0/3） | RISC-V 特权级（M/S/U-mode） | 6.4 节 |
| 操作系统：进程/内核栈 | 裸机栈空间设置 | 6.3 节 |
| 链接器/加载器 | 链接脚本控制内存布局 | 6.3 节 |

---

## 项目结构

```
lbl-rcore-tutorial-ch1/
├── .cargo/
│   └── config.toml     # Cargo 配置：指定交叉编译目标和 QEMU runner
├── build.rs            # 构建脚本：自动生成链接脚本
├── Cargo.toml          # 项目配置与依赖
├── README.md           # 本文档
└── src/
    └── main.rs         # 程序源码：入口、主函数、panic 处理
```

---

## 一、环境准备

> 如果你已安装过 Rust 和 QEMU，可以跳过本节。

### 1.1 安装 Rust 工具链

本项目使用 Rust 语言编写，需要通过 rustup 安装 Rust 工具链。

**Linux / macOS / WSL：**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

**Windows：**

从 [https://rustup.rs](https://rustup.rs) 下载并运行 `rustup-init.exe`。

验证安装：

```bash
rustc --version    # 应显示 rustc 1.xx.x
cargo --version    # 应显示 cargo 1.xx.x
```

### 1.2 添加 RISC-V 64 编译目标

由于 lbl-rcore-tutorial-ch1 是面向 RISC-V 64 裸机平台的程序，需要添加对应的编译目标：

```bash
rustup target add riscv64gc-unknown-none-elf
```

**这个目标三元组的含义：**

| 部分 | 含义 | 类比（你熟悉的概念） |
|------|------|---------------------|
| `riscv64gc` | RISC-V 64 位，支持 G（通用）和 C（压缩）指令集扩展 | x86_64 类似 |
| `unknown` | 没有特定的 CPU 厂商 | - |
| `none` | 没有操作系统 | 嵌入式/裸机 |
| `elf` | 生成 ELF 格式的可执行文件 | Windows 的 .exe |

### 1.3 安装 QEMU 模拟器

lbl-rcore-tutorial-ch1 在 QEMU 模拟的 RISC-V 64 虚拟机上运行，需要安装 `qemu-system-riscv64`（建议版本 >= 7.0）。

**Ubuntu / Debian：**

```bash
sudo apt update
sudo apt install qemu-system-misc
```

**macOS（Homebrew）：**

```bash
brew install qemu
```

**验证安装：**

```bash
qemu-system-riscv64 --version
```

### 1.4 获取源代码

```bash
# 方式一：只获取本实验
# （注：本教程使用方式二）
# cargo clone lbl-rcore-tutorial-ch1

# 方式二：从本仓库获取
cd /home/hdu/study/rust/2026s-ai4ose-lab/ai4ose-lab1-2026s/lbl-rcore-tutorial/lbl-rcore-tutorial-ch1
```

---

## 二、编译与运行

### 2.1 编译

在 `lbl-rcore-tutorial-ch1` 目录下执行：

```bash
cargo build
```

> **什么是交叉编译？**
>
> 编译器运行在主机平台（如 `x86_64-unknown-linux-gnu`）上，但生成的可执行文件需要在目标平台（`riscv64gc-unknown-none-elf`）上运行——这种情况称为**交叉编译**。
>
> 这类似于在 x86 电脑上为 ARM 手机开发应用。

### 2.2 运行

```bash
cargo run
```

预期输出：

```
Hello, world!
```

输出一行 `Hello, world!` 后，QEMU 自动退出。

> **QEMU 参数说明（`.cargo/config.toml` 中配置）：**
>
> | 参数 | 含义 |
> |------|------|
> | `-machine virt` | 使用 QEMU 的 `virt` 虚拟平台 |
> | `-nographic` | 无图形界面，输出重定向到终端 |
> | `-bios none` | 不加载任何 BIOS/SBI 固件，程序自带 M-mode 启动代码 |
> | `-kernel <file>` | 将 ELF 可执行文件加载到内存中作为内核启动 |

---

## 三、核心概念

### 3.1 为什么"Hello, world!"并不简单？

在日常开发中，你写的应用程序运行在一个**多层执行环境栈**之上：

```
┌─────────────────────────┐
│      应用程序            │  ← 你写的代码（Hello, world!）
├─────────────────────────┤
│   标准库 (std / libc)    │  ← println! 等函数的实现
├─────────────────────────┤
│     操作系统内核         │  ← 系统调用：write, exit 等
├─────────────────────────┤
│   硬件抽象层 (SBI/BIOS)  │  ← 固件，为内核提供基础服务
├─────────────────────────┤
│       硬件 (CPU/内存)    │  ← 物理硬件
└─────────────────────────┘
```

当你在 Linux 上执行 `println!("Hello, world!")` 时，实际经历了：

`println!` → Rust 标准库 → libc 的 `write()` → Linux 内核 `sys_write` → 串口/终端驱动 → 屏幕显示

**lbl-rcore-tutorial-ch1 做了什么？** 它跳过了标准库和操作系统内核，直接在裸机上通过 SBI 接口输出字符。这就是"最小执行环境"的含义。

### 3.2 裸机编程基础：`#![no_std]` 和 `#![no_main]`

> 你可能见过 `println!`，但它在裸机环境下无法使用——因为它依赖操作系统。

#### `#![no_std]` —— 不使用标准库

```rust
#![no_std]
```

告诉 Rust 编译器**不链接标准库 `std`**，改用核心库 `core`。

| 标准库 (std) | 核心库 (core) |
|-------------|--------------|
| 依赖操作系统 | 不依赖任何外部环境 |
| 文件 I/O、网络、线程等 | 基本类型、迭代器、Option/Result 等 |

#### `#![no_main]` —— 不使用标准入口

```rust
#![no_main]
```

标准的 `main()` 函数需要 C runtime 进行初始化。在裸机环境中没有这些支持，所以我们自己定义程序入口点 `_start`。

#### `#[panic_handler]` —— 自定义 panic 处理

```rust
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    shutdown(true)  // 以异常状态关机
}
```

标准库提供了 panic 时打印错误信息并终止程序的功能。使用 `#![no_std]` 后，需要自己实现 panic 处理。

### 3.3 RISC-V 裸机启动流程

> 这类似于你学过的计算机组成原理中 CPU 上电后的启动过程。

```
QEMU 加电
    │
    ▼
PC = 0x1000（QEMU 内置引导代码）
    │
    ▼
跳转到 0x80000000（M-mode 入口，tg-rcore-tutorial-sbi 的 _m_start）
    │  ── 在 M-mode 下初始化硬件环境
    │  ── 设置中断委托、PMP 等
    ▼
跳转到 0x80200000（S-mode 入口，lbl-rcore-tutorial-ch1 的 _start）
    │  ── 设置栈指针 sp
    ▼
跳转到 rust_main()
    │  ── 打印 "Hello, world!"
    │  ── 调用 SBI shutdown 关机
    ▼
QEMU 退出
```

**关键地址：**

| 地址 | 含义 |
|------|------|
| `0x80000000` | M-mode 代码起始地址（由链接脚本的 `M_BASE_ADDRESS` 指定） |
| `0x80200000` | S-mode 代码起始地址（`_start` 函数所在位置） |

**为什么需要两个地址？**

这涉及到 RISC-V 的特权级设计。M-mode（Machine Mode）拥有最高权限，负责硬件初始化；S-mode（Supervisor Mode）是操作系统内核运行的特权级。

### 3.4 特权级与 SBI

#### RISC-V 特权级

| 特权级 | 缩写 | 说明 | 类比 |
|--------|------|------|------|
| Machine Mode | M-mode | 最高特权级，直接访问所有硬件资源 | 计算机组成原理中的"管理模式" |
| Supervisor Mode | S-mode | 操作系统内核运行的特权级 | Linux Ring 0 |
| User Mode | U-mode | 应用程序运行的特权级 | Linux Ring 3 |

**特权级切换通过 `ecall` 指令实现：**

- 应用程序（U-mode）执行 `ecall` → 陷入操作系统（S-mode）：这是**系统调用**
- 操作系统（S-mode）执行 `ecall` → 陷入固件（M-mode）：这是 **SBI 调用**

虽然都是 `ecall` 指令，但因为所在特权级不同，效果也不同。

#### SBI（Supervisor Binary Interface）

SBI 是 RISC-V 的标准规范，定义 S-mode 软件（操作系统）向 M-mode 固件请求服务的接口。可以把 SBI 理解为"**操作系统的操作系统**"——它为操作系统提供最基本的硬件抽象服务。

本章使用了两个 SBI 函数：

| 函数 | 作用 |
|------|------|
| `console_putchar(c)` | 向控制台输出一个字符（通过串口） |
| `shutdown(fail)` | 关闭虚拟机 |

---

## 四、代码解读

### 4.1 项目配置：Cargo.toml

```toml
[package]
name = "lbl-rcore-tutorial-ch1"
edition = "2024"

[dependencies]
tg-sbi = { package = "tg-rcore-tutorial-sbi", path = "../../tg-rcore-tutorial/tg-rcore-tutorial-sbi", version = "0.4.5", features = ["nobios"] }
```

关键配置：
- `edition = "2024"`：使用 Rust 2024 edition
- `panic = "abort"`：panic 时直接终止，不进行栈展开
- `tg-rcore-tutorial-sbi` 依赖启用了 `nobios` 特性，使其内建 M-mode 启动代码

### 4.2 程序入口：_start 函数

```rust
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 4096;

    #[unsafe(link_section = ".bss.uninit")]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    core::arch::naked_asm!(
        "la sp, {stack} + {stack_size}", // 设置栈指针
        "j  {main}",                      // 跳转到 rust_main
        stack_size = const STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
    )
}
```

**逐行解释：**

1. `#[unsafe(naked)]` - 这是一个**裸函数**，不生成函数序言（prologue）和尾声（epilogue），可以在没有栈的情况下执行
2. `#[unsafe(no_mangle)]` - 不改变函数名，方便链接器找到入口点
3. `#[unsafe(link_section = ".text.entry")] - 将函数放置在指定的段中
4. `static mut STACK` - 手动分配 4 KiB 栈空间
5. `naked_asm!` - 内联汇编，设置栈指针并跳转到 `rust_main`

### 4.3 主函数：rust_main

```rust
extern "C" fn rust_main() -> ! {
    for c in b"Hello, world!\n" {
        console_putchar(*c);
    }
    shutdown(false)
}
```

逐字符调用 `console_putchar` 输出 "Hello, world!\n"，然后调用 `shutdown(false)` 正常关机。

---

## 五、Rust 语法快速查阅

> 针对你"能看懂但不会写"的特点，这里列出本章涉及的 Rust 语法要点。

### 5.1 属性标记（Attributes）

```rust
#![no_std]       // 告诉编译器不使用标准库（crate 级属性）
#![no_main]      // 告诉编译器不使用标准入口（crate 级属性）
#[panic_handler] // 标记 panic 处理函数
#[unsafe(...)]   // 标记不安全的操作（Rust 2024 语法）
```

### 5.2 函数属性

```rust
#[unsafe(naked)]       // 裸函数：不生成函数序言/尾声
#[unsafe(no_mangle)]   // 不改变函数名
#[unsafe(link_section = "xxx")] // 将函数放入指定段
#[cfg(target_arch = "riscv64")] // 仅在 RISC-V 架构下编译
```

### 5.3 extern "C"

```rust
unsafe extern "C" fn _start() -> !
```

`extern "C"` 表示使用 C 语言的调用约定（参数传递、栈清理等规则）。在裸机编程中常用于：
- 与硬件/汇编代码交互
- 定义入口点

### 5.4 `-> !` 返回类型

```rust
fn rust_main() -> !
```

`!` 表示"永不返回"（never returns）。这个函数会一直运行直到关机。

### 5.5 bare loop

```rust
for c in b"Hello, world!\n"
```

`b"..."` 是字节字符串字面量，类型是 `&[u8; N]`。

---

## 六、本章小结

通过本章的学习和实践，你完成了从普通应用程序到裸机程序的蜕变过程：

1. **理解了执行环境**：应用程序依赖多层执行环境（标准库 → 操作系统 → 硬件），`Hello, world!` 的背后并不简单
2. **摆脱了标准库**：通过 `#![no_std]` 和 `#![no_main]`，让 Rust 程序不再依赖操作系统
3. **掌握了裸机启动流程**：从 QEMU 加电到 M-mode 初始化，再到 S-mode 的 `_start` 入口
4. **认识了 RISC-V 特权级和 SBI**：M-mode / S-mode / U-mode 的层次关系，以及 `ecall` 指令如何跨越特权级

这是操作系统内核开发的第一步——在后续章节中，我们将在这个最小执行环境的基础上，逐步添加批处理、多道程序、内存管理、进程调度等操作系统核心功能。

---

## 七、思考题

1. **为什么 `_start` 函数必须是裸函数（`#[naked]`）？** 如果不是裸函数会发生什么问题？
   > 提示：思考函数序言（prologue）需要什么前提条件。

2. **`ecall` 指令在不同特权级中的效果有何不同？** 为什么应用程序和操作系统都使用 `ecall`，却能产生不同的行为？

3. **如果把链接脚本中的 `S_BASE_ADDRESS` 从 `0x80200000` 改为其他值（如 `0x80100000`），程序还能正常运行吗？** 需要做哪些相应的修改？

---

## 参考资料

- [rCore-Tutorial-Guide 第一章](https://learningos.github.io/rCore-Tutorial-Guide/)
- [rCore-Tutorial-Book 第一章](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter1/index.html)
- [BlogOS: A Freestanding Rust Binary](https://os.phil-opp.com/freestanding-rust-binary/)
- [RISC-V Privileged Specification](https://riscv.org/specifications/privileged-isa/)
- [RISC-V SBI Specification](https://github.com/riscv-non-isa/riscv-sbi-doc)
- [RISC-V Reader 中文版](http://riscvbook.com/chinese/RISC-V-Reader-Chinese-v2p1.pdf)

---

## 依赖

| 依赖 | 说明 |
|------|------|
| `tg-rcore-tutorial-sbi` | SBI 调用封装库，支持 nobios 模式，内建 M-mode 启动代码 |

---

## License

Licensed under GNU GENERAL PUBLIC LICENSE, Version 3.0.
