# 第三章：多道程序与分时多任务

> 适配学习者：你已完成第一章和第二章的学习，掌握了裸机编程、批处理系统、Trap 处理和系统调用基础。本章将带你进入真正的多任务操作系统——实现多道程序和分时多任务。

## 学习目标

在本章结束时，你将能够：

1. **理解**多道程序系统与批处理系统的区别
2. **掌握**任务控制块（TCB）如何管理任务的状态和上下文
3. **理解**协作式调度与抢占式调度的原理和实现
4. **理解**时钟中断的工作机制和时间片轮转算法
5. **实现**新增的系统调用：`yield`（让出 CPU）和 `clock_gettime`（获取时间）

---

## 前置知识（第一章+第二章回顾 + 本章新增）

### 你已具备的知识（来自 ch1 + ch2）

| 概念 | 说明 |
|------|------|
| `#![no_std]` | 不使用 Rust 标准库 |
| `_start` 入口 | 裸机程序入口点 |
| SBI 调用 | 通过 `console_putchar` 输出字符 |
| RISC-V 特权级 | M-mode（固件）、S-mode（内核）、U-mode（用户） |
| 批处理系统 | 依次加载运行多个用户程序 |
| Trap 处理 | 用户态 `ecall` 触发系统调用 |
| 系统调用 | write（输出）和 exit（退出） |

### 本章新增概念

| 新概念 | 与 ch1/ch2/已学知识的联系 |
|--------|-------------------------|
| 多道程序系统 | ch2 批处理是串行，ch3 多道程序是并发 |
| 任务控制块（TCB） | ch2 无任务管理，ch3 新增 TCB 封装上下文/栈 |
| 协作式调度 | ch2 无任务切换，ch3 新增 yield 系统调用 |
| 抢占式调度 | ch2 无时钟中断，ch3 新增时钟中断强制切换 |
| 时间片轮转 | 调度算法 |
| clock_gettime | ch2 无时间系统调用，ch3 新增 |

---

## 项目结构

```
lbl-rcore-tutorial-ch3/
├── .cargo/
│   └── config.toml     # Cargo 配置：交叉编译目标和 QEMU runner
├── .gitignore           # Git 忽略规则
├── build.rs            # 构建脚本：下载编译用户程序
├── Cargo.toml          # 项目配置与依赖
├── LICENSE             # GPL v3 许可证
├── README.md           # 本文档
├── rust-toolchain.toml # Rust 工具链配置
├── test.sh             # 自动测试脚本
├── exercise.md         # 练习说明
└── src/
    ├── main.rs         # 内核源码：多道程序主循环、Trap 处理
    └── task.rs         # 任务控制块（TCB）和调度事件定义
```

---

## 一、环境准备

> 如果你已按照前两章配置好开发环境，直接进入下一步。

### 1.1 额外工具安装

本章的构建脚本需要 `cargo-clone` 和 `rust-objcopy`：

```bash
cargo install cargo-clone
cargo install cargo-binutils
rustup component add llvm-tools
```

---

## 二、编译与运行

### 2.1 编译

```bash
cd /home/hdu/study/rust/2026s-ai4ose-lab/ai4ose-lab1-2026s/lbl-rcore-tutorial/lbl-rcore-tutorial-ch3
cargo build
```

### 2.2 运行

**默认模式（抢占式调度）：**

```bash
cargo run
```

**协作式调度模式：**

```bash
cargo run --features coop
```

启用 `coop` feature 后，禁用时钟中断抢占，任务只能通过 `yield` 主动让出 CPU。

**练习模式：**

```bash
cargo run --features exercise
```

### 2.3 预期输出

