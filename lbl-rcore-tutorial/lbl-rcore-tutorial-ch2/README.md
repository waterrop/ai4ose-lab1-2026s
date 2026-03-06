# 第二章：批处理系统

> 适配学习者：你已完成第一章学习，掌握了裸机编程基础。本章将带你进入真正的操作系统世界——实现一个能够运行多个用户程序的批处理系统。

## 学习目标

在本章结束时，你将能够：

1. **理解**批处理系统的工作原理
2. **掌握** RISC-V U-mode / S-mode 特权级切换过程
3. **理解** Trap（系统调用/异常）的触发和处理流程
4. **理解** 系统调用的实现原理：从用户态 `ecall` 到内核态处理
5. **理解** 用户程序如何被打包进内核并依次执行

---

## 前置知识（第一章回顾 + 本章新增）

### 你已具备的知识（来自 ch1）

| 概念 | 说明 |
|------|------|
| `#![no_std]` | 不使用 Rust 标准库 |
| `_start` 入口 | 裸机程序入口点 |
| SBI 调用 | 通过 `console_putchar` 输出字符 |
| RISC-V 特权级 | M-mode（固件）、S-mode（内核） |

### 本章新增概念

| 新概念 | 与 ch1/已学知识的联系 |
|--------|---------------------|
| 批处理系统 | ch1 只运行一个程序，ch2 依次运行多个 |
| U-mode 用户态 | ch1 无用户态，ch2 新增 |
| 特权级切换 U↔S | ch1 只有 M↔S，ch2 新增 U↔S |
| Trap 处理 | ch1 无用户态 Trap，ch2 需要处理 |
| 系统调用 | ch1 用 SBI，ch2 用户态通过 ecall 调用内核 |

---

## 项目结构

```
lbl-rcore-tutorial-ch2/
├── .cargo/
│   └── config.toml     # Cargo 配置：交叉编译目标和 QEMU runner
├── build.rs            # 构建脚本：下载编译用户程序，生成链接脚本和 APP_ASM
├── Cargo.toml          # 项目配置与依赖
├── README.md           # 本文档
├── test.sh             # 自动测试脚本
└── src/
    └── main.rs         # 内核源码：批处理主循环、Trap 处理、系统调用
```

---

## 一、环境准备

> 如果你已按照第一章配置好开发环境，只需额外安装以下工具。

### 1.1 额外工具安装

本章的构建脚本需要 `cargo-clone`（用于自动下载用户程序 crate）和 `rust-objcopy`（用于将 ELF 转为二进制）：

```bash
cargo install cargo-clone
# rust-objcopy 由 cargo-binutils 提供
cargo install cargo-binutils
rustup component add llvm-tools
```

### 1.2 获取源代码

```bash
cd /home/hdu/study/rust/2026s-ai4ose-lab/ai4ose-lab1-2026s/lbl-rcore-tutorial/lbl-rcore-tutorial-ch2
```

---

## 二、编译与运行

### 2.1 编译

```bash
cargo build
```

编译过程比第一章复杂，`build.rs` 会自动完成以下工作：

1. **生成链接脚本**：使用 `tg_linker::NOBIOS_SCRIPT` 生成内核的内存布局
2. **下载用户程序**：自动通过 `cargo clone` 获取 `tg-rcore-tutorial-user` crate
3. **编译用户程序**：为每个用户程序交叉编译到 RISC-V 64 目标
4. **生成 APP_ASM**：生成汇编文件，将所有用户程序的二进制数据内联到内核镜像中

### 2.2 运行

```bash
cargo run
```

**预期输出：**

```
[tg-rcore-tutorial-ch2 0.3.1-preview.1] Hello, world!
[ INFO] .data [0x802xxxxx, 0x802xxxxx)
[ WARN] boot_stack top=bottom=0x802xxxxx, lower_bound=0x802xxxxx
[ERROR] .bss [0x802xxxxx, 0x802xxxxx)
[ INFO] load app0 to 0x802xxxxx
Hello world from user mode program!
[ INFO] app0 exit with code 0

[ INFO] load app1 to 0x802xxxxx
...（更多用户程序输出）...
```

