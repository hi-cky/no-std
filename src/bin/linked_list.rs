#![no_std]
#![no_main]

use core::fmt::Display;

use no_std::{println, print};
use no_std::system;
use no_std::heap_allocator;

extern crate alloc;
use alloc::boxed::Box;

#[unsafe(no_mangle)]
pub fn main() -> ! {
    system::clear_bss();
    heap_allocator::init_heap();

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

    system::shutdown()
}

struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
}

impl<T> LinkedList<T> 
where T: Display{
    /// 创建新的空链表
    fn new() -> Self {
        Self { head: None }
    }

    /// 向链表头部添加元素 - O(1) 时间复杂度
    fn push(&mut self, data: T) {
        let new_node = Box::new(Node { 
            data, 
            next: self.head.take()  // 使用 take() 避免 clone
        });
        self.head = Some(new_node);
    }

    /// 从链表头部弹出元素 - O(1) 时间复杂度
    fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            let Node { data, next } = *node;
            self.head = next;
            data
        })
    }

    /// 在指定位置插入元素 - O(n) 时间复杂度
    fn insert(&mut self, data: T, index: u32) {
        if index <= 0 {
            self.push(data);
            return;
        }

        let mut curr = &mut self.head;
        
        // 找到插入位置的前一个节点
        for _ in 0..(index - 1) {
            match curr {
                Some(node) => curr = &mut node.next,
                None => return,  // 索引超出范围，直接返回
            }
        }
        
        // 安全地插入新节点
        if let Some(node) = curr {
            let new_node = Box::new(Node {
                data,
                next: node.next.take(),
            });
            node.next = Some(new_node);
        }
        // 如果 curr 是 None，说明索引超出范围，什么也不做
    }

    /// 获取链表长度 - O(n) 时间复杂度
    fn len(&self) -> usize {
        let mut count = 0;
        let mut current = &self.head;
        while let Some(node) = current {
            count += 1;
            current = &node.next;
        }
        count
    }

    /// 检查链表是否为空 - O(1) 时间复杂度
    fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// 获取指定位置的元素引用 - O(n) 时间复杂度
    fn get(&self, index: u32) -> Option<&T> {
        let mut current = &self.head;
        for _ in 0..index {
            match current {
                Some(node) => current = &node.next,
                None => return None,
            }
        }
        current.as_ref().map(|node| &node.data)
    }
    
    /// 打印链表内容
    fn print(&self) {
        let mut current = &self.head;
        print!("LinkedList[{}]: ", self.len());
        while let Some(node) = current {
            print!("{} -> ", node.data);
            current = &node.next;
        }
        println!("None");
    }
}