```
[tg-rcore-tutorial-ch3 0.3.0-preview.1] Hello, world!
[ INFO] .data [0x802xxxxx, 0x802xxxxx)
[ WARN] boot_stack top=bottom=0x802xxxxx, lower_bound=0x802xxxxx
[ERROR] .bss [0x802xxxxx, 0x802xxxxx)
[ INFO] load app0 to 0x802xxxxx
[ INFO] load app1 to 0x802xxxxx
[ INFO] load app2 to 0x802xxxxx
...

power_3 [10000/200000]
power_3 [20000/200000]
...
power_3 [200000/200000]
3^200000 = 871008973(MOD 998244353)
Test power_3 OK!
...
AAAAAAAAAA [1/5]
BBBBBBBBBB [1/5]
CCCCCCCCCC [1/5]
...（交替输出，体现时间片轮转）
```

与第二章的串行输出不同，你会观察到多个用户程序的输出交替出现（如 power_3、power_5、write_a、write_b 交错），这就是抢占式调度的效果——时钟中断强制切换任务，实现了时间片轮转。

### 2.4 测试验证

```bash
./test.sh           # 运行全部测试
./test.sh base      # 仅运行基础测试
./test.sh exercise  # 仅运行练习测试
```

---

## 三、核心概念

### 3.1 从批处理到多道程序

> 这正是你熟悉的操作系统理论中的"多道程序系统"！

**第二章的批处理系统**串行执行用户程序：一个程序运行完毕后才加载下一个。这种方式的缺点是：当一个程序等待 I/O 或主动暂停时，CPU 处于空闲状态，造成资源浪费。

**多道程序系统**（Multiprogramming）解决了这个问题：

```
批处理系统                          多道程序系统
┌──────────────────┐               ┌──────────────────┐
│ App0 ██████████  │               │ App0 ██  ██  ██  │
│ App1     ████████│               │ App1   ██  ██  ██│
│ App2         ████│               │ App2  ██  ██  ██ │
│      ──────→ 时间│               │      ──────→ 时间 │
│  串行执行，CPU   │               │  交替执行，CPU     │
│  利用率低        │               │  利用率高         │
└──────────────────┘               └──────────────────┘
```

核心改进：
- **一次性加载**：所有用户程序在启动时同时加载到内存，减少切换开销
- **任务切换**：内核可以在多个任务之间快速切换，保证每个任务都能得到执行
- **调度算法**：决定何时切换、切换到哪个任务

### 3.2 任务控制块（TCB）

> 这类似于你熟悉的数据结构中的"链表节点"或"结构体"——封装了任务的所有信息。

**任务控制块**（Task Control Block, TCB）是内核管理任务的核心数据结构。在 lbl-rcore-tutorial-ch3 中，每个 TCB 包含：

| 字段 | 类型 | 说明 |
|------|------|------|
| `ctx` | `LocalContext` | 用户态上下文（所有通用寄存器 + CSR） |
| `finish` | `bool` | 任务是否已完成 |
| `stack` | `[usize; 1024]` | 独立的用户栈（8 KiB） |

与第二章相比，本章将用户上下文和栈空间封装到 TCB 中，使得多个任务可以独立管理，互不干扰。

**任务状态变化：**

```
          init()
  [未初始化] ──→ [就绪]
                   │
           execute()│
                   ▼
                [运行中]
               ╱   │    ╱
         yield/  exit/   异常/
         超时   退出   被杀死
             ╱     │      ╱
           ▼      ▼      ▼
        [就绪] [已完成] [已完成]
```

### 3.3 任务切换机制

任务切换是操作系统的核心机制。ch3 使用 `LocalContext` 实现：

1. **保存当前任务上下文**：将所有用户寄存器保存到当前 TCB 的 `ctx` 中
2. **恢复目标任务上下文**：从目标 TCB 的 `ctx` 中恢复用户寄存器
3. **切换执行**：通过 `sret` 指令返回到目标任务的用户态

```
当前任务（App A）                    下一个任务（App B）
     │                                    ▲
     ▼                                    │
  触发 Trap                           sret 返回
     │                                    ▲
     ▼                                    │
  保存 A 的上下文到 TCB[A]     恢复 B 的上下文从 TCB[B]
     │                                    ▲
     └──────── 内核调度决策 ───────────────┘
```

与第二章的 Trap 处理相比，本章增加了"不结束当前任务但切换到下一个"的逻辑。

