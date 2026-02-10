#![no_std]          // 禁用Rust标准库，使用核心库（core）
#![no_main]         // 禁用标准main入口，使用自定义入口点
#![deny(warnings)]  // 将所有警告视为错误，严格代码质量

/// Supervisor汇编入口。
/// 设置栈并跳转到Rust
#[unsafe(naked)]      // 裸函数：编译器不生成函数前导/后缀代码
#[unsafe(no_mangle)]          // 禁止名称修饰，确保链接时能找到"_start"
#[unsafe(link_section = ".text.entry")] // 将此函数放在特定内存段
unsafe extern "C" fn _start() -> ! {  // 永不返回的函数
    const STACK_SIZE: usize = 4096;  // 定义4KB栈大小
    #[unsafe(link_section = ".bss.uninit")]  // 将栈放在未初始化数据段
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];
    // 内联汇编：设置栈指针并跳转
    core::arch::naked_asm!(
        "la sp, {stack} + {stack_size}",  // 加载栈顶地址到sp
        "j {main}",                       // 跳转到rust_main
        stack_size = const STACK_SIZE,
        stack = sym STACK,
        main = sym rust_main,
    )
}

///  特权态裸机程序。
/// 打印`Hello, world!`，然后关机。
extern "C" fn rust_main() -> !{
    use sbi_rt::*;
    for c in b"Hello, world!\n"{
        #[allow(deprecated)]
        legacy::console_putchar(*c as _);
    }
    system_reset(Shutdown, NoReason);
    unreachable!()
}

/// Rust异常处理函数，以异常方式关机
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> !{
    use sbi_rt::*;
    system_reset(Shutdown, SystemFailure);
    loop{}
}