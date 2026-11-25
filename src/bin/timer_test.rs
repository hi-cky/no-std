//! 周期定时中断冒烟测试：回调只更新计数，主程序检查并输出结果。

#![no_std]
#![no_main]

use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};
use no_std::trap::InterruptGuard;
use no_std::{heap, println, system, thread, timer};

static TICKS: AtomicUsize = AtomicUsize::new(0);

fn tick() {
    // 中断里只做短小的原子更新，避免打印或获取其他模块的锁。
    TICKS.fetch_add(1, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub fn main() -> ! {
    // 启动时 MIE 关闭；保护结束后不能擅自开启它。
    assert!(!interrupts_enabled());
    {
        let _guard = InterruptGuard::new();
        assert!(!interrupts_enabled());
    }
    assert!(!interrupts_enabled());
    heap::init_heap();

    let start = timer::get_time();
    timer::init(10, tick);
    {
        let _outer = InterruptGuard::new();
        let count = TICKS.load(Ordering::Relaxed);
        {
            let _inner = InterruptGuard::new();
            assert!(!interrupts_enabled());
            // 线程 API 和堆分配器的内部 guard 不能提前开启外层关闭的中断。
            assert!(thread::current_thread().is_none());
            let value = no_std::__alloc::boxed::Box::new(42usize);
            assert_eq!(*value, 42);
            drop(value);
        }
        assert!(!interrupts_enabled());
        // 跨过定时期限，中断请求可以挂起，但回调不能执行。
        thread::delay(25);
        assert_eq!(TICKS.load(Ordering::Relaxed), count);
    }
    assert!(interrupts_enabled());
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
    println!("timer_test passed: interrupt guards, periodic callback and reinitialization");
    system::shutdown()
}

/// 测试运行于机器模式，只读取全局中断开关 MIE。
fn interrupts_enabled() -> bool {
    let status: usize;
    unsafe { asm!("csrr {}, mstatus", out(reg) status) };
    status & (1 << 3) != 0
}
