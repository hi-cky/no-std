//! 🦀 测试 Rust 链表实现
//! 
//! 测试链表的创建、插入、获取、弹出等操作

#![no_std]
#![no_main]

use no_std::logging;
use no_std::println;
use no_std::system;
use no_std::heap;
use no_std::collection::linked_list::LinkedList;

#[unsafe(no_mangle)]
pub fn main() -> ! {
    logging::init();
    // system::clear_bss(); // 没必要
    heap::init_heap();

    linked_list();

    system::shutdown()
}

fn linked_list() {
    println!("🦀 测试 Rust 链表实现");
    
    let mut list = LinkedList::new();
    println!("创建空链表: 长度 = {}, 是否为空 = {}", list.len(), list.is_empty());
    
    // 测试 push
    list.push(1);
    list.push(2);
    list.push(3);
    println!("Push 1, 2, 3 后:");
    list.print();
    
    // 测试 insert
    list.insert(4, 1);  // 在位置1插入4
    list.insert(5, 0);  // 在位置0插入5 (头部)
    list.insert(6, 5);  // 在位置5插入6 (尾部)
    println!("Insert 操作后:");
    list.print();
    
    // 测试 get
    if let Some(value) = list.get(2) {
        println!("位置 2 的元素: {}", value);
    }
    
    // 测试 pop
    if let Some(popped) = list.pop() {
        println!("Pop 元素: {}", popped);
    }
    println!("Pop 后:");
    list.print();
    
    println!("最终状态: 长度 = {}, 是否为空 = {}", list.len(), list.is_empty());
}
