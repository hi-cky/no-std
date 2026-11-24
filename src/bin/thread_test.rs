//! 🦀 测试 Rust 线程 实现

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use no_std::heap;
use no_std::logging;
// 引入 no_std 提供的 format! 宏（内部依赖堆分配）
use no_std::format;
use no_std::println;
use no_std::system;
use no_std::thread;
use no_std::thread::delay;
use no_std::thread::sleep;
use no_std::timer;

#[unsafe(no_mangle)]
pub fn main() -> ! {
    logging::init();
    // system::clear_bss(); // 没必要
    // system::print_memory_layout();
    heap::init_heap();

    thread::init(main_thread);

    system::shutdown()
}

// fn main_thread() {
//     println!("hello");
// }

fn main_thread() {
    // 创建n条子线程
    let n = 3;
    let times = 5;
    let mut handles: Vec<thread::ThreadHandle> = Vec::new();
    for i in 1..=n {
        let thread_name = format!("thread{}", i);
        let thread = thread::new_thread(move || {
            task(&thread_name, times);
        });
        // println!("threads are going to start");
        thread.start();
        handles.push(thread);
    }
    println!("主线程添加所有线程完毕，等待子线程执行完毕");
    // 等待所有子线程结束后再开始主线程的任务
    handles.iter().for_each(|&handle| thread::join(handle));
    task("main_thread", times);
    println!("主线程结束")
}

fn task(thread_name: &str, times: usize) {
    for i in 1..=times {
        println!("我是 {}, result: {}, time: {}", thread_name, fib(i), timer::get_time());
        // sleep(100 * i);
        // delay(50);
    }
}

// 递归实现fib
fn fib(n: usize) -> usize {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}
