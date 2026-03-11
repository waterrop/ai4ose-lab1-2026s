# tg-rcore-tutorial-ch1-clock 实现计划

## Context

用户希望在 `tg-rcore-tutorial-ch1` 的基础上创建一个新内核 `tg-rcore-tutorial-ch1-clock`，功能是定时输出字符。

### 现有项目结构

`tg-rcore-tutorial-ch1` 位于 `/home/hdu/study/rust/2026s-ai4ose-lab/ai4ose-lab1-2026s/tg-rcore-tutorial/tg-rcore-tutorial-ch1/`，包含：
- `.cargo/config.toml` - QEMU 运行配置
- `build.rs` - 构建脚本
- `Cargo.toml` - 项目配置，依赖 `tg-sbi`
- `src/main.rs` - 入口代码
- `rust-toolchain.toml` - Rust 工具链配置

### 关键依赖

`tg-sbi` 库（位于 `tg-rcore-tutorial-sbi`）提供了：
- `console_putchar(c: u8)` - 输出字符
- `set_timer(timer: u64)` - 设置定时器中断（设置 CLINT 的 stimecmp）
- `shutdown(failure: bool)` - 关机

## Plan

### 步骤 1: 创建新目录结构

```
tg-rcore-tutorial-ch1-clock/
├── .cargo/
│   └── config.toml           # 复制自 ch1
├── build.rs                  # 复制自 ch1
├── Cargo.toml                # 复制自 ch1（name 改为 ch1-clock）
├── rust-toolchain.toml       # 复制自 ch1
└── src/
    └── main.rs              # 修改，添加定时器功能
```

### 步骤 2: 修改 Cargo.toml

将 `name` 从 `tg-rcore-tutorial-ch1` 改为 `tg-rcore-tutorial-ch1-clock`。

### 步骤 3: 修改 src/main.rs

实现定时输出字符功能：

1. **读取时间的内联汇编函数**
   ```rust
   #[inline]
   fn time() -> u64 {
       let mut time: u64;
       unsafe { asm!("rdtime {}", out(reg) time) }
       time
   }
   ```

2. **设置 S-mode trap handler**
   - 配置 `stvec` CSR 指向 trap handler
   - 配置 `sie` 启用定时器中断 (STIE)
   - 配置 `sstatus` 启用全局中断 (SIE)

3. **定时器中断处理函数**
   - 识别定时器中断（scause == 5）
   - 输出字符
   - 重新设置下一次定时器中断

4. **主循环**
   - 初始化定时器
   - 进入循环（或直接返回，等待定时器中断触发）

### 步骤 4: 运行测试

```bash
cd tg-rcore-tutorial-ch1-clock
cargo run
```

预期效果：QEMU 启动后，每隔固定时间（如 1 秒）输出一个字符（如 "t"），形成无限循环。

## 关键实现细节

### RISC-V S-mode 定时器中断设置

1. **启用 S-mode 定时器中断**：
   ```rust
   // 启用全局中断
   sstatus |= 0x2;
   // 启用定时器中断
   sie |= 0x20;
   ```

2. **设置定时器**（使用 SBI）：
   ```rust
   let interval = 10_000_000; // 10M 时钟周期（约1秒）
   set_timer(time() + interval);
   ```

3. **Trap 处理**：
   - scause = 5 表示 S-mode 定时器中断
   - 处理完后需要重新设置定时器

### 可复用的现有函数

- `tg_sbi::console_putchar()` - 输出字符
- `tg_sbi::set_timer()` - 设置定时器（通过 SBI）

## 验证方法

1. 运行 `cargo run` 启动 QEMU
2. 观察输出：每隔固定时间应看到字符输出
3. 可以使用 Ctrl+C 终止 QEMU
