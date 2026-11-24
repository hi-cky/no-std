//! 周期定时中断冒烟测试：回调只更新计数，主程序检查并输出结果。

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};
use no_std::{println, system, timer};

static TICKS: AtomicUsize = AtomicUsize::new(0);

fn tick() {
    // 中断里只做短小的原子更新，避免打印或获取其他模块的锁。
    TICKS.fetch_add(1, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub fn main() -> ! {
    let start = timer::get_time();
    timer::init(10, tick);
    // 保持主程序运行，确认中断能打断程序并多次正常返回。
    while TICKS.load(Ordering::Relaxed) < 3 {
        assert!(
            timer::get_time() - start < timer::CLOCK_FREQ,
            "timer timeout"
        );
        core::hint::spin_loop();
    }
    assert!(timer::get_time() - start >= 30 * (timer::CLOCK_FREQ / 1000));

    // 再次初始化应替换配置，并从新期限开始计时。
    timer::init(20, tick);
    let count = TICKS.load(Ordering::Relaxed);
    let restart = timer::get_time();
    while TICKS.load(Ordering::Relaxed) < count + 3 {
        assert!(
            timer::get_time() - restart < timer::CLOCK_FREQ,
            "timer restart timeout"
        );
        core::hint::spin_loop();
    }
    println!("timer_test passed: periodic callback and reinitialization");
    system::shutdown()
}
