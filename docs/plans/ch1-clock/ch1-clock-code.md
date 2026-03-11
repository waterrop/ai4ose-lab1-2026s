# tg-rcore-tutorial-ch1-clock 详细实现指南

本文档详细说明如何在 `tg-rcore-tutorial-ch1` 的基础上实现定时输出字符的内核。

## 目录

- [项目结构](#项目结构)
- [实现步骤](#实现步骤)
- [常见困难及解决方案](#常见困难及解决方案)

---

## 项目结构

```
tg-rcore-tutorial-ch1-clock/
├── .cargo/
│   └── config.toml           # 复制自 ch1
├── build.rs                  # 复制自 ch1
├── Cargo.toml                # name 改为 ch1-clock
├── rust-toolchain.toml       # 复制自 ch1
└── src/
    ├── main.rs              # 主代码
    └── sie.rs               # 新增：S-mode 中断使能模块
```

---

## 实现步骤

### 步骤 1: 创建 `src/sie.rs`

创建 `src/sie.rs` 文件，实现 S-mode 中断使能功能：

```rust
//! S-Mode Interrupt Enable (sie) 寄存器操作
//!
//! 本模块提供 S-mode 定时器中断的启用功能。

/// 启用 S-mode 定时器中断
///
/// 需要完成以下两步：
/// 1. 设置 sstatus 寄存器的 SIE 位 (bit 1)
/// 2. 设置 sie 寄存器的 STIE 位 (bit 5)
#[inline]
pub fn set_stimer() {
    unsafe {
        // ========== 步骤 1: 启用全局中断 (SIE in sstatus) ==========
        // 读取 sstatus 寄存器
        let mut sstatus: usize;
        core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus);

        // 设置 SIE 位 (bit 1)
        sstatus |= 1 << 1;

        // 写回 sstatus 寄存器
        core::arch::asm!("csrw sstatus, {0}", in(reg) sstatus);

        // ========== 步骤 2: 启用定时器中断 (STIE in sie) ==========
        // 读取 sie 寄存器
        let mut sie: usize;
        core::arch::asm!("csrr {0}, sie", out(reg) sie);

        // 设置 STIE 位 (bit 5)
        sie |= 1 << 5;

        // 写回 sie 寄存器
        core::arch::asm!("csrw sie, {0}", in(reg) sie);
    }
}
```

### 步骤 2: 修改 `src/main.rs` - 顶部声明

在 `src/main.rs` 顶部添加模块声明：

```rust
// 不使用标准库
#![no_std]
#![no_main]
#![cfg_attr(target_arch = "riscv64", deny(warnings, missing_docs))]

// 引入 SBI 调用
use tg_sbi::{console_putchar, set_timer, shutdown};

// 新增：引入 sie 模块
mod sie;
```

### 步骤 3: 修改 `src/main.rs` - 添加时间读取函数

在 `rust_main` 函数之前添加：

```rust
/// 读取当前时间 (mtime 寄存器)
///
/// RISC-V 中有一个内存映射的定时器 (mtime)，可以通过 rdtime 指令读取。
/// 返回值是以时钟 tick 为单位的当前时间。
///
/// QEMU virt 中，时钟频率通常是 10MHz (每微秒 10 个 tick)
#[inline]
fn rdtime() -> u64 {
    let mut time: u64;
    unsafe { core::arch::asm!("rdtime {}", out(reg) time) }
    time
}
```

### 步骤 4: 修改 `src/main.rs` - 替换 rust_main

将 `rust_main` 函数替换为：

```rust
/// S 态主函数：定时输出字符
///
/// 工作流程：
/// 1. 启用定时器中断
/// 2. 设置第一次定时器中断
/// 3. 打印启动消息
/// 4. 进入无限循环，等待定时器中断触发
extern "C" fn rust_main() -> ! {
    // ========== 步骤 1: 启用定时器中断 ==========
    unsafe { sie::set_stimer() };

    // ========== 步骤 2: 设置第一次定时器中断 ==========
    // 定时器间隔：10,000,000 时钟周期 ≈ 1 秒 (假设 10MHz 时钟)
    let interval = 10_000_000u64;
    let current_time = rdtime();
    set_timer(current_time + interval);

    // ========== 步骤 3: 打印启动消息 ==========
    for c in b"Timer started!\n" {
        console_putchar(*c);
    }

    // ========== 步骤 4: 无限循环等待中断 ==========
    // 使用 wfi (Wait For Interrupt) 指令进入低功耗等待
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
```

### 步骤 5: 修改 `src/main.rs` - 添加 trap handler

在 `rust_main` 函数之后添加：

```rust
/// S-mode trap handler (中断/异常处理函数)
///
/// 当发生定时器中断时，CPU 会自动跳转到 stvec 指向的地址。
/// 我们需要在 trap handler 中：
/// 1. 判断中断类型
/// 2. 处理定时器中断（输出字符 + 重新设置定时器）
/// 3. 返回原程序继续执行
#[unsafe(no_mangle)]
extern "C" fn s_trap_handler() {
    // ========== 步骤 1: 读取 scause，判断中断类型 ==========
    let scause: usize;
    unsafe { core::arch::asm!("csrr {0}, scause", out(reg) scause) }

    // scause 最高位为 1 表示中断，为 0 表示异常
    // 最低几位表示中断类型：
    //   - 5 (0b00101): S-mode 定时器中断 (STIP)
    //   - 1 (0b00001): S-mode 软件中断 (SSIP)
    //   - 9 (0b01001): S-mode 外部中断 (SEIP)

    let is_interrupt = (scause >> 63) != 0;
    let interrupt_code = scause & 0xFFFF_FFFF;

    if is_interrupt && interrupt_code == 5 {
        // ========== 步骤 2: 处理定时器中断 ==========

        // 输出字符 't'
        console_putchar(b't');

        // 重新设置下一次定时器中断
        let interval = 10_000_000u64;
        let current_time = rdtime();
        set_timer(current_time + interval);
    }

    // ========== 步骤 3: 清除挂起的定时器中断 ==========
    // 清除 sip 寄存器中的 STIP 位，避免重复触发
    unsafe {
        core::arch::asm!(
            "csrc sip, {0}",
            in(reg) (1 << 5), // Clear STIP
        );
    }
}
```

### 步骤 6: 修改 `src/main.rs` - 在 _start 中设置 stvec

修改 `_start` 函数，在设置栈之后跳转到 main 之前添加 stvec 设置：

```rust
#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 4096;

    #[unsafe(link_section = ".bss.uninit")]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    core::arch::naked_asm!(
        "la sp, {stack} + {stack_size}",  // 设置栈指针

        // ========== 设置 stvec 指向 trap handler ==========
        // stvec 是 S-mode 的陷阱向量寄存器
        // 最低位为 0 表示 Direct 模式：所有陷阱都跳转到 base 地址
        // 最低位为 1 表示 Vectored 模式：不同中断类型跳转到不同地址
        "la t0, s_trap_handler",           // 加载 trap handler 地址到 t0
        "csrw stvec, t0",                  // 设置 stvec

        "j  {main}",                       // 跳转到 rust_main

        stack_size = const STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
    )
}
```

---

## 常见困难及解决方案

### 困难 1: 不知道如何操作 CSR 寄存器

**问题**: RISC-V 的 `csrr`/`csrw`/`csrc`/`csrs` 指令在 Rust 中如何使用？

**原因**: 不熟悉 Rust 的内联汇编语法和 RISC-V 汇编指令

**解决方案**:

```rust
// 读取 CSR 寄存器 -> csrr destination, csr_register
let value: usize;
unsafe { core::arch::asm!("csrr {0}, sie", out(reg) value) }

// 写入 CSR 寄存器 -> csrw csr_register, source
unsafe { core::arch::asm!("csrw sie, {0}", in(reg) value) }

// 原子设置 CSR 位 -> csrs csr_register, mask (设置为 1)
unsafe { core::arch::asm!("csrs sie, {0}", in(reg) (1 << 5)) }

// 原子清除 CSR 位 -> csrc csr_register, mask (设置为 0)
unsafe { core::arch::asm!("csrc sip, {0}", in(reg) (1 << 5)) }

// 读取并修改的完整示例
unsafe {
    core::arch::asm!(
        "csrr t0, sie",    // 读取 sie 到 t0
        "ori t0, t0, {0}", // t0 = t0 | mask
        "csrw sie, t0",    // 写回 sie
        in(reg) (1 << 5)   // mask = 1 << 5 (STIE)
    );
}
```

### 困难 2: 不知道定时器中断的 scause 值

**问题**: 如何判断触发的是定时器中断而不是其他中断？

**原因**: 不了解 RISC-V 中断编号规范

**解决方案**:

| scause 值 | 含义 | 说明 |
|-----------|------|------|
| 0x8000000000000005 | S-mode 定时器中断 | 最高位 1 = 中断，低 5 位 5 = STIP |
| 0x8000000000000001 | S-mode 软件中断 | 最高位 1 = 中断，低 5 位 1 = SSIP |
| 0x8000000000000009 | S-mode 外部中断 | 最高位 1 = 中断，低 5 位 9 = SEIP |

简化判断方法：
```rust
let is_interrupt = (scause >> 63) != 0;  // 最高位为 1 表示中断
let interrupt_code = scause & 0xF;       // 低 4 位表示中断类型

if is_interrupt && interrupt_code == 5 {
    // 定时器中断
}
```

### 困难 3: trap handler 无法正常工作（寄存器保存问题）

**问题**: trap handler 执行后程序崩溃或行为异常

**原因**: 进入 trap 时，CPU 不会自动保存所有寄存器，需要手动保存/恢复

**解决方案**:

**方案 A: 使用简化版轮询方案（推荐初学者）**

不使用真正的 trap handler，而是在主循环中轮询检查时间：

```rust
extern "C" fn rust_main() -> ! {
    unsafe { sie::set_stimer() };

    let interval = 10_000_000u64;
    let mut next_tick = rdtime() + interval;
    set_timer(next_tick);

    for c in b"Timer started!\n" {
        console_putchar(*c);
    }

    loop {
        let current = rdtime();
        if current >= next_tick {
            console_putchar(b't');
            next_tick = current + interval;
            set_timer(next_tick);
        }
    }
}
```

**方案 B: 手动保存/恢复寄存器（正确但复杂）**

```rust
#[unsafe(no_mangle)]
extern "C" fn s_trap_handler() {
    // 保存寄存器
    core::arch::asm!(
        "addi sp, sp, -128",  // 分配栈空间
        "sd ra, 0(sp)",        // 保存 ra
        "sd t0, 8(sp)",        // 保存 t0
        // ... 保存其他需要的寄存器
    );

    // 处理中断
    let scause: usize;
    core::arch::asm!("csrr {0}, scause", out(reg) scause);
    if scause == 5 {
        console_putchar(b't');
        set_timer(rdtime() + 10_000_000);
    }

    // 恢复寄存器
    core::arch::asm!(
        "ld ra, 0(sp)",
        "ld t0, 8(sp)",
        "addi sp, sp, 128",
        "sret"  // 返回原程序
    );
}
```

### 困难 4: 不知道如何初始化定时器

**问题**: 定时器中断始终不触发

**原因**: 定时器初始化有多个步骤，缺一不可

**解决方案**:

必须完成以下 4 步：

```rust
fn init_timer() {
    // 步骤 1: 启用全局中断 (sstatus.SIE)
    unsafe {
        let mut sstatus: usize;
        core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus);
        sstatus |= 1 << 1;  // SIE 位
        core::arch::asm!("csrw sstatus, {0}", in(reg) sstatus);
    }

    // 步骤 2: 启用定时器中断 (sie.STIE)
    unsafe {
        let mut sie: usize;
        core::arch::asm!("csrr {0}, sie", out(reg) sie);
        sie |= 1 << 5;  // STIE 位
        core::arch::asm!("csrw sie, {0}", in(reg) sie);
    }

    // 步骤 3: 设置第一次定时器中断 (通过 SBI)
    let interval = 10_000_000u64;
    set_timer(rdtime() + interval);

    // 步骤 4: 设置 trap handler (stvec)
    // 在 _start 中完成
}
```

### 困难 5: `wfi` 指令不起作用

**问题**: 使用 `wfi` 进入低功耗等待后，中断触发但程序没有响应

**原因**: 可能没有正确设置中断或定时器

**解决方案**:

1. **调试步骤**：
   - 先使用轮询方案，确认定时器逻辑正确
   - 确认 `set_timer` 确实被调用了
   - 使用 `console_putchar` 输出调试信息

2. **检查清单**：
   ```rust
   fn debug_check() {
       // 检查 sstatus
       let sstatus: usize;
       unsafe { core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus) }
       console_putchar(if (sstatus & 0x2) != 0 { '1' } else { '0' });

       // 检查 sie
       let sie: usize;
       unsafe { core::arch::asm!("csrr {0}, sie", out(reg) sie) }
       console_putchar(if (sie & 0x20) != 0 { '1' } else { '0' });

       // 检查 mip
       let mip: usize;
       unsafe { core::arch::asm!("csrr {0}, mip", out(reg) mip) }
       console_putchar(if (mip & 0x20) != 0 { '1' } else { '0' });
   }
   ```

3. **如果中断仍然不工作**：使用轮询方案作为后备：
   ```rust
   loop {
       // 不用 wfi，直接轮询
       let current = rdtime();
       if current >= next_tick {
           console_putchar(b't');
           next_tick = current + interval;
           set_timer(next_tick);
       }
   }
   ```

---

## 验证方法

### 编译项目

```bash
cd /home/hdu/study/rust/2026s-ai4ose-lab/ai4ose-lab1-2026s/tg-rcore-tutorial/tg-rcore-tutorial-ch1-clock
cargo build
```

### 运行项目

```bash
cargo run
```

### 预期输出

```
Timer started!
tttttttttt...
```

应该看到 "Timer started!" 后，每隔约 1 秒输出一个 't' 字符。

### 常见问题排查

1. **没有任何输出**: 检查代码是否编译成功，确认 `cargo run` 运行的是正确的项目
2. **只输出 "Timer started!" 但没有 't'**: 定时器中断未触发，检查 sie/sstatus 设置
3. **输出乱码**: 可能是时序问题，检查 interval 值是否合理
