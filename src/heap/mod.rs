//! 🌱 堆内存分配器模块
//!
//! 提供基于 buddy_system_allocator 的堆内存管理功能
mod interrupt_heap;

use interrupt_heap::InterruptHeap;
use buddy_system_allocator::LockedHeap;

/// 全局堆分配器
#[global_allocator]
static HEAP_ALLOCATOR: InterruptHeap = InterruptHeap {inner: LockedHeap::empty()};

/// 堆内存大小：1MB
static HEAP_SIZE: usize = 1024 * 1024;

/// 堆内存空间（存储在 BSS 段中）
static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// 🌱 初始化堆分配器
///
/// 将预分配的堆内存空间注册到全局分配器中
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .inner
            .lock()
            .init(core::ptr::addr_of!(HEAP_SPACE) as usize, HEAP_SIZE);
    }
}

/// 🚨 内存分配错误处理器
///
/// 当堆内存分配失败时调用此函数
#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

/// `format!` 宏（no_std 版本）
///
/// 说明：
/// - 该宏内部会进行堆分配，必须先调用 `heap::init_heap()` 初始化堆，否则会触发分配失败
/// - 用法示例：`let s = no_std::format!("x = {}", 123);`
#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        // 直接转发到 alloc::format!（返回 alloc::string::String）
        // 注意：用 $crate::__alloc 避免要求调用方也显式 `extern crate alloc`
        $crate::__alloc::format!($($arg)*)
    }};
}
