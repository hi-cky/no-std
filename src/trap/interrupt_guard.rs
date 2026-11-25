use core::arch::asm;

/// 单核机器模式下的中断保护：创建时关中断，离开作用域时恢复原状态。
/// 应先创建此 guard，再获取资源锁；资源锁必须先于它释放。
#[must_use = "请将 guard 绑定到变量，保持到需要保护的作用域结束"]
pub struct InterruptGuard {
    was_enabled: bool,
}

impl InterruptGuard {
    pub fn new() -> Self {
        let status: usize;
        // 一条指令同时读取原 mstatus 并清除 MIE，避免读与关之间的窗口。
        // 不使用 nomem，让编译器保留受保护内存操作与中断开关的顺序。
        unsafe { asm!("csrrci {}, mstatus, 8", out(reg) status) };
        Self {
            was_enabled: status & (1 << 3) != 0,
        }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        // 只恢复 MIE，不覆盖 mstatus 的其他位；嵌套保护会保持外层的关闭状态。
        unsafe {
            if self.was_enabled {
                asm!("csrsi mstatus, 8");
            } else {
                asm!("csrci mstatus, 8");
            }
        }
    }
}