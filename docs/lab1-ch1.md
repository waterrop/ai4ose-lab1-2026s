# 如何自动化构建、运行系统
prompt：我正在进行一个操作系统实验
要求：操作系统内核编程语言是Rust，操作系统内核基于的硬件环境是RISC-V 64，操作系统内核实验在QEMU模拟器上运行。要求基于Rust Crate组件化编程，即每个内核或内核模块是可以发布到crates.io上的条件（cargo publish --dry-run 成功运行）
请按以下功能：
运行系统：
cargo qemu --ch <n>

在 qemu 运行第 n 章的操作系统。

编译系统：
cargo make --ch <n>
编译第 n 章操作系统。

清空系统：

只清空第 n 章的编译生成的文件：cargo clean -p ch <n>

彻底清空所有章节/工具生成物：根目录下直接 cargo clean
为这个实验进行自动化构建、运行系统。