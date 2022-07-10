#![no_std] // 不链接 Rust 标准库
#![no_main] // 禁用所有 Rust 层级的入口点
#![feature(custom_test_frameworks)]
#![test_runner(build_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use build_os::println;

#[no_mangle] // 不重整函数名
// 编译器会寻找一个名为 `_start` 的函数，这个函数是入口点
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}

/// 这个函数将在 panic 时被调用
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    build_os::test_panic_handler(info)
}


#[test_case]
fn trivial_assertion() {
    println!("Hello World{}", "!");
}
