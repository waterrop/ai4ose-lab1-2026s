# 6天AI协作操作系统学习执行包

本执行包用于落地 `6天AI协作操作系统学习计划`，目标是：

1. 6 天内完成 `ch1~ch8` 主线学习。
2. 完成 5 个练习章（`ch3/ch4/ch5/ch6/ch8`）的实现与验收。
3. 形成可复用的个性化教程（含每章导学、常错点、最小实验、检查清单）。

## 文件清单

- `6day-day1-bootstrap.md`：Day 1 执行单（ch1/ch2）
- `6day-day2-ch3-ex1.md`：Day 2 执行单（ch3 trace）
- `6day-day3-ch4-ex2.md`：Day 3 执行单（ch4 trace + mmap/munmap）
- `6day-day4-ch5-ex3.md`：Day 4 执行单（spawn + stride）
- `6day-day5-ch6-ex4.md`：Day 5 执行单（linkat/unlinkat/fstat）
- `6day-day6-ch8-final.md`：Day 6 执行单（死锁检测 + 总交付）
- `6day-ai-collab-prompts.md`：AI 协作提示词包
- `6day-exercise-logbook.md`：练习通过记录与回归记录
- `6day-ch1-ch8-personalized-tutorial.md`：ch1~ch8 个性化教程改进稿

## 每天固定执行节奏（建议）

- 输入学习（2.5h）：阅读章节文档 + 关键代码定位
- 编码实践（3.5h）：按最小可通过路径实现 + 测试
- 费曼输出与教程改进（2h）：讲解录音/讲稿 + 文档迭代

## 每天必须提交的最小产物

1. 当日 `费曼讲解稿`（3~8 分钟）。
2. 当日 `错误排查记录`（至少 3 条）。
3. 当日 `章节改进稿`（至少新增导学、常错点、检查清单）。
4. 在 `6day-exercise-logbook.md` 更新命令与结果。

## 执行原则

- 先通过测例，再优化代码，再整理教程。
- 单个问题卡住超过 90 分钟，立即切换为“日志驱动排查”模式。
- 每晚做一次回归快测，避免最后一天集中爆雷。
- 输出优先：每天必须有文档与讲解产物，防止只写代码不沉淀。
