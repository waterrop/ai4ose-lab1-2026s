# Day 6 执行单：ch7/ch8收束 + 练习5（ch8死锁检测）+ 最终交付

## 今日目标

- 完成 `ch8` 练习：`enable_deadlock_detect` 与锁/信号量联动。
- 收束 `ch7/ch8` 的 IPC 与并发知识。
- 交付 `ch1~ch8` 个性化教程与练习通过记录。

## 时间切片（建议）

- 09:00-11:30：阅读 `ch7/README.md`、`ch8/README.md`、`ch8/exercise.md`
- 14:00-17:30：实现死锁检测并通过练习测试
- 19:30-22:00：全量回归 + 最终文档整理

## 实现优先级

1. `enable_deadlock_detect(is_enable)` 参数校验（只接受 0/1）。
2. 为进程维护死锁检测开关状态。
3. 在 `mutex_lock` 与 `semaphore_down` 路径接入检测。
4. 检测到死锁时返回 `-0xDEAD`。
5. 兼容要求：可按需启用/禁用，不破坏基础流程。

## 验证命令

```bash
cd /home/chyyuu/thecodes/ai4ose/rCore-Tutorial-in-single-workspace/ch8
cargo run --features exercise
# 进入用户shell后执行
ch8_usertest
./test.sh exercise
# 按需补充
./test.sh base
```

## AI协作任务单

- 让 AI 把死锁检测算法转成当前代码结构可落地的伪代码。
- 让 AI 给出“线程/资源图构建”最小实现建议。
- 让 AI 审核 `-0xDEAD` 返回路径是否覆盖 `mutex/semaphore` 两类资源。

## 最终交付清单

1. 5 个练习通过记录：更新 `6day-exercise-logbook.md`
2. 8 章个性化改进稿：更新 `6day-ch1-ch8-personalized-tutorial.md`
3. 最终总讲解稿：20~30 分钟结构化讲解
4. 回归记录：至少一轮基础回归 + 练习回归

## 最终讲解大纲（20~30分钟）

1. ch1/ch2：启动与系统调用最小闭环
2. ch3：任务切换与分时
3. ch4：地址空间与权限
4. ch5：进程与调度策略
5. ch6：文件系统与链接语义
6. ch7：IPC 与信号
7. ch8：线程同步与死锁检测
8. 学习方法复盘：AI协作如何提高效率

## Day 6 验收清单

- [ ] `enable_deadlock_detect` 实现完成
- [ ] `./test.sh exercise` 通过
- [ ] 必要 `base` 回归已执行
- [ ] 5 个练习通过记录已整理
- [ ] 8 章改进稿已完成
- [ ] 20~30 分钟总讲解稿已完成
