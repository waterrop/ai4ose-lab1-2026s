# ch1~ch8 个性化操作系统实验教程（AI协作版）

学习者画像：Rust 中等，OS 基础一般，实践时间有限，容易在调试中迷失。  
使用策略：每章遵循“最小闭环 -> 测试通过 -> 费曼输出 -> 教程改进”。

---

## ch1 裸机程序 LibOS

### 学习目标
- 理解最小执行环境与启动流程。
- 能解释 `-bios none` 的意义。
- 能完成裸机 HelloWorld 的编译运行。

### 先修知识
- Rust `no_std` 基础
- RISC-V 特权级概念

### 最小实践任务
1. `cargo build`
2. `cargo run`
3. 观察输出并解释为何能关机退出

### 关键代码导航
- `ch1/src/main.rs`
- `ch1/.cargo/config.toml`
- `ch1/build.rs`

### 常错点
- 忘记添加 `riscv64gc-unknown-none-elf` target。
- QEMU 未安装或版本太旧。
- 把主机环境编译错误当成内核逻辑错误。

### 验收清单
- [ ] 能独立跑通 ch1
- [ ] 能讲清启动到输出的链路
- [ ] 能说出 3 个排错点

---

## ch2 Batch OS

### 学习目标
- 理解 U/S 态切换与 Trap 处理。
- 理解系统调用从 `ecall` 到内核分发的路径。

### 先修知识
- ch1 启动流程
- 寄存器上下文基本概念

### 最小实践任务
1. 跑通 `cargo run`
2. 观察多个用户程序按批次运行
3. 执行测试脚本并记录结果

### 关键代码导航
- `ch2/src/main.rs`
- `ch2/build.rs`
- `ch2/test.sh`

### 常错点
- 不理解用户程序是如何被打包进内核镜像。
- 把“系统调用错误”误认为“QEMU问题”。
- 缺少日志级别控制导致定位困难。

### 验收清单
- [ ] 能解释 Trap 保存/恢复上下文
- [ ] 能解释 write/exit 的最小系统调用语义
- [ ] 能完成基础测试

---

## ch3 MultiProg/TimeSharing + exercise(trace)

### 学习目标
- 理解任务调度（协作式/抢占式）核心差异。
- 完成 trace 系统调用与 syscall 计数统计。

### 先修知识
- ch2 Trap 与系统调用路径
- 任务上下文概念

### 最小实践任务
1. 实现 `trace_request=0/1/2`
2. 把 syscall 计数接入任务结构
3. 跑通 `./test.sh exercise`

### 关键代码导航
- `ch3/src/main.rs`
- `ch3/src/task.rs`
- `ch3/exercise.md`

### 常错点
- 计数时机不对（没把本次 trace 计入）。
- 读写地址转换类型错误。
- 非法 `trace_request` 未正确返回 `-1`。

### 验收清单
- [ ] trace 三种请求均可用
- [ ] 测试通过
- [ ] 能解释协作式与抢占式行为区别

---

## ch4 AddrSpace + exercise(trace重写/mmap/munmap)

### 学习目标
- 理解地址空间隔离与页权限检查。
- 完成 trace 权限版与 mmap/munmap。

### 先修知识
- ch3 trace 行为
- 页表和权限位基本概念

### 最小实践任务
1. 用 `translate` 重写 trace 读写逻辑
2. 完成 mmap 参数校验和映射
3. 完成 munmap 解除映射

### 关键代码导航
- `ch4/src/main.rs`
- `ch4/src/process.rs`
- `ch4/exercise.md`

### 常错点
- `prot` 位和页表权限位映射混乱。
- 未设置用户位导致用户不可访问。
- 地址未对齐或区间冲突未拦截。

### 验收清单
- [ ] trace 在地址非法时返回 `-1`
- [ ] mmap/munmap 功能与错误分支完整
- [ ] 练习测试通过

---

