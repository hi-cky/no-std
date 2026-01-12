extern crate alloc;
use alloc::boxed::Box;

use crate::print;
use crate::println;
use core::fmt::Display;

struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    length: usize,
}

/// 链表的只读迭代器
///
/// 说明：
/// - `next` 保存“下一个要访问的节点引用”
/// - 每次 `next()` 返回当前节点的数据引用，然后推进到下一个节点
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // 推进迭代器指向下一个节点
            self.next = node.next.as_deref();
            &node.data
        })
    }
}

/// 链表的可变迭代器
///
/// 注意：
/// - 单链表的 `IterMut` 需要非常小心地推进引用，避免同时借用同一节点的多个字段导致冲突
/// - 这里的实现每次只返回一个节点的数据可变引用，并把“下一跳”的可变引用保存起来
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            // 先把 next 取出来，避免同时借用 `node.next` 与返回 `&mut node.data` 产生冲突
            let next = node.next.as_deref_mut();
            self.next = next;
            &mut node.data
        })
    }
}

impl<T> LinkedList<T> {
    /// 创建新的空链表
    pub fn new() -> Self {
        Self {
            head: None,
            length: 0,
        }
    }

    /// 向链表头部添加元素 - O(1) 时间复杂度
    pub fn push(&mut self, data: T) {
        let new_node = Box::new(Node {
            data,
            next: self.head.take(), // 使用 take() 避免 clone
        });
        self.head = Some(new_node);
        self.length += 1;
    }

    /// 从链表头部弹出元素 - O(1) 时间复杂度
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            let Node { data, next } = *node;
            self.head = next;
            self.length -= 1;
            data
        })
    }

    /// 在指定位置插入元素 - O(n) 时间复杂度
    pub fn insert(&mut self, data: T, index: u32) {
        if index <= 0 {
            self.push(data);
            return;
        }

        let mut curr = &mut self.head;

        // 找到插入位置的前一个节点
        for _ in 0..(index - 1) {
            match curr {
                Some(node) => curr = &mut node.next,
                None => return, // 索引超出范围，直接返回
            }
        }

        // 安全地插入新节点
        if let Some(node) = curr {
            let new_node = Box::new(Node {
                data,
                next: node.next.take(),
            });
            node.next = Some(new_node);
            self.length += 1;
        }
        // 如果 curr 是 None，说明索引超出范围，什么也不做
    }

    /// 获取链表长度 - O(1) 时间复杂度
    pub fn len(&self) -> usize {
        self.length
    }

    /// 检查链表是否为空 - O(1) 时间复杂度
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// 获取指定位置的元素引用 - O(n) 时间复杂度
    pub fn get(&self, index: u32) -> Option<&T> {
        let mut current = &self.head;
        for _ in 0..index {
            match current {
                Some(node) => current = &node.next,
                None => return None,
            }
        }
        current.as_ref().map(|node| &node.data)
    }

    /// 获取只读迭代器
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    /// 获取可变迭代器
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}


impl<T> LinkedList<T>
where
    T: Display,
{
    /// 打印链表内容
    pub fn print(&self) {
        let mut current = &self.head;
        print!("LinkedList[{}]: ", self.len());
        while let Some(node) = current {
            print!("{} -> ", node.data);
            current = &node.next;
        }
        println!("None");
    }
}
