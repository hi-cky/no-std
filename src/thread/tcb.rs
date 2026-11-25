extern crate alloc;
use alloc::boxed::Box;
use core::fmt::{Display, Formatter};
use core::arch::asm;
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

pub struct TCB {
    // Fields for the TCB
    pub id: usize,
    pub state: ThreadState,
    /// 指向栈上保存的切换现场，布局见 switch.S。
    pub sp: usize,
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
            sp: 0,
            job,
            waiting_for: None,
            stack: Box::new([0; STACK_SIZE]),
        };
        // 新线程预先构造与 switch.S 相同的现场，让首次切换也走普通恢复流程。
        let stack_base = tcb.stack.as_ptr() as usize;
        let sp_top = stack_base + STACK_SIZE * mem::size_of::<usize>();
        let sp_top_aligned = sp_top & !0xF;
        // 共 16 个 usize（128 字节）：ra、s0..s11、mepc、mstatus、对齐填充。
        let frame = (sp_top_aligned - stack_base) / mem::size_of::<usize>() - 16;
        let status: usize;
        // 当前运行于机器模式；继承状态位，但新线程的 MIE 必须开启。
        unsafe { asm!("csrr {}, mstatus", out(reg) status) };
        tcb.stack[frame] = thread_entry as *const () as usize;
        tcb.stack[frame + 14] = status | (1 << 3);
        // 其他槽位已由栈的零初始化填好；恢复后 sp 会回到对齐的栈顶。
        tcb.sp = stack_base + frame * mem::size_of::<usize>();
        tcb
    }
}

// 实现一下display
impl Display for TCB {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "TCB {{ id: {}, state: {:?}, sp: 0x{:x}}}",
            self.id, self.state, self.sp
        )
    }
}
