//! 🖥️ 系统功能模块
//! 
//! 提供系统级功能，包括关机、重启、内存布局打印和 BSS 段清理。

use crate::println;

/// 🖥️ RISC-V 系统关机函数
/// 
/// 在 QEMU virt 平台上，通过向 Power Management 寄存器写入特定值来实现关机
/// 这是 QEMU 特有的关机机制，在实际硬件上需要根据具体平台实现

// QEMU virt 平台的 Power Management 寄存器地址
const VIRT_TEST: usize = 0x100000;

/// 🖥️ 系统关机函数
/// 
/// 在 QEMU virt 平台上，通过向 Power Management 寄存器写入特定值来实现关机
/// 这是 QEMU 特有的关机机制，在实际硬件上需要根据具体平台实现
pub fn shutdown() -> ! {    
    // 关机命令：写入 0x5555 到 Power Management 寄存器
    // 这个值告诉 QEMU 模拟器关闭虚拟机
    unsafe {
        core::ptr::write_volatile(VIRT_TEST as *mut u32, 0x5555);

        // 如果关机失败，进入无限循环
        loop {
            // 使用 fence 指令确保内存操作完成
            core::arch::asm!("fence");
        }
    }
}


/// 🚀 系统重启函数
/// 
/// 通过向 Power Management 寄存器写入重启命令来实现系统重启
/// 🚀 系统重启函数
/// 
/// 通过向 Power Management 寄存器写入重启命令来实现系统重启
pub fn reboot() -> ! {
    // 重启命令：写入 0x7777 到 Power Management 寄存器
    unsafe {
        core::ptr::write_volatile(VIRT_TEST as *mut u32, 0x7777);

        // 如果重启失败，进入无限循环
        loop {
            core::arch::asm!("fence");
        }
    }
}


/// 🗺️ 打印内存段地址信息
/// 
/// 显示所有内存段的开始地址、结束地址和大小信息
/// 包括 .text、.rodata、.data、.bss 和 .stack 段
pub fn print_memory_layout() {
    // 外部链接声明，引用链接脚本中定义的段地址变量
    unsafe extern "C" {
        static __TEXT_START: u8;
        static __TEXT_END: u8;
        static __RODATA_START: u8;
        static __RODATA_END: u8;
        static __DATA_START: u8;
        static __DATA_END: u8;
        static __BSS_START: u8;
        static __BSS_END: u8;
        static __STACK_START: u8;
        static __STACK_END: u8;
        static __STACK_TOP: u8;
    }
    println!("📋 内存段布局信息:");
    println!("==================");
    
    unsafe {
        // 打印各段信息
        println!("🔧 .text 段:");
        println!("   开始地址: 0x{:08x}", &__TEXT_START as *const u8 as usize);
        println!("   结束地址: 0x{:08x}", &__TEXT_END as *const u8 as usize);
        
        println!("📖 .rodata 段:");
        println!("   开始地址: 0x{:08x}", &__RODATA_START as *const u8 as usize);
        println!("   结束地址: 0x{:08x}", &__RODATA_END as *const u8 as usize);
        
        println!("💾 .data 段:");
        println!("   开始地址: 0x{:08x}", &__DATA_START as *const u8 as usize);
        println!("   结束地址: 0x{:08x}", &__DATA_END as *const u8 as usize);
        
        println!("🗑️ .bss 段:");
        println!("   开始地址: 0x{:08x}", &__BSS_START as *const u8 as usize);
        println!("   结束地址: 0x{:08x}", &__BSS_END as *const u8 as usize);
        
        println!("📚 .stack 段:");
        println!("   开始地址: 0x{:08x}", &__STACK_START as *const u8 as usize);
        println!("   结束地址: 0x{:08x}", &__STACK_END as *const u8 as usize);
        println!("   栈顶地址: 0x{:08x}", &__STACK_TOP as *const u8 as usize);
        println!("==================");
    }
}



/// 🧹 清空 BSS 段
/// 
/// 将 .bss 段的所有内存初始化为 0
/// 这是裸机程序启动时的必要步骤，确保未初始化的全局变量为 0
pub fn clear_bss() {
    // 外部链接声明，引用链接脚本中定义的 BSS 段地址变量
    unsafe extern "C" {
        static __BSS_START: u8;
        static __BSS_END: u8;
    }
    
    unsafe {
        // 获取 BSS 段的开始和结束地址
        let bss_start = &__BSS_START as *const u8 as usize;
        let bss_end = &__BSS_END as *const u8 as usize;
        
        // 计算 BSS 段大小
        let bss_size = bss_end - bss_start;
        
        println!("🧹 清空 BSS 段:");
        println!("   开始地址: 0x{:08x}", bss_start);
        println!("   结束地址: 0x{:08x}", bss_end);
        println!("   段大小: {} 字节", bss_size);
        
        // 将 BSS 段的所有字节设置为 0
        let bss_start_ptr = bss_start as *mut u8;
        for i in 0..bss_size {
            *bss_start_ptr.add(i) = 0;
        }
        
        println!("✅ BSS 段清空完成");
    }
}

