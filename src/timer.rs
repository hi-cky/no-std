//! QEMU virt 单核环境下的时间读取和周期定时中断。

use core::ptr::{read_volatile, write_volatile};

use crate::trap;

/// 每秒的 tick 数；10_000 tick 对应 1 毫秒。
pub const CLOCK_FREQ: usize = 10_000_000;

// 当前时间寄存器，以及第一个 CPU 的定时中断触发时间寄存器。
const MTIME: usize = 0x0200_BFF8;
const MTIMECMP: usize = 0x0200_4000;

// 保存间隔和回调。仅用于单核：修改时关闭中断，处理时也不会嵌套中断。
static mut INTERVAL_TICKS: usize = 0;
static mut HANDLER: fn() = || {};

/// 设置间隔（毫秒）和回调，并立即启动周期定时中断。
///
/// 回调应短小，不要阻塞、切换线程、使用浮点运算或再次初始化定时器。
/// 当前中断入口只保存整数寄存器，且本模块仅支持单核运行。
pub fn init(interval_ms: usize, handler: fn()) {
    assert!(interval_ms > 0, "timer interval must be positive");
    let ticks = interval_ms
        .checked_mul(CLOCK_FREQ / 1000)
        .expect("timer interval too large");

    // 先关中断，避免旧的定时中断在配置更新到一半时触发。
    trap::disable_interrupts();
    unsafe {
        // 单核且已关闭中断，这里不会与中断处理同时访问这两个变量。
        INTERVAL_TICKS = ticks;
        HANDLER = handler;
    }
    set_trigger(get_time() + ticks);
    // 触发时间和回调都准备好后，再设置入口并打开中断开关。
    trap::init();
}

/// 由 trap 调用：设置下一次触发时间，再执行保存的回调。
pub(crate) fn on_interrupt() {
    let now = get_time();
    // 硬件撤销中断请求可能有延迟；尚未到新期限时忽略重复通知。
    if now < unsafe { read_volatile(MTIMECMP as *const u64) as usize } {
        return;
    }
    // 中断进入时 CPU 自动关闭全局中断；单核环境下可直接读取配置。
    let (interval, handler) = unsafe { (INTERVAL_TICKS, HANDLER) };
    set_trigger(now + interval);
    handler();
}

/// 返回当前时间，单位为 tick，不是毫秒。
pub fn get_time() -> usize {
    // QEMU virt 的 RV64 环境下，此地址是对齐的 64 位硬件寄存器。
    // volatile 保证每次实际读取寄存器，不被编译器当成普通内存读取优化掉。
    unsafe { read_volatile(MTIME as *const u64) as usize }
}

/// 写入绝对触发时间；当前时间达到此值时，硬件请求定时中断。
fn set_trigger(time: usize) {
    // 此地址为 QEMU virt 第一个 CPU 的 64 位定时比较寄存器。
    unsafe { write_volatile(MTIMECMP as *mut u64, time as u64) };
}
