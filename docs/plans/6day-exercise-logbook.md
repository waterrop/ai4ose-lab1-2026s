# 5个练习通过记录与回归日志

> 用法：每天完成后补充“命令、结果、失败原因、修复摘要”。  
> 建议只记录关键信息，避免长日志污染。

## 记录规范

- 命令：写完整可复现命令。
- 结果：`PASS / FAIL`。
- 失败摘要：一句话说明现象。
- 修复摘要：一句话说明改动点。
- 回归：标记是否执行基础回归。

## 练习进度总览

| 章节 | 练习内容 | 目标状态 | 当前状态 |
|---|---|---|---|
| ch3 | trace | PASS | TODO |
| ch4 | trace + mmap/munmap | PASS | TODO |
| ch5 | spawn + set_priority + stride | PASS | TODO |
| ch6 | linkat/unlinkat/fstat | PASS | TODO |
| ch8 | enable_deadlock_detect | PASS | TODO |

## ch3 记录

- 命令：
  - `cargo run --features exercise`
  - `./test.sh exercise`
- 结果：`TODO`
- 失败摘要：`TODO`
- 修复摘要：`TODO`
- 回归：`TODO`

## ch4 记录

- 命令：
  - `cargo run --features exercise`
  - `./test.sh exercise`
- 结果：`TODO`
- 失败摘要：`TODO`
- 修复摘要：`TODO`
- 回归：`TODO`

## ch5 记录

- 命令：
  - `cargo run --features exercise`
  - `ch5_usertest`
  - `./test.sh exercise`
- 结果：`TODO`
- 失败摘要：`TODO`
- 修复摘要：`TODO`
- 回归：`TODO`

## ch6 记录

- 命令：
  - `cargo run --features exercise`
  - `ch6_usertest`
  - `./test.sh exercise`
- 结果：`TODO`
- 失败摘要：`TODO`
- 修复摘要：`TODO`
- 回归：`TODO`

## ch8 记录

- 命令：
  - `cargo run --features exercise`
  - `ch8_usertest`
  - `./test.sh exercise`
  - `./test.sh base`
- 结果：`TODO`
- 失败摘要：`TODO`
- 修复摘要：`TODO`
- 回归：`TODO`

## 最终验收

- [ ] 5/5 练习通过
- [ ] 至少完成 1 轮回归快测
- [ ] 所有失败项均有“根因 + 修复”记录
- [ ] 可以独立复现实验结果
