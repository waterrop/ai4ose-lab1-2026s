# ch3编码
基于文档的要求，给我“最小可通过测例”的实现路线图。
要求：
1. 按文件、函数、改动点列出；
2. 每步说明输入/输出与预期日志；
3. 不一次性给全量代码，只给当前步需要的核心片段。
4. 使用rust

## 文档内容
引入一个新的系统调用 sys_trace（ID 为 410）用来追踪当前任务系统调用的历史信息，并做对应的修改。定义如下：
fn trace(&self, _caller: tg_syscall::Caller, _trace_request: usize, _id: usize, _data: usize) -> isize
调用规范：
这个系统调用有三种功能，根据 trace_request 的值不同，执行不同的操作：
如果 trace_request 为 0，则 id 应被视作 *const u8，表示读取当前任务 id 地址处一个字节的无符号整数值。此时应忽略 data 参数。返回值为 id 地址处的值。
如果 trace_request 为 1，则 id 应被视作 *mut u8，表示写入 data（作为 u8，即只考虑最低位的一个字节）到该用户程序 id 地址处。返回值应为 0。
如果 trace_request 为 2，表示查询当前任务调用编号为 id 的系统调用的次数，返回值为这个调用次数。本次调用也计入统计。
否则，忽略其他参数，返回值为 -1。

## TCB
pub struct TaskControlBlock {
    /// PID
    // pub pid: usize,
    /// 用户态上下文：保存 Trap 时的所有寄存器状态
    ctx: LocalContext,
    /// 任务完成标志：true 表示已退出或被杀死
    pub finish: bool,
    /// 用户栈：8 KiB（1024 个 usize = 1024 × 8 = 8192 字节）
    /// 每个任务拥有独立的栈空间，避免栈溢出影响其他任务
    stack: [usize; 1024],
    /// 用于记录系统调用的次数
    syscnt: [usize; 512],
}

    pub fn handle_syscall(&mut self) -> SchedulingEvent {
        use tg_syscall::{SyscallId as Id, SyscallResult as Ret};
        use SchedulingEvent as Event;
        // a7 寄存器存放 syscall ID
        let id = self.ctx.a(7).into();
        let sysid = tg_syscall::SyscallId(id);
        self.update_syscnt(id);
        // a0-a5 寄存器存放系统调用参数
        let args = [
            self.ctx.a(0),
            self.ctx.a(1),
            self.ctx.a(2),
            self.ctx.a(3),
            self.ctx.a(4),
            self.ctx.a(5),
        ];
        match tg_syscall::handle(Caller { entity: 0, flow: self.get_cnt(id) }, sysid, args) {
            Ret::Done(ret) => match sysid {
                // exit 系统调用：返回退出事件
                Id::EXIT => Event::Exit(self.ctx.a(0)),
                // yield 系统调用：返回让出事件
                Id::SCHED_YIELD => {
                    *self.ctx.a_mut(0) = ret as _;
                    self.ctx.move_next(); // sepc += 4，跳过 ecall 指令
                    Event::Yield
                }
                // 其他系统调用（如 write、clock_gettime）：继续执行
                _ => {
                    *self.ctx.a_mut(0) = ret as _;
                    self.ctx.move_next(); // sepc += 4，跳过 ecall 指令
                    Event::None
                }
            },
            // 不支持的系统调用
            Ret::Unsupported(_) => Event::UnsupportedSyscall(sysid),
        }
    }
}


# 演讲稿
这是我的讲解稿：
```
# 从批处理到分时：任务切换与上下文保存的最小闭环

批处理系统是串行执行任务的，即这个任务执行完才能执行下一个任务，这就会导致一个问题——当现在在执行的任务请求I/O时，并没有使用CPU，这时候CPU就是空闲状态，会导致CPU的浪费，所以有没有一种办法可以让任务不用CPU的时候让出CPU让其他任务执行呢？——这就是任务调度。

确定了任务调度后，又出现了新的问题：
1. 什么时候进行任务调度呢？
2. 让出CPU后，如何选择下一个该执行的任务呢？

## 一、什么时候进行任务调度呢？

有两个选择的方式，一是时钟中断方式，也就是抢占式调度；二是任务主动让出CPU，也就是协作式调度。

## 二、让出CPU后，如何选择下一个该执行的任务呢？

有很多调度算法，比如先来先服务、短作业优先、优先级等。

## 三、如何实现任务调度？

想实现任务调度，就需要更好的管理任务，所以我们定义一个任务控制块（Task Control Block, TCB）用来管理任务，TCB是内核管理任务的核心数据结构。
在 tg-ch3 中，每个 TCB 包含：
```
| 字段 | 类型 | 说明 |
|------|------|------|
| `ctx` | `LocalContext` | 用户态上下文（所有通用寄存器 + CSR） |
| `finish` | `bool` | 任务是否已完成 |
| `stack` | `[usize; 1024]` | 独立的用户栈（8 KiB） |
```

- Trap 上下文与任务上下文区别

- trace 如何帮助观测系统调用行为

- 典型 bug 与排查
```

请严格审查并输出：
1. 逻辑跳步位置；
2. 术语误用；
3. 缺失前提；
4. 面向大二学生的改写版（5分钟）。