批处理系统依次加载并运行每个用户程序：
- 正常的用户程序会打印输出，然后通过 `exit` 系统调用退出
- 出错的用户程序会被内核杀死，然后继续运行下一个

### 2.3 测试验证

```bash
./test.sh
```

---

## 三、核心概念

### 3.1 批处理系统

> 这类似于你学过的操作系统理论中的"批处理系统"——最早期的操作系统形态。

**批处理系统**（Batch System）将多个程序打包到一起输入计算机，当一个程序运行结束后，计算机自动执行下一个程序。

```
内核启动
    │
    ▼
初始化（清零 BSS、初始化控制台和系统调用）
    │
    ▼
┌─→ 加载第 i 个用户程序
│       │
│       ▼
│   创建用户上下文（设置入口地址、用户栈、U-mode）
│       │
│       ▼
│   execute() → sret 切换到 U-mode 运行用户程序
│       │
│       ▼
│   用户程序触发 Trap（ecall 或异常）
│       │
│       ▼
│   内核处理 Trap（系统调用 / 杀死出错程序）
│       │
│       ├─ 系统调用 write → 输出数据，继续运行
│       ├─ 系统调用 exit  → 程序退出
│       └─ 异常            → 杀死程序
│       │
│       ▼
└── 加载下一个用户程序（i++）
        │
        ▼
    所有程序完成 → 关机
```

**为什么需要特权级？**

如果用户程序的错误（如访问非法地址、执行特权指令）能够影响内核的运行，那整个系统就不可靠了。特权级机制将用户程序和内核隔离，确保出错的用户程序只会被杀死，而不会破坏内核。

### 3.2 RISC-V 特权级机制

| 特权级 | 缩写 | 运行的软件 | 类比 |
|--------|------|-----------|------|
| Machine Mode | M-mode | SBI 固件 | 计算机组成原理中的"管理模式" |
| Supervisor Mode | S-mode | 操作系统内核 | Linux Ring 0 |
| User Mode | U-mode | 用户程序 | Linux Ring 3 |

**特权级切换的方向：**

- **U → S**（Trap）：用户程序执行 `ecall` 或发生异常时，CPU 自动陷入 S-mode
- **S → U**（sret）：内核执行 `sret` 指令返回 U-mode 继续运行用户程序

> **与 ch1 的联系**：ch1 只涉及 M↔S（S-mode 通过 SBI 调用 M-mode），ch2 新增了 U↔S。

### 3.3 Trap 处理

> 这类似于你学过的计算机组成原理中的"中断"或"异常"处理。

**Trap** 是 CPU 从低特权级陷入高特权级的机制，触发原因包括：
- **系统调用**：用户程序执行 `ecall` 指令
- **异常**：非法指令、访存错误、页错误等
- **中断**：时钟中断、外部中断等（本章暂不涉及）

**Trap 相关的 CSR（控制状态寄存器）：**

| CSR | 功能 |
|-----|------|
| `stvec` | Trap 处理入口地址 |
| `sepc` | Trap 发生前最后一条指令的地址 |
| `scause` | Trap 原因（系统调用、非法指令等） |
| `stval` | Trap 附加信息 |
| `sstatus` | SPP 字段记录 Trap 前的特权级 |

**Trap 处理流程：**

```
用户程序执行 ecall
       │
       ▼
  ┌── 硬件自动完成 ──┐
  │ 1. sstatus.SPP ← U  │  （记录 Trap 前的特权级）
  │ 2. sepc ← ecall 地址  │  （记录 Trap 前的 PC）
  │ 3. scause ← 原因      │  （如 UserEnvCall）
  │ 4. PC ← stvec         │  （跳转到 Trap 入口）
  │ 5. 特权级 ← S-mode    │  （切换到内核态）
  └──────────────────────┘
       │
       ▼
  Trap 入口（__alltraps）
  ── 保存所有用户寄存器到内核栈（Trap 上下文）
  ── 跳转到 Rust 的 trap_handler
       │
       ▼
  trap_handler 处理
  ── 读取 scause 判断 Trap 类型
  ── 系统调用：处理后 sepc += 4（跳过 ecall 指令）
  ── 异常：杀死程序
       │
       ▼
  __restore
  ── 从内核栈恢复用户寄存器
  ── 执行 sret 返回 U-mode
       │
       ▼
  用户程序从 ecall 的下一条指令继续执行
```

