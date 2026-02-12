# Day 3 执行单：ch4 + 练习2（trace重写 + mmap/munmap）

## 今日目标

- 理解地址空间与页表权限。
- 完成 `ch4` 练习：`trace` 权限版 + `mmap/munmap`。
- 输出 `D3-虚存与权限检查讲义`。

## 时间切片（建议）

- 09:00-11:30：阅读 `ch4/README.md` + `ch4/exercise.md`
- 14:00-17:30：编码实现 + 单项验证
- 19:30-21:30：全量测试 + 文档改进

## 前置操作

按章节要求处理本地依赖（如果未完成）：

```bash
cd /home/chyyuu/thecodes/ai4ose/rCore-Tutorial-in-single-workspace/ch4
cargo clone tg-kernel-vm
```

并在 `Cargo.toml` 使用本地路径依赖。

## 实现优先级

1. `trace` 地址翻译与权限检查（读/写分离）。
2. `mmap` 参数检查：
   - 对齐检查
   - `prot` 位合法性
   - 区间冲突检查
3. `mmap` 映射创建（`len` 向上按页对齐）。
4. `munmap` 区间合法性与解除映射。

## 验证命令

```bash
cd /home/chyyuu/thecodes/ai4ose/rCore-Tutorial-in-single-workspace/ch4
cargo run --features exercise
./test.sh exercise
```

## AI协作任务单

- 让 AI 给出 `prot` 与页表权限位映射表（可读/可写/可执行/用户位）。
- 让 AI 生成“mmap/munmap 参数错误分支检查单”。
- 让 AI 对 `trace` 返回 `-1` 的场景做穷举检查。

## D3 输出模板

### D3-讲义标题
《地址空间引入后，系统调用为什么必须做权限检查》

### 讲义最小结构
1. ch3 到 ch4 的本质变化
2. 地址翻译的必要性
3. 用户可见/可写检查规则
4. mmap/munmap 的最小语义
5. 失败分支如何设计

### ch4 教程改进（至少3条）
- 增加 `trace`（ch3版本 vs ch4版本）差异对比。
- 增加 `mmap/munmap` 参数校验流程图。
- 增加“prot 位非法/地址未对齐/区间重叠”的排错表。

## Day 3 验收清单

- [ ] `trace` 权限检查版可用
- [ ] `mmap` 可用且参数校验完整
- [ ] `munmap` 可用且错误分支可回报
- [ ] `cargo run --features exercise` 通过
- [ ] `./test.sh exercise` 通过
- [ ] 完成 D3 讲义与 ch4 改进稿
