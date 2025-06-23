//! 🚨 错误处理模块
//! 
//! 提供统一的错误处理机制和 panic 处理器。

/// 简单的错误处理模块 - 只提供基本的 panic 处理
use crate::{println, system::shutdown};
use core::panic::PanicInfo;

/// 🚨 Panic 处理器
/// 
/// 当程序发生 panic 时调用此函数
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("🚨 PANIC: {}", info);
    
    // 关机
    shutdown()
} 