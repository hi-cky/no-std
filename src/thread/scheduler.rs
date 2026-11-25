extern crate alloc;
use super::tcb::{TCB, ThreadState};
use crate::system;
use alloc::{boxed::Box, vec::Vec};
use log::{info, warn};

pub struct Scheduler {
    // Fields for the Scheduler
    pub current: Option<usize>,
    /// 线程表：索引即线程 id，空槽位用 None 表示（便于复用）
    threads: Vec<Option<TCB>>,
    /// 首次调度时保存启动栈的 sp。
    idle_sp: usize,
}

impl Scheduler {
    /// 构造空调度器，可用于静态初始化；此时不分配线程表内存。
    pub const fn new() -> Self {
        Scheduler {
            current: None,
            threads: Vec::new(),
            idle_sp: 0,
        }
    }

    /// 添加一个线程，返回线程 id
    pub fn add_thread(&mut self, job: Box<dyn FnOnce() + Send + 'static>) -> usize {
        // 线程 id 使用“空槽位优先”的策略，避免 id 无限增长
        // [TODO] 槽位复用会使旧句柄误指向新线程（ABA）；引入索引 + 代数，详见 README 已知问题。
        let id = self
            .threads
            .iter()
            .position(|slot| slot.is_none())
            .unwrap_or_else(|| {
                self.threads.push(None);
                self.threads.len() - 1
            });
        let new_thread = TCB::new(id, Some(job));
        self.threads[id] = Some(new_thread);
        id
    }

    pub fn yield_thread(&mut self, thread_id: usize) {
        if let Some(thread) = self.get_thread(thread_id) {
            thread.state = ThreadState::Ready;
        }
    }

    /// 返回是否真正将当前线程置为阻塞，目标已退出时无需等待。
    pub fn block_thread(&mut self, current_id: usize, target_id: usize) -> bool {
        // [TODO] 仅按 ID 查找无法识别已复用的槽位；句柄和 waiting_for 应保存并校验完整线程身份。
        // 只有当目标线程存在且没有结束时才阻塞
        if let Some(target) = self.find_thread(target_id) {
            if target.state == ThreadState::Terminated {
                return false;
            }
        } else {
            return false;
        }
        if let Some(current) = self.get_thread(current_id) {
            current.state = ThreadState::Blocked;
            current.waiting_for = Some(target_id);
            info!("Thread {} blocked, waiting for thread {}", current_id, target_id);
            info!("ThreadState {:?}", current.state);
            return true;
        }
        false
    }

    pub fn exit_thread(&mut self, thread_id: usize) {
        // 先从线程表里移除，完成真正的槽位清理
        let mut thread = match self.threads.get_mut(thread_id).and_then(|slot| slot.take()) {
            Some(t) => t,
            None => return,
        };

        // 标记状态并记录日志
        thread.state = ThreadState::Terminated;
        info!("Thread {} exited", thread.id);
        info!("ThreadState {:?}", thread.state);
        info!("Thread sp 0x{:x}", thread.sp);
        info!("Threads Count {}", self.threads.iter().filter(|t| t.is_some()).count());

        // 当前线程已退出，清空 current，避免后续调度访问到已被清理的槽位
        if self.current == Some(thread_id) {
            self.current = None;
        }

        // 唤醒等待指定线程结束的所有线程
        // [TODO] 引入代数后，按完整线程身份匹配 waiting_for，保持等待与唤醒使用同一身份规则。
        for slot in self.threads.iter_mut() {
            if let Some(t) = slot.as_mut() {
                if t.state == ThreadState::Blocked && t.waiting_for == Some(thread_id) {
                    t.state = ThreadState::Ready;
                    t.waiting_for = None;
                }
            }
        }
    }

    /// 更新调度状态并返回切换参数；实际换栈由调用方解锁后执行。
    pub fn prepare_switch(&mut self) -> Option<(*mut usize, usize)> {
        let current_id = self.current;
        // 选择下一线程，寄存器现场由 switch.S 保存在各自栈上。
        let next_id = match self.find_ready_thread_id() {
            Some(id) => id,
            None => {
                // 如果没有就绪线程，直接关机
                warn!("No ready threads, shutting down...");
                system::shutdown();
            }
        };

        let next_thread = self.get_thread(next_id).unwrap();
        next_thread.state = ThreadState::Running;

        // 仍选中当前线程时继续执行即可；旧 sp 指向上次现场，不能再次恢复它。
        if current_id == Some(next_id) {
            return None;
        }
        info!("Thread {:?} switched", current_id);
        info!("");
        info!("Running thread {}", next_thread.id);
        info!("ThreadState {:?}", next_thread.state);
        info!("Thread sp 0x{:x}", next_thread.sp);
        let next_sp = next_thread.sp;
        self.current = Some(next_id);

        let current_sp = match current_id {
            Some(id) => &mut self.get_thread(id).unwrap().sp,
            None => {
                // 没有当前线程时，只需暂存启动栈的 sp。
                &mut self.idle_sp
            }
        };

        // 裸指针不延长对 guard 的借用，允许调用方先解锁再切换。
        // 调用方保持中断关闭，保证解锁后到切换前地址有效、调度状态不被改变。
        return Some((current_sp as *mut usize, next_sp));
    }

    /// 从当前线程的下一槽位开始查找，最后检查当前线程；
    /// 没有当前线程时，从 0 号槽位开始。
    fn find_ready_thread_id(&self) -> Option<usize> {
        let len = self.threads.len();
        if len == 0 {
            return None;
        }

        let start = self.current.map_or(0, |id| (id + 1) % len);

        // 最多检查一圈，跳过空槽位和非就绪线程。
        for offset in 0..len {
            let id = (start + offset) % len;
            if let Some(thread) = &self.threads[id] {
                if thread.state == ThreadState::Ready {
                    return Some(id);
                }
            }
        }

        None
    }

    pub(crate) fn get_thread(&mut self, thread_id: usize) -> Option<&mut TCB> {
        self.threads.get_mut(thread_id).and_then(|t| t.as_mut())
    }

    pub(crate) fn find_thread(&self, thread_id: usize) -> Option<&TCB> {
        self.threads.get(thread_id).and_then(|t| t.as_ref())
    }
}
