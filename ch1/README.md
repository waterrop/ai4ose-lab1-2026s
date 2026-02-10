# 第一章

操作系统内核编程语言是Rust，操作系统内核基于的硬件环境是RISC-V 64，操作系统内核实验在QEMU模拟器上运行。要求基于Rust Crate组件化编程，即每个内核或内核模块是可以发布到crates.io上的条件（cargo publish --dry-run 成功运行）

第一章旨在引导你构建一个尽量简单的**特权态裸机应用程序**：

- 什么是特权态裸机应用程序？它与构建操作系统是什么关系？
特权态是**CPU硬件提供的一种权限机制**，拥有对所有硬件的完全控制权。引导加载程序（如RustSBI）或最底层固件就运行在此态。
裸机是指程序的运行环境，即程序直接运行在硬件上。
所以我们要构建的特权态裸机应用程序是一种直接运行在硬件上的应用程序。
- 为什么从构建特权态裸机应用程序开始？
- 如何构建一个特权态裸机应用程序？
    计算机硬件只能识别二进制01，我们用一系列01串即指令，来控制计算机硬件。
    内核的硬件环境是RISC-V 64，而大部分人的电脑不支持RISC-V 64架构，所以我们使用QEMU模拟RISC-V 64硬件环境。
    使用build.rs定制链接脚本，将主函数、栈放进内存段的相应位置


## 编译
在ch1/下键入cargo build --target riscv64gc-unknown-none-elf

## 使用qemu运行（需提前在根目录安装好rustsbi-qemu.bin）
在ch1/下键入qemu-system-riscv64 -machine virt -nographic -bios ../rustsbi-qemu.bin -kernel target/riscv64gc-unknown-none-elf/debug/ch1