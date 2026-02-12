# Day 5 执行单：ch6 + 练习4（linkat/unlinkat/fstat）

## 今日目标

- 理清目录项、inode、数据块和 fd 表的关系。
- 完成 `linkat/unlinkat/fstat` 三个系统调用。
- 输出 `D5-文件系统与inode讲义` 与 ch6 改进稿。

## 时间切片（建议）

- 09:00-11:30：阅读 `ch6/README.md` + `ch6/exercise.md`
- 14:00-17:30：完成 3 个 syscall 实现
- 19:30-21:30：测试回归 + 讲义与改进稿

## 前置操作

按章节要求处理本地依赖（如果未完成）：

```bash
cd /home/chyyuu/thecodes/ai4ose/rCore-Tutorial-in-single-workspace/ch6
cargo clone tg-easy-fs
```

并在 `Cargo.toml` 使用本地路径依赖。

## 实现优先级

1. `fstat`：先打通结构体回填链路（fd -> inode -> Stat）。
2. `linkat`：创建新目录项并增加硬链接计数。
3. `unlinkat`：删除目录项并在链接计数到 0 时回收资源。

## 验证命令

```bash
cd /home/chyyuu/thecodes/ai4ose/rCore-Tutorial-in-single-workspace/ch6
cargo run --features exercise
# 进入用户shell后执行
ch6_usertest
./test.sh exercise
```

## AI协作任务单

- 让 AI 给出“硬链接计数变化时机表（create/link/unlink/delete）”。
- 让 AI 审查 `Stat` 各字段填充完整性。
- 让 AI 给出“unlink 后资源回收遗漏”排查单。

## D5 输出模板

### D5-讲义标题
《硬链接不是复制文件：目录项与 inode 的关系》

### 讲义最小结构
1. inode 与目录项分工
2. 硬链接语义
3. unlink 为什么不一定删除数据
4. fstat 如何反映文件状态
5. 常见一致性 bug

### ch6 教程改进（至少3条）
- 增加“硬链接生命周期图”（创建/增加/删除/回收）。
- 增加 `fstat` 字段解释与示例输出。
- 增加“文件不存在/同名链接”的异常行为说明。

## Day 5 验收清单

- [ ] `linkat` 功能正确
- [ ] `unlinkat` 功能正确并支持回收
- [ ] `fstat` 返回结构正确
- [ ] `ch6_usertest` 通过
- [ ] `./test.sh exercise` 通过
- [ ] 完成 D5 讲义与 ch6 改进稿
