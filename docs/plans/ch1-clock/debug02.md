# tg-rcore-tutorial-ch1-clock Trap Handler 不触发排查

## 问题描述

Trap handler 没有被调用，即在输出 `Time started!` 后，既没有看到调试字符 `X`，也没有看到 `t`。

---

## 可能原因分析

### 原因 1: 定时器中断没有被正确触发

**分析**: 定时器中断触发需要以下条件：
1. `sstatus.SIE` 位 = 1（启用 S 态全局中断）
2. `sie.STIE` 位 = 1（启用 S 态定时器中断）
3. `set_timer` 被调用，且定时器时间已过期
4. 定时器中断被委托给 S 态（`mideleg.STIP` = 1）

**检查方法**: 在 `rust_main` 中添加调试输出检查寄存器值：

```rust
extern "C" fn rust_main() -> ! {
    // 启用定时器中断
    sie::set_stimer();

    // 调试：检查 sstatus
    let sstatus: usize;
    unsafe { core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus) }
    console_putchar(if (sstatus & 0x2) != 0 { 'Y' } else { 'N' });  // Y = OK, N = SIE not set

    // 调试：检查 sie
    let sie: usize;
    unsafe { core::arch::asm!("csrr {0}, sie", out(reg) sie) }
    console_putchar(if (sie & 0x20) != 0 { 'Y' } else { 'N' });  // Y = OK, N = STIE not set

    // 调试：检查 mideleg（中断委托）
    let mideleg: usize;
    unsafe { core::arch::asm!("csrr {0}, mideleg", out(reg) mideleg) }
    console_putchar(if (mideleg & 0x20) != 0 { 'Y' } else { 'N' });  // Y = OK, N = STIP not delegated

    // ... 原有代码
}
```

预期输出：`YYY` 表示中断配置正确。

---

### 原因 2: `s_trap_handler` 不是裸函数，无法正确处理 trap

**分析**: 当 trap 发生时，CPU 会跳转到 `stvec` 指向的地址。但是：
- 当前的 `s_trap_handler` 是一个**普通函数**（不是 naked 函数）
- 进入 trap 时，CPU 不会自动保存寄存器状态
- Rust 函数调用会尝试使用栈，但此时栈可能没有正确设置

**关键问题**: 在 `_start` 中设置 `stvec` 后，`s_trap_handler` 作为普通函数，会生成函数序言（prologue），尝试使用栈。但如果 trap 发生在 `rust_main` 之前的某个时刻，栈可能还没有准备好。

**解决方案**: 将 `s_trap_handler` 改为裸函数（naked function），完全用汇编编写。

---

### 原因 3: 中断委托问题 - 定时器中断没有被委托到 S 态

**分析**: 查看 `m_entry.asm`，发现 `mideleg` 被设置为 `0xffff`，这应该委托了所有中断。但需要确认。

**解决方案**: 检查 `mideleg` 寄存器的值：

```rust
// 在 rust_main 中添加
let mideleg: usize;
unsafe { core::arch::asm!("csrr {0}, mideleg", out(reg) mideleg) }
console_putchar(((mideleg >> 20) & 1) as u8 + b'0');  // 检查 STIP 位 (bit 5)
```

---

### 原因 4: `stvec` 设置时机问题

**分析**: 在 `_start` 中设置 `stvec`，但 trap handler 函数的地址可能还没有被正确链接。

**解决方案**: 检查 `stvec` 的值：

```rust
// 在 rust_main 开头添加
let stvec: usize;
unsafe { core::arch::asm!("csrr {0}, stvec", out(reg) stvec) }
// 输出 stvec 的低 32 位
console_putchar((stvec & 0xFF) as u8);
console_putchar(((stvec >> 8) & 0xFF) as u8);
```

---

## 解决方案

### 方案 A: 使用裸函数实现 trap handler（推荐）

将整个 trap handler 用汇编实现，确保在 trap 发生时能够正确执行：

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
        "sd a0, 24(sp)",

        // 调试：输出 X 确认进入 trap
        "li a0, 88",  // 'X'
        "call console_putchar",

        // 读取 scause
        "csrr t0, scause",

        // 检查是否是定时器中断 (scause == 5，需要带中断标志位)
        // 中断标志位在 bit 63，scause = 0x8000000000000005
        "li t1, 5",
        "bne t0, t1, .Lnot_timer",

        // 是定时器中断：输出 't'
        "li a0, 116",  // 't'
        "call console_putchar",

        // 重新设置定时器
        "rdtime a0",
        "li a1, 10000000",  // 10M
        "add a0, a0, a1",
        "call set_timer",

        // 清除挂起的定时器中断
        "csrc sip, {0}",
        in(reg) (1 << 5),

        ".Lnot_timer:",

        // 恢复寄存器
        "ld ra, 0(sp)",
        "ld t0, 8(sp)",
        "ld t1, 16(sp)",
        "ld a0, 24(sp)",
        "addi sp, sp, 64",

        // 返回
        "sret",
    );
}
```

### 方案 B: 在 trap handler 中不调用 Rust 函数

当前问题：`console_putchar` 和 `set_timer` 是 Rust 函数调用，它们需要栈和 C runtime。trap 发生时，这些可能没有准备好。

**解决方案**: 完全使用内联汇编实现 trap handler。

### 方案 C: 简化方案 - 轮询

如果中断方式太复杂，可以使用轮询方案（见 debug01.md）。

---

## 关键发现

经过分析，最可能的原因是：

**`s_trap_handler` 是普通函数而非裸函数**，当 trap 发生时：
1. CPU 跳转到 `stvec` 指向的地址
2. 但 Rust 函数会生成 prologue，尝试保存旧寄存器、建立新栈帧
3. 这会破坏 trap 发生时的寄存器状态
4. 而且，普通函数的返回地址是 `ra` 寄存器，但 trap 发生时 `ra` 没有被保存
5. 执行完毕后，函数返回到一个错误的位置（因为 `ra` 没有被正确设置）

**必须使用裸函数（naked function）或纯汇编来实现 trap handler**。

---

## 验证步骤

1. 在 `rust_main` 开头添加寄存器检查调试输出
2. 如果寄存器都正确设置，但 trap handler 仍不触发，使用方案 A 改写 trap handler
3. 如果不想用复杂的中断方式，使用轮询方案

---

## 快速检查清单

| 检查项 | 预期 | 检查位置 |
|--------|------|----------|
| sstatus.SIE | 1 | `sie::set_stimer()` 后 |
| sie.STIE | 1 | `sie::set_stimer()` 后 |
| mideleg.STIP | 1 | `m_entry.asm` 中设置 |
| stvec | 指向 `s_trap_handler` | `_start` 中设置 |
| trap handler 类型 | naked function | 函数定义 |