**为什么 sepc 要加 4？** 因为 `ecall` 指令本身占 4 字节。硬件将 `sepc` 设为 `ecall` 的地址，如果不加 4，`sret` 后会再次执行 `ecall`，陷入无限循环。

### 3.4 系统调用

> 这正是你熟悉的操作系统理论中的"系统调用"！在 RISC-V 中，它通过 `ecall` 实现。

系统调用是用户程序请求内核服务的唯一合法途径。用户程序将参数放入寄存器，执行 `ecall`，内核读取参数并处理。

**RISC-V 系统调用约定：**

| 寄存器 | 用途 |
|--------|------|
| `a7` | syscall ID |
| `a0` - `a5` | 参数 |
| `a0` | 返回值 |

**本章支持的系统调用：**

| syscall ID | 名称 | 功能 |
|-----------|------|------|
| 64 | `write` | 将缓冲区数据写入文件描述符（fd=1 为标准输出） |
| 93 | `exit` | 退出当前用户程序 |

**系统调用流程：**

```
用户程序调用 println!("Hello")
       │
       ▼
用户库将其转为 sys_write(fd=1, buf, len)
       │
       ▼
内嵌汇编：a7=64, a0=1, a1=buf, a2=len, ecall
       │
       ▼
Trap 进入内核 → handle_syscall
       │
       ▼
内核读取 a7=64 → 调用 write 处理函数
       │
       ▼
将 buf 指向的数据通过 SBI 输出到控制台
       │
       ▼
返回值写入 a0，sepc += 4，sret 回到用户态
```

### 3.5 用户程序的打包与加载

与第一章不同，本章需要将多个用户程序嵌入到内核中。`build.rs` 在编译时完成以下工作：

1. 自动下载 `tg-rcore-tutorial-user` crate
2. 逐个编译用户程序为 RISC-V 64 的 ELF 文件
3. 使用 `rust-objcopy` 将 ELF 转为纯二进制格式
4. 生成汇编文件，用 `.incbin` 指令将所有 .bin 文件嵌入到内核的 `.data` 段

---

## 四、代码解读

### 4.1 程序入口：_start 函数

与 ch1 类似，但分配的栈更大（32 KiB）：

```rust
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 8 * 4096;  // 8页 = 32 KiB（ch1 是 4 KiB）
    // ...
}
```

### 4.2 内核主函数：rust_main

核心的批处理循环：

```rust
extern "C" fn rust_main() -> ! {
    // 1. 清零 BSS 段
    unsafe { tg_linker::KernelLayout::locate().zero_bss() };

    // 2. 初始化控制台
    tg_console::init_console(&Console);

    // 3. 初始化系统调用
    tg_syscall::init_io(&SyscallContext);
    tg_syscall::init_process(&SyscallContext);

    // 4. 批处理循环
    for (i, app) in tg_linker::AppMeta::locate().iter().enumerate() {
        // 创建用户上下文
        let mut ctx = LocalContext::user(app_base);

        // 循环执行用户程序
        loop {
            unsafe { ctx.execute() };  // sret 到用户态

            // 处理 Trap
            match scause::read().cause() {
                Trap::Exception(Exception::UserEnvCall) => {
                    // 系统调用处理
                }
                trap => {
                    // 异常处理：杀死程序
                }
            }
        }
    }

    tg_sbi::shutdown(false)
}
```

### 4.3 系统调用处理：handle_syscall

```rust
fn handle_syscall(ctx: &mut LocalContext) -> SyscallResult {
    // a7 寄存器存放 syscall ID
    let id = ctx.a(7).into();
    // a0-a5 寄存器存放系统调用参数
    let args = [ctx.a(0), ctx.a(1), ctx.a(2), ctx.a(3), ctx.a(4), ctx.a(5)];

    match tg_syscall::handle(Caller { entity: 0, flow: 0 }, id, args) {
        Ret::Done(ret) => match id {
            Id::EXIT => SyscallResult::Exit(ctx.a(0)),
            _ => {
                *ctx.a_mut(0) = ret as _;
                ctx.move_next();  // sepc += 4
                SyscallResult::Done
            }
        },
        Ret::Unsupported(id) => SyscallResult::Error(id),
    }
}
```

