//! 线程句柄：用于启动线程，以及传给 join 等接口。

use super::SCHEDULER;
use crate::trap::InterruptGuard;

/// 句柄仅保存线程 ID，不拥有线程或线程栈。
#[derive(Debug, Clone, Copy)]
pub struct ThreadHandle {
    // 允许父模块构造句柄和查询 ID，外部调用者不能直接修改。
    pub(super) id: usize,
}

impl ThreadHandle {
    /// 将线程标记为就绪，等待调度器选择它运行。
    pub fn start(self) {
        let _interrupt_guard = InterruptGuard::new();
        SCHEDULER.lock().yield_thread(self.id);
    }
}
