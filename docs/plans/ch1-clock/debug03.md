# tg-rcore-tutorial-ch1-clock mideleg 问题排查

## 问题描述

调试输出显示 `YYXXXXXXXXXXXXX`，说明：
- ✅ `sstatus.SIE` = 1 (第一个 Y)
- ✅ `sie.STIE` = 1 (第二个 Y)
- ❌ `mideleg.STIP` = 0 (缺少第三个 Y)

## 问题根因

**定时器中断 (STIP) 没有被委托到 S 态！**

### RISC-V 中断委托机制

RISC-V 使用 `mideleg` (Machine Interrupt Delegation) 寄存器将中断委托给 S 态处理。当 `mideleg.STIP = 1` 时，定时器中断会被送到 S 态处理（即触发 S 态 trap）；当 `mideleg.STIP = 0` 时，定时器中断仍然在 M 态处理。

### 当前问题分析

查看 `m_entry.asm` 第 26-27 行：
```asm
li t0, 0xffff
csrw mideleg, t0
```

理论上 `mideleg` 被设置为 `0xffff`，应该委托所有中断。但实际运行时 `mideleg.STIP = 0`，可能的原因：

1. **QEMU 配置覆盖**：QEMU 可能在启动时重置了 `mideleg`
2. **其他代码修改**：`tg-rcore-tutorial-sbi` 可能在其他地方修改了 `mideleg`
3. **SBI 调用影响**：某些 SBI 实现可能会修改中断委托设置

---

## 解决方案

### 方案 A: 在 S 态手动设置 mideleg（推荐）

在 `rust_main` 中，在启用定时器中断之前，手动设置 `mideleg.STIP = 1`：

```rust
extern "C" fn rust_main() -> ! {
    // 手动设置 mideleg，委托定时器中断到 S 态
    unsafe {
        let mut mideleg: usize;
        core::arch::asm!("csrr {0}, mideleg", out(reg) mideleg);
        mideleg |= 1 << 5;  // 设置 STIP 位
        core::arch::asm!("csrw mideleg, {0}", in(reg) mideleg);
    }

    // 验证设置
    let mideleg: usize;
    unsafe { core::arch::asm!("csrr {0}, mideleg", out(reg) mideleg) }
    console_putchar(if (mideleg & 0x20) != 0 { b'Y' } else { b'N' });  // 应该输出 Y

    // 启用定时器中断
    sie::set_stimer();

    // ... 原有代码
}
```

或者更简单的方式，直接设置而不读取：

```rust
// 直接设置 mideleg.STIP = 1，不读取原值
unsafe {
    core::arch::asm!("csrs mideleg, {0}", in(reg) (1 << 5));
}
```

### 方案 B: 修改 tg-rcore-tutorial-sbi 的 m_entry.asm

找到 `tg-rcore-tutorial-sbi/src/m_entry.asm`，确保 `mideleg` 的设置正确：

```asm
# 在 m_entry.asm 中确保 STIP 位被设置
li t0, 0xffff
csrw mideleg, t0
# 或者显式设置
li t0, 1 << 5
csrs mideleg, t0
```

---

## 验证步骤

1. 在 `rust_main` 开头添加 `mideleg` 设置代码：
   ```rust
   unsafe {
       core::arch::asm!("csrs mideleg, {0}", in(reg) (1 << 5));
   }
   ```

2. 重新编译运行

3. 预期输出：`YYYTime started!` （三个 Y 表示所有寄存器都正确设置）

---

## 完整修复代码

修改 `src/main.rs` 中的 `rust_main` 函数：

```rust
extern "C" fn rust_main() -> ! {
    // ========== 步骤 1: 手动设置 mideleg，委托定时器中断到 S 态 ==========
    // mideleg.STIP = 1 表示定时器中断委托给 S 态处理
    unsafe {
        core::arch::asm!("csrs mideleg, {0}", in(reg) (1 << 5));
    }

    // 调试：验证 mideleg 设置
    let mideleg: usize;
    unsafe { core::arch::asm!("csrr {0}, mideleg", out(reg) mideleg) }
    console_putchar(if (mideleg & 0x20) != 0 { b'Y' } else { b'N' });

    // ========== 步骤 2: 启用定时器中断 ==========
    sie::set_stimer();

    // 调试：检查 sstatus
    let sstatus: usize;
    unsafe { core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus) }
    console_putchar(if (sstatus & 0x2) != 0 { b'Y' } else { b'N' });

    // 调试：检查 sie
    let sie: usize;
    unsafe { core::arch::asm!("csrr {0}, sie", out(reg) sie) }
    console_putchar(if (sie & 0x20) != 0 { b'Y' } else { b'N' });

    // ========== 步骤 3: 设置第一次定时器中断 ==========
    let interval = 10_000_000u64;
    let current_time = rdtime();
    set_timer(current_time + interval);

    for c in b"Time started!\n" {
        console_putchar(*c);
    }

    // ========== 步骤 4: 无限循环等待中断 ==========
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
```

---

## 知识补充

### mideleg 寄存器位定义

| 位 | 名称 | 描述 |
|----|------|------|
| 0 | SSIP | S-mode 软件中断 |
| 1 | USIP | U-mode 软件中断 |
| 2 | MSTIP | M-mode 软件定时器中断 |
| 4 | SEIP | S-mode 外部中断 |
| 5 | STIP | S-mode 定时器中断 |
| 8 | UTIP | U-mode 定时器中断 |

注意：虽然 `m_entry.asm` 设置 `mideleg = 0xffff`，但某些环境下可能被覆盖，所以最好在 S 态再次设置。

---

## 检查清单

| 步骤 | 检查项 | 预期 | 命令/代码 |
|------|--------|------|-----------|
| 1 | 设置 mideleg.STIP | = 1 | `csrs mideleg, 0x20` |
| 2 | 验证 mideleg | = 1 | `csrr t0, mideleg; andi t0, t0, 0x20` |
| 3 | 设置 sstatus.SIE | = 1 | `sie::set_stimer()` |
| 4 | 设置 sie.STIE | = 1 | `sie::set_stimer()` |
| 5 | 设置定时器 | 已调用 | `set_timer(rdtime() + interval)` |
