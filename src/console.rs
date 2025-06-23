//! 🖥️ 串口控制台模块
//! 
//! 提供基于 QEMU virt 平台的 UART 串口输出功能，
//! 支持格式化打印和换行输出。

use core::fmt::{self, Write};

/// QEMU virt UART 基地址
const UART_BASE: usize = 0x1000_0000;

/// 🖥️ 通用异步收发器 (UART)
pub struct Uart;

impl Uart {
    /// 创建新的 UART 实例
    pub const fn new() -> Self {
        Self
    }
    
    /// 检查 UART 是否可写
    fn is_writable(&self) -> bool {
        // 检查状态寄存器 (LSR) 的发送就绪位
        unsafe { (core::ptr::read_volatile((UART_BASE + 0x5) as *const u8) & (1 << 5)) != 0 }
    }
    
    /// 写入单个字节
    pub fn write_byte(&self, byte: u8) {
        // 等待发送缓冲区空闲
        while !self.is_writable() {}
        
        // 写入数据寄存器
        unsafe {
            core::ptr::write_volatile(UART_BASE as *mut u8, byte);
        }
    }
    
    /// 写入字节序列
    pub fn write_bytes(&self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }
}

/// 全局控制台写入器
pub struct ConsoleWriter;

impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let uart = Uart::new();
        uart.write_bytes(s.as_bytes());
        Ok(())
    }
}

/// 初始化控制台
pub fn init() {
    // QEMU virt 平台 UART 默认已初始化
}

/// 输出格式化内容
pub fn _print(args: fmt::Arguments) {
    let mut writer = ConsoleWriter;
    let _ = fmt::write(&mut writer, args);
}

/// 输出格式化内容并换行
pub fn _println(args: fmt::Arguments) {
    _print(args);
    let uart = Uart::new();
    uart.write_byte(b'\n');
}

/// print! 宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*))
    };
}

/// println! 宏
#[macro_export]
macro_rules! println {
    () => {
        $crate::console::_println(format_args!(""))
    };
    ($($arg:tt)*) => {
        $crate::console::_println(format_args!($($arg)*))
    };
}