### 3.4 协作式调度（yield）

> 这正是你熟悉的操作系统理论中的"协作式调度"！

**协作式调度**依赖任务主动让出 CPU。用户程序调用 `yield` 系统调用，告诉内核"我暂时不需要 CPU 了，可以去执行别的任务"。

**典型使用场景**：当程序需要等待外设完成 I/O 操作时，与其忙等浪费 CPU 时间，不如 yield 让出 CPU 给其他任务。

```
App A 发起 I/O 请求                App B 在运行
     │                                 │
     ├─ 调用 yield                     │
     │   （ecall，a7=124）             │
     │                                 │
     ▼                                 ▼
  内核处理：                        继续执行
  标记 A 为"就绪"                       │
  切换到 B                             │
     │                                 │
     ...（一段时间后轮转回 A）...       │
     │                                 │
     ▼                                 │
  A 继续执行                           │
  检查 I/O 是否完成                    │
```

启用 `coop` feature 可以体验纯协作式调度——时钟中断被禁用，任务只能通过 yield 主动让出 CPU。

### 3.5 抢占式调度（时钟中断）

> 这类似于你学过的计算机组成原理中的"定时器中断"！

**协作式调度的问题**：如果一个任务永远不调用 yield（例如进入死循环），其他任务就永远得不到执行。

**抢占式调度**通过时钟中断解决这个问题：

```
App A 正在执行（可能是死循环）
     │
     │  ← 12500 个时钟周期后
     │     时钟中断触发！
     ▼
  硬件自动陷入 S-mode
     │
     ▼
  scause = Interrupt::SupervisorTimer
     │
     ▼
  内核处理：
  1. 清除时钟中断（set_timer(u64::MAX)）
  2. 切换到下一个任务
     │
     ▼
  App B 开始执行
```

### 3.6 时间片轮转算法

ch3 使用最简单的轮转算法：维护一个任务索引 `i`，每次时钟中断后 `i = (i + 1) % n`，循环执行各任务。每个任务获得相等的时间片（12500 个时钟周期 ≈ 1ms）。

### 3.7 时钟中断的实现

RISC-V 的时钟中断机制：

| 组件 | 说明 |
|------|------|
| `mtime` 寄存器 | 硬件计数器，持续递增 |
| `mtimecmp` 寄存器 | 比较值，当 `mtime >= mtimecmp` 时触发中断 |
| `sie.stie` | S 特权级时钟中断使能位 |
| `set_timer()` | 通过 SBI 调用设置 `mtimecmp` |

初始化步骤：
1. `unsafe { sie::set_stimer() }` —— 开启 S 特权级时钟中断
2. 每次执行用户程序前：`set_timer(time::read64() + interval)` —— 设置下次中断时间

---

## 四、系统调用

ch3 在第二章的基础上新增了 `yield` 和 `clock_gettime` 两个系统调用：

| syscall ID | 名称 | 功能 |
|-----------|------|------|
| 64 | `write` | 将缓冲区数据写入文件描述符（fd=1 为标准输出） |
| 93 | `exit` | 退出当前任务 |
| 124 | `sched_yield` | 主动让出 CPU，切换到下一个任务 |
| 113 | `clock_gettime` | 获取当前时间（纳秒精度） |
| 410 | `trace` | 追踪系统调用信息（**练习题**，需自行实现） |

---

## 五、代码解读

### 5.1 src/task.rs —— 任务管理

**`TaskControlBlock`**：任务控制块
- `init(entry)` —— 创建用户态上下文，分配独立用户栈
- `execute()` —— 切换到 U-mode 执行
- `handle_syscall()` —— 处理系统调用并返回调度事件

**`SchedulingEvent`**：调度事件枚举
- `None` / `Yield` / `Exit(code)` / `UnsupportedSyscall(id)`

### 5.2 src/main.rs —— 内核主体

核心的多道程序循环：

