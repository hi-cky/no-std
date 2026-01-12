extern crate alloc;
use alloc::boxed::Box;
use core::fmt::{Display, Formatter};
use core::mem;

use super::{STACK_SIZE, thread_entry};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadState {
    Uninit,
    Running,
    Ready,
    Blocked,
    Terminated,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ThreadContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

impl Display for ThreadContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ThreadContext {{ ra: 0x{:x}, sp: 0x{:x}, s: [{:?}] }}",
            self.ra, self.sp, self.s
        )
    }
}

pub struct TCB {
    // Fields for the TCB
    pub id: usize,
    pub state: ThreadState,
    pub context: ThreadContext,
    /// 线程要执行的任务（闭包）
    ///
    /// 说明：
    /// - `FnOnce` 表示只能执行一次（执行时会 move 掉闭包本身）
    /// - 用 `Option` 包一层，便于在 `run()` 里通过 `take()` 安全地取出并执行
    /// - 加 `Send` 是为了后续能把调度器放进全局静态（例如 `lazy_static`），满足类型约束
    pub job: Option<Box<dyn FnOnce() + Send + 'static>>,

    /// join 等待目标线程的 id；None 表示未阻塞等待
    pub waiting_for: Option<usize>,

    pub stack: Box<[usize; STACK_SIZE]>,
}

impl TCB {
    pub fn new(id: usize, job: Option<Box<dyn FnOnce() + Send + 'static>>) -> Self {
        let mut tcb = TCB {
            id,
            state: ThreadState::Uninit,
            context: ThreadContext::default(),
            job,
            waiting_for: None,
            stack: Box::new([0; STACK_SIZE]),
        };
        // 初始化线程上下文：
        // - ra 指向线程入口 trampoline（统一入口负责调用 job）
        // - sp 指向“栈顶”（RISC-V 栈向低地址增长），并按 16 字节对齐
        let sp_top = tcb.stack.as_ptr() as usize + STACK_SIZE * mem::size_of::<usize>();
        let sp_top_aligned = sp_top & !0xF;
        tcb.context = ThreadContext {
            ra: thread_entry as usize,
            sp: sp_top_aligned,
            s: [0; 12],
        };
        tcb
    }

    // Methods for the TCB
    pub fn run(&mut self) {
        if let Some(job) = self.job.take() {
            job();
        }
    }
}

// 实现一下display
impl Display for TCB {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "TCB {{ id: {}, state: {:?}, context: {:?}}}",
            self.id, self.state, self.context
        )
    }
}
