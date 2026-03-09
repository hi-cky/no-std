//! ⏱️ 计时器模块
//!
//! 提供 RISC-V 机器定时器（CLINT/mtime）相关的基础接口。
//! 目前基于 QEMU virt 的内存映射地址实现。

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::trap;

/// 系统时钟频率（Hz）
///
/// 说明：
/// - QEMU virt 默认 timebase 频率通常为 10MHz
/// - 如果平台频率不同，请在这里调整
pub const CLOCK_FREQ: usize = 10_000_000;

// QEMU virt 平台 CLINT 基地址（mtime/mtimecmp）
const CLINT_BASE: usize = 0x0200_0000;
// mtime 寄存器地址（64-bit）
const MTIME: usize = CLINT_BASE + 0xBFF8;
// mtimecmp 寄存器基地址（64-bit），每个 hart 占用 8 字节
const MTIMECMP_BASE: usize = CLINT_BASE + 0x4000;

/// 保存计时器中断处理函数指针（0 表示未设置）
static TIMER_INTERRUPT_HANDLER: AtomicUsize = AtomicUsize::new(0);

pub fn init(handler: fn()) {
    // 先注册处理函数并设置一个“安全”的初始触发时间，
    // 避免开启中断时 mtimecmp 仍为 0 导致立即进入中断并无法恢复。
    set_timer_interrupt_handler(handler);
    set_next_trigger(get_time().wrapping_add(1));
    trap::init();
}

/// 设置计时器中断处理函数
///
/// 说明：
/// - 这里只保存函数指针，不负责开启/关闭中断
/// - 中断处理函数必须是 `fn()` 类型（不捕获环境）
pub fn set_timer_interrupt_handler(handler: fn()) {
    // 使用原子写入避免并发场景下出现部分写入
    TIMER_INTERRUPT_HANDLER.store(handler as usize, Ordering::Release);
}

/// 获取系统时间（返回计数器数值）
///
/// 说明：
/// - 返回值单位是 timebase tick
/// - 对应 RISC-V `mtime` 寄存器的当前值
pub fn get_time() -> usize {
    // 读取 64-bit mtime
    let ticks = read_mtime();
    ticks as usize
}

/// 设置下一次计时器中断触发时间（绝对 tick）
///
/// 说明：
/// - 传入的是绝对时间点（timebase tick），不是相对增量
/// - 常见用法：`set_next_trigger(get_time() + delta)`
pub fn set_next_trigger(time: usize) {
    let now = read_mtime();
    // 确保写入时间点在未来，避免 mtimecmp <= mtime 导致中断一直 pending
    let mut next = time as u64;
    if next <= now {
        next = now.wrapping_add(1);
    }
    // 写入 64-bit mtimecmp（按 hart 选择对应 compare 寄存器）
    unsafe { write_volatile(mtimecmp_addr() as *mut u64, next) };
}

/// 从已注册的中断处理函数中调用（可选的内部工具）
///
/// 说明：
/// - 该函数未对外暴露；具体的 trap/中断入口可在需要时调用
#[allow(dead_code)]
pub(crate) fn call_timer_interrupt_handler() {
    let handler = TIMER_INTERRUPT_HANDLER.load(Ordering::Acquire);
    if handler != 0 {
        let handler = unsafe { core::mem::transmute::<usize, fn()>(handler) };
        handler();
    }
}

/// 读取 mtime（64-bit）
fn read_mtime() -> u64 {
    unsafe { read_volatile(MTIME as *const u64) }
}

/// 读取当前 hart id
fn read_mhartid() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("csrr {0}, mhartid", out(reg) hart_id);
    }
    hart_id
}

/// 获取当前 hart 对应的 mtimecmp 地址
fn mtimecmp_addr() -> usize {
    MTIMECMP_BASE + read_mhartid() * core::mem::size_of::<u64>()
}
