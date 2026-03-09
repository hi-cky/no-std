//! 🦀 RISC-V 裸机操作系统内核
//!
//! 这是一个基于 Rust 的 no_std 裸机操作系统项目，运行在 RISC-V 64 位架构上。
//! 项目采用 lib + bin 结构，支持多个应用程序。
//!
//! ## 项目结构
//! - `console.rs` - 串口控制台输出
//! - `error.rs` - 错误处理模块
//! - `system.rs` - 系统功能（关机、重启、内存布局等）
//! - `heap_allocator.rs` - 堆内存分配器
//! - `bin/` - 应用程序目录

#![no_std]
#![feature(alloc_error_handler)]

// 设置不用test模块

// no_std 环境下显式引入 alloc，并对外提供一个稳定路径供宏展开使用
pub extern crate alloc as __alloc;

use core::arch::global_asm;

// 包含汇编入口点
global_asm!(include_str!("entry.asm"));

// 导出核心模块
pub mod collection;
pub mod console;
pub mod error;
pub mod heap;
pub mod logging;
pub mod system;
pub mod thread;
pub mod timer;
pub mod trap;
