//! 🧷 Trap/中断处理模块
//!
//! 提供 RISC-V 机器模式 trap 入口与基本分发逻辑。
//! 定时中断交给 timer 处理，其他异常或中断记录后停机。
mod interrupt_guard;
pub use interrupt_guard::InterruptGuard;

use core::arch::{asm, global_asm};
use log::error;

use crate::timer;

// 引入汇编 trap 入口
global_asm!(include_str!("trap.S"));

/// 设置异常入口，并开启定时中断和全局中断。
///
/// 由 timer::init 在回调和触发时间配置完成后调用。
pub fn init() {
    disable_interrupts();
    set_mtvec(__trap_entry as *const () as usize);
    // 机器模式下访问中断控制寄存器：先开分类开关，最后开全局开关。
    unsafe {
        // mie.MTIE（第 7 位）：允许定时器中断。
        asm!("csrs mie, {}", in(reg) 1usize << 7);
        // mstatus.MIE（第 3 位）：允许接收机器模式中断。
        asm!("csrsi mstatus, 8");
    }
}

/// 暂时关闭全局中断，供定时器在更新配置前调用。
pub(crate) fn disable_interrupts() {
    // 运行于机器模式，清除 mstatus.MIE；不改变各类中断的独立开关。
    unsafe { asm!("csrci mstatus, 8") };
}

/// Rust 侧 trap 处理函数（由汇编入口调用）
///
/// 说明：
/// - 读取 mcause 判断中断类型
/// - 定时中断处理后返回，其余情况记录诊断信息后停机
#[unsafe(no_mangle)]
pub extern "C" fn trap_handler() {
    let cause = read_csr("mcause");
    // RV64 最高位为 1 表示中断，原因编号 7 为机器定时器中断。
    if cause == (1usize << 63) | 7 {
        timer::on_interrupt();
        return;
    }

    let tval = read_csr("mtval");
    let epc = read_csr("mepc");

    // 记录异常或中断并停机，避免返回后反复触发。
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