### 4.4 接口实现：impls 模块

```rust
mod impls {
    // 控制台：通过 SBI 输出字符
    pub struct Console;
    impl tg_console::Console for Console {
        fn put_char(&self, c: u8) {
            tg_sbi::console_putchar(c);
        }
    }

    // 系统调用上下文
    pub struct SyscallContext;

    // write 系统调用
    impl tg_syscall::IO for SyscallContext {
        fn write(&self, ..., fd, buf, count) -> isize {
            match fd {
                STDOUT => { print!("..."); count as _ }
                _ => -1
            }
        }
    }

    // exit 系统调用
    impl tg_syscall::Process for SyscallContext {
        fn exit(&self, ..., status) -> isize { 0 }
    }
}
```

---

## 五、Rust 语法快速查阅（扩展）

### 5.1 trait（特征）

```rust
// trait 定义了一组方法签名
trait Console {
    fn put_char(&self, c: u8);
}

// 实现 trait
impl tg_console::Console for Console {
    fn put_char(&self, c: u8) {
        // 具体实现
    }
}
```

### 5.2 impl 块

```rust
// 为结构体实现方法
impl SyscallContext {
    fn new() -> Self {
        // 构造函数
    }
}
```

### 5.3 global_asm! 宏

```rust
// 将汇编代码嵌入到编译产物中
core::arch::global_asm!(include_str!(env!("APP_ASM")));
```

### 5.4 `-> !` 返回类型

```rust
fn rust_main() -> !  // "never returns"，永不返回
```

---

## 六、本章小结

通过本章的学习和实践，你在第一章的基础上迈出了重要的一步：

1. **理解了批处理系统**：操作系统自动依次加载和运行多个用户程序
2. **掌握了特权级机制**：U-mode / S-mode 的隔离保护了内核不受用户程序错误的影响
3. **理解了 Trap 处理流程**：从 `ecall` 触发到硬件自动保存 CSR，再到软件保存/恢复上下文
4. **实现了系统调用**：`write` 和 `exit` 是用户程序与内核交互的最基本接口
5. **了解了用户程序的打包**：在编译期将用户程序嵌入内核镜像

在后续章节中，我们将从批处理系统演进为**多道程序系统**和**分时共享系统**，实现多任务切换和时间片调度。

---

## 七、思考题

1. **为什么需要内核栈和用户栈分离？** 如果 Trap 处理时仍然使用用户栈，会有什么安全问题？

2. **`sepc` 在系统调用和异常时的值有何不同？** 为什么处理系统调用时需要将 `sepc` 加 4，而处理异常时不需要？

3. **`fence.i` 指令的作用是什么？** 在批处理系统中，为什么在加载下一个用户程序前需要执行这条指令？

4. **如果用户程序执行了 S-mode 的特权指令（如 `sret`），会发生什么？** 从特权级机制的角度解释这个行为。

---

## 参考资料

- [rCore-Tutorial-Guide 第二章](https://learningos.github.io/rCore-Tutorial-Guide/)
- [rCore-Tutorial-Book 第二章](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/index.html)
- [RISC-V Privileged Specification](https://riscv.org/specifications/privileged-isa/)
- [RISC-V Reader 中文版](http://riscvbook.com/chinese/RISC-V-Reader-Chinese-v2p1.pdf)

---

## 依赖

| 依赖 | 说明 |
|------|------|
| `tg-rcore-tutorial-sbi` | SBI 调用封装库 |
| `tg-rcore-tutorial-linker` | 链接脚本生成、内核布局定位 |
| `tg-rcore-tutorial-console` | 控制台输出和日志 |
| `tg-rcore-tutorial-kernel-context` | 用户上下文管理 |
| `tg-rcore-tutorial-syscall` | 系统调用定义与分发框架 |
| `riscv` | RISC-V CSR 寄存器访问 |

---

## License

Licensed under GNU GENERAL PUBLIC LICENSE, Version 3.0.
