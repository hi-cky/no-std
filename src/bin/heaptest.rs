//! 🧪 堆内存测试应用程序
//! 
//! 测试堆内存分配器的功能：
//! - Box 动态分配
//! - Vec 动态数组
//! - 内存地址验证

#![no_std]
#![no_main]

use no_std::println;
use no_std::system;
use no_std::heap_allocator;

extern crate alloc;

/// 🧪 应用程序入口点
/// 
/// 初始化系统并运行堆内存测试
#[unsafe(no_mangle)]
pub fn main() -> ! {
    system::clear_bss();
    system::print_memory_layout();

    // 初始化堆
    heap_allocator::init_heap();

    heap_test();

    system::shutdown()
}

/// 🧪 堆内存测试函数
/// 
/// 测试 Box 和 Vec 的动态内存分配功能
#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    
    unsafe extern "C" {
        static __BSS_START: u8;
        static __BSS_END: u8;
    }
    
    unsafe {
        let bss_range = &__BSS_START as *const u8 as usize..&__BSS_END as *const u8 as usize;
        
        // 测试 Box 分配
        let a = Box::new(5);
        assert_eq!(*a, 5);
        // 检查分配的内存地址是否在 BSS 段范围内
        assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
        drop(a);
        
        // 测试 Vec 分配
        let mut v: Vec<usize> = Vec::new();
        for i in 0..500 {
            v.push(i);
        }
        for i in 0..500 {
            assert_eq!(v[i], i);
        }
        assert!(bss_range.contains(&(v.as_ptr() as usize)));
        drop(v);
        
        println!("heap_test passed!");
    }
}