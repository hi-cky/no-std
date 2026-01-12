pub mod scheduler;
pub mod tcb;

extern crate alloc;
use alloc::boxed::Box;
use core::arch::global_asm;
use core::cell::UnsafeCell;

use scheduler::Scheduler;

global_asm!(include_str!("switch.S"));

struct GlobalScheduler(UnsafeCell<Option<Scheduler>>);
unsafe impl Sync for GlobalScheduler {}

static SCHEDULER: GlobalScheduler = GlobalScheduler(UnsafeCell::new(None));

pub static STACK_SIZE: usize = 1024;

fn sched() -> &'static mut Scheduler {
    unsafe {
        let slot = &mut *SCHEDULER.0.get();
        if slot.is_none() {
            *slot = Some(Scheduler::new());
        }
        slot.as_mut().unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThreadHandle {
    id: usize,
}

impl ThreadHandle {
    pub fn start(self) {
        sched().yield_thread(self.id);
    }
}

pub fn new_thread(job: impl FnOnce() + Send + 'static) -> ThreadHandle {
    let id = sched().add_thread(Box::new(job));
    ThreadHandle { id }
}

pub fn init(main_thread: impl FnOnce() + Send + 'static) {
    let h = new_thread(main_thread);
    h.start();
    sched().run_next();
}

pub fn current_thread() -> Option<ThreadHandle> {
    sched().current.map(|id| ThreadHandle { id })
}

/// 线程入口（trampoline）：从当前 TCB 取出 job 执行
pub(crate) extern "C" fn thread_entry() {
    let current_id = sched().current.expect("current thread not set");

    // 取出 job：离开该作用域后会释放对调度器的可变借用
    let job = {
        let t = sched()
            .get_thread(current_id)
            .expect("current thread not found");
        t.job.take()
    };

    // 在不持有 &mut Scheduler 的情况下执行 job，允许线程内再创建线程
    if let Some(job) = job {
        job();
    }

    // 线程退出，同时唤醒等待它的线程
    sched().exit_thread(current_id);

    // 切换到下一个就绪线程（通常不会返回）
    sched().run_next();
}

pub fn yield_now() {
    // 获取当前线程id
    let current_id = sched().current.expect("current thread not set");

    // 标记当前线程为就绪状态
    sched().yield_thread(current_id);

    // 切换到下一个就绪线程（通常不会返回）
    sched().run_next();
}

/// 等待指定线程结束（协作式 join）
///
/// 说明：
/// - 这是“阻塞式”的 join：当前线程会进入 Blocked，直到目标线程结束
/// - 依赖目标线程能运行并最终退出，否则当前线程会一直阻塞
/// - 若传入的是当前线程，直接返回，避免死等
pub fn join(handle: ThreadHandle) {
    // 获取当前线程id
    let current_id = sched().current.expect("current thread not set");

    if handle.id == current_id {
        return;
    }

    // 标记当前线程被目标阻塞
    sched().block_thread(current_id, handle.id);

    // 切换到下一个就绪线程
    sched().run_next();
}
