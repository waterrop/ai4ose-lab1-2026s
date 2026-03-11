# tg-rcore-tutorial-ch1-clock Bug 排查方案

## 问题描述

代码可以正常编译运行，输出 `Time started!` 后，但没有 `t` 字符输出。

这说明：
- ✅ 定时器已设置 (`set_timer` 被调用)
- ✅ 中断已启用 (`sie::set_stimer()` 被调用)
- ❌ 定时器中断没有触发，或者中断处理函数没有正确执行

---

## 可能原因分析

### 原因 1: trap handler 缺少 `sret` 指令（最可能）

**问题**: `s_trap_handler` 函数执行完毕后没有返回到原来的代码位置。

**分析**: 查看当前代码，`s_trap_handler` 是一个普通函数，执行完后会返回到 `loop` 中的 `wfi` 之后继续执行，而不是返回到中断发生的位置。需要使用 `sret` 指令显式返回。

### 原因 2: trap handler 没有保存/恢复寄存器

**问题**: 进入 trap 时，CPU 不会自动保存所有寄存器，而当前代码直接使用寄存器可能导致数据丢失或程序崩溃。

**分析**: 参考 `m_entry.asm` 中的 trap handler，它会保存和恢复寄存器。

### 原因 3: 中断委托问题

**问题**: RISC-V 可以将中断委托给 S 态处理，但需要确保定时器中断被正确委托。

**分析**: 从 `m_entry.asm` 可以看到 `mideleg` 被设置为 `0xffff`，这意味着大部分中断已委托给 S 态。

---

## 排查步骤

### 步骤 1: 添加调试输出，验证 trap handler 是否被调用

在 `s_trap_handler` 入口处添加一个字符输出，观察是否触发：

```rust
#[unsafe(no_malloc)]
extern "C" fn s_trap_handler(){
    // 添加这一行调试输出
    console_putchar(b'X');  // 如果看到 X，说明 trap handler 被调用了

    // ... 原有代码
}
```

### 步骤 2: 检查 sstatus 和 sie 寄存器的值

在 `rust_main` 中添加调试代码：

```rust
extern "C" fn rust_main() -> ! {
    sie::set_stimer();

    // 调试：检查 sstatus
    let sstatus: usize;
    unsafe { core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus) }
    console_putchar(if (sstatus & 0x2) != 0 { '1' } else { '0' });  // 应该输出 '1'

    // 调试：检查 sie
    let sie: usize;
    unsafe { core::arch::asm!("csrr {0}, sie", out(reg) sie) }
    console_putchar(if (sie & 0x20) != 0 { '1' } else { '0' });  // 应该输出 '1'

    // 调试：检查当前时间
    let t = rdtime();
    console_putchar(((t >> 48) & 0xFF) as u8);  // 输出高位字节用于调试
    // ... 原有代码
}
```

### 步骤 3: 检查 mip 寄存器（定时器中断挂起状态）

在 `rust_main` 循环中添加：

```rust
loop {
    let mip: usize;
    unsafe { core::arch::asm!("csrr {0}, mip", out(reg) mip) }
    if (mip & 0x20) != 0 {
        console_putchar(b'M');  // 如果输出 M，说明定时器中断已挂起
    }
    unsafe{
        core::arch::asm!("wfi");
    }
}
```

---

## 解决方案

### 方案 A: 使用轮询方式（最简单，推荐）

不使用真正的中断处理，而是轮询检查时间：

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

### 方案 B: 正确实现 trap handler（需要保存/恢复寄存器 + sret）

修改 `src/main.rs`：

