mod scheduler;
mod tcb;
mod thread_handle;

// 统一从 thread 模块导出，外部仍使用 thread::ThreadHandle。
pub use thread_handle::ThreadHandle;

extern crate alloc;
use crate::timer;
use crate::trap::InterruptGuard;
use alloc::boxed::Box;
use core::arch::global_asm;
use lazy_static::lazy_static;
use spin::Mutex;

use scheduler::Scheduler;

global_asm!(include_str!("switch.S"));

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

/// 每条线程的栈容量，单位为 usize；RV64 下共 8 KiB。
pub const STACK_SIZE: usize = 1024;

/// 传入主线程要执行的函数，设置抢占式调度轮换间隔时间，初始化线程模块
pub fn init(main_thread: impl FnOnce() + Send + 'static, interval_ms: Option<usize>) {
    // 可选操作，启用每隔一段时间触发线程切换的定时器实现抢占式调度
    if let Some(interval_ms) = interval_ms {
        timer::init(interval_ms, || yield_now());
    }

    let _interrupt_guard = InterruptGuard::new();
    new_thread(main_thread).start();

    let switch = SCHEDULER.lock().prepare_switch();
    switch_thread(switch);
}

pub fn new_thread(job: impl FnOnce() + Send + 'static) -> ThreadHandle {
    let _interrupt_guard = InterruptGuard::new();
    let id = SCHEDULER.lock().add_thread(Box::new(job));
    ThreadHandle { id }
}

pub fn current_thread() -> Option<ThreadHandle> {
    let _interrupt_guard = InterruptGuard::new();
    SCHEDULER.lock().current.map(|id| ThreadHandle { id })
}

/// 主动让出 CPU，允许其他就绪线程运行。
pub fn yield_now() {
    // 保护调度状态修改及换栈之间的整个区间；切回本线程后恢复原中断状态。
    let _interrupt_guard = InterruptGuard::new();
    let switch = {
        let mut scheduler = SCHEDULER.lock();
        if let Some(id) = scheduler.current {
            scheduler.yield_thread(id);
        }
        scheduler.prepare_switch()
    };

    switch_thread(switch);
}

/// 等待目标线程退出；目标已退出或目标是自己时直接返回。
pub fn join(handle: ThreadHandle) {
    let _interrupt_guard = InterruptGuard::new();
    let switch = {
        let mut scheduler = SCHEDULER.lock();
        let current_id = scheduler.current.expect("current thread not set");

        if handle.id == current_id {
            return;
        }
        if !scheduler.block_thread(current_id, handle.id) {
            return;
        }

        scheduler.prepare_switch()
    };

    switch_thread(switch);
}

pub fn delay(ms: usize) {
    let start = timer::get_time();
    let ticks = ms * (timer::CLOCK_FREQ / 1000);

    while timer::get_time().wrapping_sub(start) < ticks {
        core::hint::spin_loop();
    }
}

pub fn sleep(ms: usize) {
    let end_time = timer::get_time() + timer::CLOCK_FREQ * ms / 1000;

    while timer::get_time() < end_time {
        yield_now();
    }
}

/// 线程入口（trampoline）：从当前 TCB 取出 job 执行
pub(crate) extern "C" fn thread_entry() {
    // 取出任务后先释放锁、恢复中断，任务本身不在保护区内执行。
    let (current_id, job) = {
        let _interrupt_guard = InterruptGuard::new();
        let mut scheduler = SCHEDULER.lock();
        let id = scheduler.current.expect("current thread not set");
        let job = scheduler
            .get_thread(id)
            .expect("current thread not found")
            .job
            .take();
        (id, job)
    };

    if let Some(job) = job {
        job();
    }

    let _interrupt_guard = InterruptGuard::new();
    let switch = {
        let mut scheduler = SCHEDULER.lock();
        scheduler.exit_thread(current_id);
        scheduler.prepare_switch()
    };

    switch_thread(switch);
}

/// 在调度器锁已释放后执行切换；None 表示继续运行当前线程。
fn switch_thread(switch: Option<(*mut usize, usize)>) {
    unsafe extern "C" {
        pub fn __switch(current_sp: *mut usize, next_sp: usize);
    }
    if let Some((current_sp, next_sp)) = switch {
        unsafe {
            // 调用方已关中断并释放锁，保存位置在切换前不会被移动。
            // 第一个参数是保存当前 sp 的地址，第二个是下一线程的 sp 值。
            __switch(current_sp, next_sp);
        }
    }
}
