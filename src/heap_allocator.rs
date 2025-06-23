// os/src/mm/heap_allocator.rs

//! 🌱 堆内存分配器模块
//! 
//! 提供基于 buddy_system_allocator 的堆内存管理功能，
//! 支持动态内存分配和释放。

use buddy_system_allocator::LockedHeap;

/// 全局堆分配器
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

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