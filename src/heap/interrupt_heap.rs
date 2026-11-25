use core::alloc::{GlobalAlloc, Layout};

use buddy_system_allocator::LockedHeap;
use crate::trap::InterruptGuard;

/// 保留原分配器的堆锁，在获取它之前先关闭本 CPU 的中断。
pub struct InterruptHeap {
    pub inner: LockedHeap
}

// 分配规则交给原分配器，额外保证单核下持堆锁期间不会被定时器抢占。
unsafe impl GlobalAlloc for InterruptHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _interrupt_guard = InterruptGuard::new();
        // 原分配器在返回前释放堆锁，然后本作用域恢复中断。
        unsafe { self.inner.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _interrupt_guard = InterruptGuard::new();
        // 调用方保证 ptr 和 layout 对应此前分配的有效内存块。
        unsafe { self.inner.dealloc(ptr, layout) }
    }
}