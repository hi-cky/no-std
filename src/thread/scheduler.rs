extern crate alloc;
use super::tcb::{TCB, ThreadContext, ThreadState};
use crate::system;
use alloc::{boxed::Box, vec::Vec};
use log::{info, warn};

pub struct Scheduler {
    // Fields for the Scheduler
    pub current: Option<usize>,
    /// 线程表：索引即线程 id，空槽位用 None 表示（便于复用）
    threads: Vec<Option<TCB>>,
    /// 当没有当前线程时的“占位上下文”，避免引用临时值
    idle_context: ThreadContext,
}

impl Scheduler {
    // Methods for the Scheduler
    pub fn new() -> Self {
        Scheduler {
            current: None,
            threads: Vec::new(),
            idle_context: ThreadContext::default(),
        }
    }

    /// 添加一个线程，返回线程 id
    pub fn add_thread(&mut self, job: Box<dyn FnOnce() + Send + 'static>) -> usize {
        // 线程 id 使用“空槽位优先”的策略，避免 id 无限增长
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
            info!("Thread {} switched", thread_id);
            info!("ThreadState {:?}", thread.state);
        }
    }

    pub fn block_thread(&mut self, current_id: usize, target_id: usize) {
        // 只有当目标线程存在且没有结束时才阻塞
        if let Some(target) = self.find_thread(target_id) {
            if target.state == ThreadState::Terminated {
                return;
            }
        } else {
            return;
        }
        if let Some(current) = self.get_thread(current_id) {
            current.state = ThreadState::Blocked;
            current.waiting_for = Some(target_id);
        }
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
        info!("ThreadContext {}", thread.context);
        info!("Threads Count {}", self.threads.iter().filter(|t| t.is_some()).count());

        // 当前线程已退出，清空 current，避免后续调度访问到已被清理的槽位
        if self.current == Some(thread_id) {
            self.current = None;
        }

        // 唤醒等待指定线程结束的所有线程
        for slot in self.threads.iter_mut() {
            if let Some(t) = slot.as_mut() {
                if t.state == ThreadState::Blocked && t.waiting_for == Some(thread_id) {
                    t.state = ThreadState::Ready;
                    t.waiting_for = None;
                }
            }
        }
    }

    pub fn run_next(&mut self) {
        unsafe extern "C" {
            pub fn __switch(
                current_thread_cx_ptr: *mut ThreadContext,
                next_thread_cx_ptr: *const ThreadContext,
            );
        }

        let current_id = self.current;
        // 注意：这是“顺序执行版”的跑法（还没接上下文切换），只用于先把基础结构跑通
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
        info!("");
        info!("Running thread {}", next_thread.id);
        info!("ThreadState {:?}", next_thread.state);
        info!("ThreadContext {}", next_thread.context);
        let next_thread_cx_ptr = &next_thread.context as *const ThreadContext;
        self.current = Some(next_id);

        let current_thread_cx = match current_id {
            Some(id) => &mut self.get_thread(id).unwrap().context,
            None => {
                // 如果没有当前线程，使用调度器内的占位上下文
                &mut self.idle_context
            }
        };

        unsafe {
            __switch(current_thread_cx, next_thread_cx_ptr);
        }
    }

    /// 找到第一个就绪线程的 id（优先找其他线程）（只读遍历，避免把整个 `&mut self` 借用住）
    fn find_ready_thread_id(&self) -> Option<usize> {
        // 需求：优先挑选“非当前线程”的 Ready；若没有，再考虑当前线程是否 Ready
        let current_id = self.current;

        // 1) 先找其它就绪线程
        let other_ready = self.threads.iter().enumerate().find_map(|(id, slot)| {
            slot.as_ref().and_then(|t| {
                if t.state == ThreadState::Ready && Some(t.id) != current_id {
                    Some(id)
                } else {
                    None
                }
            })
        });
        if other_ready.is_some() {
            return other_ready;
        }

        // 2) 找不到其它就绪线程：如果当前线程仍是 Ready，就返回当前线程
        match current_id {
            Some(id)
                if self
                    .find_thread(id)
                    .map(|t| t.state == ThreadState::Ready)
                    .unwrap_or(false) =>
            {
                Some(id)
            }
            _ => None,
        }
    }

    pub(crate) fn get_thread(&mut self, thread_id: usize) -> Option<&mut TCB> {
        self.threads.get_mut(thread_id).and_then(|t| t.as_mut())
    }

    pub(crate) fn find_thread(&self, thread_id: usize) -> Option<&TCB> {
        self.threads.get(thread_id).and_then(|t| t.as_ref())
    }
}
