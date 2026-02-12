# Day 4 执行单：ch5 + 练习3（spawn + stride）

## 今日目标

- 建立 Process/PCB 视角下的系统调用与调度理解。
- 完成 `spawn` 与 `set_priority`，落地 stride 调度。
- 完成前向兼容回归（按章节要求）。

## 时间切片（建议）

- 09:00-11:30：阅读 `ch5/README.md` + `ch5/exercise.md`
- 14:00-17:30：实现 spawn + stride
- 19:30-21:30：`ch5_usertest` + 回归 + 文档输出

## 实现要点

### A. spawn

- 读取用户态程序名（路径 + 长度）。
- 在应用集合中查找可执行程序。
- 直接创建并加入新进程（无需复制父进程地址空间）。
- 成功返回子进程 pid，失败返回 `-1`。

### B. stride 调度

- 在进程结构中加入 `stride`、`priority` 字段。
- 初始 `priority=16`，`stride=0`。
- 每次选择 runnable 中 stride 最小的进程运行。
- 调度后 `stride += pass`，其中 `pass = BigStride / priority`。
- `set_priority(prio)`：`prio >= 2` 才合法。

## 验证命令

```bash
cd /home/chyyuu/thecodes/ai4ose/rCore-Tutorial-in-single-workspace/ch5
cargo run --features exercise
# 进入用户shell后执行
ch5_usertest
./test.sh exercise
```

## AI协作任务单

- 让 AI 给出 stride 的“最简可测实现 + 防溢出建议”。
- 让 AI 列“前向兼容回归关键点”：
  - mmap/munmap 行为
  - fork/exec/wait 语义
  - clock_gettime 行为

## D4 输出模板

### D4-讲义标题
《从 Task 到 Process：调度策略为何要引入优先级》

### 讲义最小结构
1. 进程模型扩展了什么
2. spawn 与 fork/exec 的设计差异
3. stride 的公平性直觉
4. 优先级无效输入的系统行为
5. 前向兼容为何重要

### ch5 教程改进（至少3条）
- 增加 spawn 与 fork/exec 的对照案例。
- 增加 stride 调度参数与行为示例。
- 增加“回归失败优先排查点”。

## Day 4 验收清单

- [ ] `spawn` 可创建并启动目标进程
- [ ] `set_priority` 参数校验正确
- [ ] stride 调度生效
- [ ] `ch5_usertest` 通过
- [ ] `./test.sh exercise` 通过
- [ ] 完成 D4 讲义与 ch5 改进稿