## ch5 Process + exercise(spawn/stride)

### 学习目标
- 理解进程作为资源容器的角色。
- 完成 spawn 和带优先级的 stride 调度。

### 先修知识
- ch4 地址空间
- 基本调度策略概念

### 最小实践任务
1. 完成 `spawn`
2. 完成 `set_priority`
3. 在调度器中应用 stride 选择逻辑

### 关键代码导航
- `ch5/src/main.rs`
- `ch5/src/process.rs`
- `ch5/src/processor.rs`
- `ch5/exercise.md`

### 常错点
- `prio < 2` 未正确拒绝。
- `pass` 计算未考虑溢出风险。
- 回归时破坏上一章 `mmap/munmap`。

### 验收清单
- [ ] `ch5_usertest` 通过
- [ ] 练习测试通过
- [ ] 前向兼容回归通过

---

## ch6 Filesystem + exercise(linkat/unlinkat/fstat)

### 学习目标
- 理解文件系统层次和硬链接语义。
- 完成 linkat/unlinkat/fstat 三个 syscall。

### 先修知识
- 进程文件描述符表
- inode 与目录项关系

### 最小实践任务
1. `fstat` 先打通
2. `linkat` 增加硬链接
3. `unlinkat` 删除链接并正确回收

### 关键代码导航
- `ch6/src/main.rs`
- `ch6/src/fs.rs`
- `ch6/exercise.md`

### 常错点
- 把“删除目录项”误当成“立即删除文件内容”。
- 忽略链接计数导致资源泄漏。
- `fstat` 结构字段填充不全。

### 验收清单
- [ ] `ch6_usertest` 通过
- [ ] 练习测试通过
- [ ] 能解释目录项/inode/数据块关系

---

## ch7 IPC（管道与信号）

### 学习目标
- 理解 Pipe 与 Signal 的使用场景与实现思路。
- 形成对“文件抽象统一接口”的系统认知。

### 先修知识
- ch6 文件系统接口
- 进程调度与状态转换

### 最小实践任务
1. 运行管道测试程序
2. 观察信号处理的关键路径
3. 总结 IPC 与文件系统接口的连接点

### 关键代码导航
- `ch7/src/main.rs`
- `ch7/src/fs.rs`
- `ch7/src/process.rs`

### 常错点
- 混淆“同步通信”与“异步通知”。
- 忽略文件描述符生命周期。
- 信号屏蔽字与处理器语义混淆。

### 验收清单
- [ ] 能画出 pipe 数据流
- [ ] 能讲清 signal 发送-处理路径
- [ ] 能解释与 ch6 的继承关系

---

## ch8 Thread + exercise(死锁检测)

### 学习目标
- 理解线程与进程分离模型。
- 理解同步原语与死锁检测。
- 完成 `enable_deadlock_detect` 练习。

### 先修知识
- ch7 IPC 与进程结构
- 互斥锁/信号量基本语义

### 最小实践任务
1. 实现死锁检测开关 syscall
2. 在 `mutex_lock` / `semaphore_down` 接入检测
3. 检测到死锁返回 `-0xDEAD`

### 关键代码导航
- `ch8/src/main.rs`
- `ch8/src/process.rs`
- `ch8/src/processor.rs`
- `ch8/exercise.md`

### 常错点
- 开关状态未绑定到当前进程，导致跨进程污染。
- 只检查一种资源（锁或信号量）导致漏检。
- 返回码错误或未覆盖所有拒绝路径。

### 验收清单
- [ ] `ch8_usertest` 通过
- [ ] `./test.sh exercise` 通过
- [ ] 关键 base 回归通过

---

## 统一复盘模板（每章复用）

1. 我今天真正理解的 3 个点：
2. 我今天踩过的 3 个坑：
3. 我通过测试的最小证据：
4. 如果让我教别人，我会怎么讲：
5. 下一章前我必须补的前置知识：