```rust
/// S-mode trap handler
///
/// 关键：必须保存寄存器并使用 sret 返回
#[unsafe(no_mangle)]
extern "C" fn s_trap_handler() {
    // 保存寄存器（在栈上分配空间）
    core::arch::asm!(
        "addi sp, sp, -64",  // 分配栈空间
        "sd ra, 0(sp)",      // 保存 ra
        "sd t0, 8(sp)",      // 保存 t0
        "sd t1, 16(sp)",     // 保存 t1
        "sd a0, 24(sp)",     // 保存 a0
    );

    // 读取 scause
    let scause: usize;
    unsafe { core::arch::asm!("csrr {0}, scause", out(reg) scause) }

    if scause == 0x8000000000000005 {
        // 处理定时器中断
        console_putchar(b't');

        let interval = 10_000_000u64;
        let current_time = rdtime();
        set_timer(current_time + interval);
    }

    // 清除挂起的定时器中断
    unsafe {
        core::arch::asm!("csrc sip, {0}", in(reg) (1 << 5));
    }

    // 恢复寄存器
    core::arch::asm!(
        "ld ra, 0(sp)",
        "ld t0, 8(sp)",
        "ld t1, 16(sp)",
        "ld a0, 24(sp)",
        "addi sp, sp, 64",   // 释放栈空间
        "sret",              // 返回到中断发生的位置
    );
}
```

### 方案 C: 使用裸函数（naked function）实现 trap handler

```rust
#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn s_trap_handler() {
    core::arch::naked_asm!(
        // 保存寄存器
        "addi sp, sp, -64",
        "sd ra, 0(sp)",
        "sd t0, 8(sp)",
        "sd t1, 16(sp)",

        // 读取 scause 到 t0
        "csrr t0, scause",

        // 检查是否是定时器中断 (scause == 5，带中断标志)
        "li t1, 5",
        "bne t0, t1, .Lskip",

        // 处理定时器中断
        "addi a0, zero, 116",  // 't' 的 ASCII
        "call console_putchar",

        // 重新设置定时器
        "rdtime a0",
        "addi a1, zero, 100",  // 简化：直接加固定值
        "mul a1, a1, 100000",
        "add a0, a0, a1",
        "call set_timer",

        // 清除挂起的定时器中断
        "csrc sip, {0}",
        in(reg) (1 << 5),

        ".Lskip:",

        // 恢复寄存器
        "ld ra, 0(sp)",
        "ld t0, 8(sp)",
        "ld t1, 16(sp)",
        "addi sp, sp, 64",

        // 返回
        "sret",
    );
}
```

---

## 快速验证清单

| 检查项 | 预期结果 | 检查方法 |
|--------|----------|----------|
| sstatus SIE 位 | 1 | 添加调试输出 `console_putchar(if (sstatus & 2) != 0 { '1' } else { '0' })` |
| sie STIE 位 | 1 | 添加调试输出 |
| trap handler 被调用 | 看到 'X' | 在 handler 入口添加输出 |
| 时间是否过期 | 当前时间 > 设置时间 | 在循环中输出当前时间 |

---

## 推荐修复顺序

1. **首先**: 使用方案 A（轮询方式）快速验证功能是否正常
2. **然后**: 如果轮询方式工作但中断方式不工作，使用方案 B 或 C 修复 trap handler
3. **关键点**: 确认 trap handler 中有 `sret` 指令

---

## 常见错误

### 错误 1: 忘记加 `sret`

```rust
// ❌ 错误：函数结束后不知道返回到哪里
#[unsafe(no_malloc)]
extern "C" fn s_trap_handler(){
    // 处理代码
}

// ✅ 正确：使用 sret 返回
#[unsafe(no_malloc)]
extern "C" fn s_trap_handler(){
    // 处理代码
    unsafe { core::arch::asm!("sret"); }
}
```

### 错误 2: scause 值判断错误

```rust
// ❌ 错误：只判断低 5 位，忽略中断标志位
if scause == 5 { ... }

// ✅ 正确：scause = 0x8000000000000005（最高位为 1 表示中断）
if scause == 0x8000000000000005 { ... }

// 或者
if (scause >> 63) != 0 && (scause & 0xF) == 5 { ... }
```

### 错误 3: 没有清除挂起的中断

```rust
// ✅ 正确：清除 sip 中的 STIP 位
unsafe {
    core::arch::asm!("csrc sip, {0}", in(reg) (1 << 5));
}
```
