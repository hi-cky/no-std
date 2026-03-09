//! 🧷 Trap/中断处理模块
//!
//! 提供 RISC-V 机器模式 trap 入口与基本分发逻辑。
//! 目前只处理机器定时器中断，其它异常直接记录并自旋。

use core::arch::{asm, global_asm};
use log::{error, info, warn};

use crate::timer;

// 引入汇编 trap 入口
global_asm!(include_str!("trap.S"));

/// 机器定时器中断的 cause 值（RV64）
const MCAUSE_MACHINE_TIMER: usize = 0x8000_0000_0000_0007;

/// 初始化 trap：设置 mtvec 并开启机器模式定时器中断
///
/// 说明：
/// - 使用 direct 模式（mtvec 低 2 位为 0）
/// - 这里只打开 MIE.MTIE 和 mstatus.MIE
pub fn init() {
    unsafe {
        // 设置 trap 向量入口
        set_mtvec(__trap_entry as usize);

        // 开启机器模式定时器中断（mie.MTIE）
        let mut mie: usize;
        asm!("csrr {0}, mie", out(reg) mie);
        mie |= 1 << 7; // MTIE
        asm!("csrw mie, {0}", in(reg) mie);

        // 全局中断使能（mstatus.MIE）
        let mut mstatus: usize;
        asm!("csrr {0}, mstatus", out(reg) mstatus);
        mstatus |= 1 << 3; // MIE
        asm!("csrw mstatus, {0}", in(reg) mstatus);
    }
}

/// Rust 侧 trap 处理函数（由汇编入口调用）
///
/// 说明：
/// - 读取 mcause 判断中断类型
/// - 目前仅处理机器定时器中断
#[unsafe(no_mangle)]
pub extern "C" fn trap_handler() {
    let cause = read_csr("mcause");
    let tval = read_csr("mtval");
    let epc = read_csr("mepc");

    if cause == MCAUSE_MACHINE_TIMER {
        // 调用已注册的计时器中断处理函数（如果存在）
        // warn!("[INTERTUPT] Timer interrupt occurred");
        timer::call_timer_interrupt_handler();
        return;
    }

    // 其它异常/中断：记录并停机（避免无穷异常）
    error!(
        "Unhandled trap: mcause=0x{:x}, mtval=0x{:x}, mepc=0x{:x}",
        cause, tval, epc
    );
    loop {
        unsafe { asm!("wfi") }
    }
}

/// 读取 CSR（小工具函数）
fn read_csr(name: &str) -> usize {
    let value: usize;
    unsafe {
        match name {
            "mcause" => asm!("csrr {0}, mcause", out(reg) value),
            "mtval" => asm!("csrr {0}, mtval", out(reg) value),
            "mepc" => asm!("csrr {0}, mepc", out(reg) value),
            _ => {
                // 未知 CSR，返回 0
                value = 0;
            }
        }
    }
    value
}

/// 设置 mtvec（直达模式）
fn set_mtvec(addr: usize) {
    // 低 2 位为 0 代表 direct 模式
    unsafe {
        asm!("csrw mtvec, {0}", in(reg) addr);
    }
}

unsafe extern "C" {
    fn __trap_entry();
}
