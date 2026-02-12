# Day 2 执行单：ch3 + 练习1（trace）

## 今日目标

- 理解多道程序与分时调度的核心机制。
- 完成 `ch3/exercise.md` 的 `trace` 练习并通过测例。
- 输出 `D2-调度与上下文切换讲义`。

## 时间切片（建议）

- 09:00-11:30：阅读 `ch3/README.md` + `ch3/exercise.md`
- 14:00-17:30：实现 `sys_trace`
- 19:30-21:30：测试 + 讲义与教程改进

## 关键实践路径

### A. 理解代码落点

优先检查：
- `ch3/src/main.rs` 中系统调用分发与 `Trace` trait 实现
- `ch3/src/task.rs` 中任务结构与可扩展状态

### B. 实现策略（最小通过优先）

1. 为当前任务维护 syscall 计数结构。
2. 在 syscall 处理入口处累计计数。
3. `trace_request=0`：从用户地址读 `u8`。
4. `trace_request=1`：向用户地址写 `u8`。
5. `trace_request=2`：返回指定 syscall id 计数（本次调用计入）。
6. 其他 `trace_request` 返回 `-1`。

## 验证命令

```bash
cd /home/chyyuu/thecodes/ai4ose/rCore-Tutorial-in-single-workspace/ch3
cargo run --features exercise
./test.sh exercise
```

## AI协作任务单

- 让 AI 输出“最小可过测例实现步骤”，每次只推进一小步。
- 失败时粘贴日志，要求 AI 给“3个最可能根因 + 验证步骤”。
- 实现完成后，让 AI 审查边界条件：
  - `trace_request` 非法值
  - 地址读写异常
  - syscall 计数是否覆盖 trace 自身

## D2 输出模板

### D2-讲义标题
《从批处理到分时：任务切换与上下文保存的最小闭环》

### 讲义最小结构
1. 为什么需要调度
2. 协作式与抢占式差异
3. Trap 上下文与任务上下文区别
4. trace 如何帮助观测系统调用行为
5. 典型 bug 与排查

### ch3 教程改进（至少3条）
- 增加 `trace` 的“请求类型 -> 语义 -> 返回值”对照表。
- 增加“syscall 计数放置位置”的设计权衡说明。
- 增加“常见错误：计数时机错误导致测试偏差”的案例。

## Day 2 验收清单

- [ ] `trace` 功能实现完整
- [ ] `cargo run --features exercise` 通过
- [ ] `./test.sh exercise` 通过
- [ ] 完成 D2 讲义
- [ ] 完成 ch3 教程改进稿