```rust
// 初始化 → 加载所有应用到 TCB 数组
// → 开启时钟中断
// → 轮转执行：
while remain > 0 {
    if !tcb.finish {
        set_timer(...);        // 设置时间片
        tcb.execute();         // 切换到 U-mode
        match scause {
            Timer     → 切换到下一个任务
            UserEnvCall → 处理系统调用
            Exception → 杀死任务
        }
    }
    i = (i + 1) % n;         // 轮转到下一个
}
shutdown()
```

---

## 六、Rust 语法快速查阅（扩展）

### 6.1 enum（枚举）

```rust
// 枚举类型：表示多种可能的值
enum SchedulingEvent {
    None,                    // 继续执行当前任务
    Yield,                   // 切换到下一个任务
    Exit(usize),             // 任务退出，携带退出码
    UnsupportedSyscall(SyscallId), // 不支持的系统调用
}
```

### 6.2 struct（结构体）

```rust
// 结构体：封装多个字段
struct TaskControlBlock {
    ctx: LocalContext,       // 用户上下文
    finish: bool,            // 任务是否完成
    stack: [usize; 1024],   // 用户栈
}
```

### 6.3 impl 块

```rust
// 为结构体实现方法
impl TaskControlBlock {
    // 构造函数
    pub fn init(entry: usize) -> Self { ... }

    // 执行任务
    pub fn execute(&mut self) { ... }
}
```

---

## 七、编程练习

### 7.1 题目：实现 sys_trace

在 ch3 中实现 `sys_trace`（ID 为 410）用来追踪系统调用历史。

**功能：**

| trace_request | 功能 |
|--------------|------|
| 0 | 读取用户内存 |
| 1 | 写入用户内存 |
| 2 | 查询系统调用计数 |

### 7.2 运行练习

```bash
cargo run --features exercise
./test.sh exercise
```

---

## 八、本章小结

通过本章的学习和实践，你在第二章的基础上实现了重要的进化：

1. **从串行到并发**：批处理系统一次只运行一个程序，多道程序系统让多个程序交替执行
2. **任务控制块（TCB）**：封装任务的上下文、状态和栈空间
3. **协作式调度**：任务通过 `yield` 主动让出 CPU
4. **抢占式调度**：时钟中断强制切换任务
5. **时间片轮转**：最基本的调度算法
6. **时间管理**：`clock_gettime` 让用户程序获取系统时间

在后续章节中，我们将引入**地址空间**，为每个任务提供独立的虚拟内存。

---

## 九、思考题

1. **协作式 vs 抢占式调度的权衡？** 协作式调度的优点和缺点分别是什么？

2. **时间片大小的影响？** 如果把时间片设得非常大（如 1 秒），系统行为会如何变化？

3. **为什么需要 `SchedulingEvent` 枚举？** 如果不用枚举，直接在 `handle_syscall` 中决定是否切换任务，会有什么设计问题？

4. **时钟中断和 `sstatus.sie` 的关系？** 在 Trap 处理过程中，时钟中断会被屏蔽吗？

---

## 参考资料

- [rCore-Tutorial-Guide 第三章](https://learningos.github.io/rCore-Tutorial-Guide/)
- [rCore-Tutorial-Book 第三章](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/index.html)
- [RISC-V Privileged Specification](https://riscv.org/specifications/privileged-isa/)
- [RISC-V Reader 中文版](http://riscvbook.com/chinese/RISC-V-Reader-Chinese-v2p1.pdf)

---

## 依赖

| 依赖 | 说明 |
|------|------|
| `tg-rcore-tutorial-sbi` | SBI 调用封装库（含 set_timer） |
| `tg-rcore-tutorial-linker` | 链接脚本生成 |
| `tg-rcore-tutorial-console` | 控制台输出和日志 |
| `tg-rcore-tutorial-kernel-context` | 用户上下文管理 |
| `tg-rcore-tutorial-syscall` | 系统调用定义与分发 |
| `riscv` | RISC-V CSR 寄存器访问 |

---

## License

Licensed under GNU GENERAL PUBLIC LICENSE, Version 3.0.
