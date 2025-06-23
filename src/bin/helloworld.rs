//! 🌍 Hello World 应用程序
//! 
//! 一个简单的示例应用，演示基本的系统功能：
//! - 串口输出
//! - 系统关机

#![no_std]
#![no_main]

use no_std::println;
use no_std::system;

/// 🌍 应用程序入口点
/// 
/// 打印 "Hello, World!" 并关闭系统
#[unsafe(no_mangle)]
pub fn main() -> ! {
    println!("Hello, World!");

    system::shutdown()
}