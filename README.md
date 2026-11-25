# 🦀 Rust RISC-V 多任务内核

一个使用 Rust `no_std` 实现、运行于 QEMU RISC-V `virt` 平台的单核抢占式多任务内核，作为简易 RTOS 的原型。项目采用 lib + bin 结构，通过独立示例验证各模块。

目前支持线程创建、协作式与定时抢占式调度、线程等待与唤醒、堆内存管理，以及单核中断临界区保护。当前还没有最坏响应时间和截止期限的保证，线程栈回收等已知问题见文末待办。

## 🏗️ 项目结构

```
src/
├── lib.rs              # 库模块声明和项目概述
├── console.rs          # 串口控制台输出模块
├── error.rs            # 错误处理和 panic 处理器
├── system.rs           # 系统功能（关机、重启、内存布局等）
├── logging.rs          # 日志输出
├── heap/               # 堆分配器及分配、释放时的中断保护
├── thread/             # 线程接口、句柄、TCB、调度器和切换汇编
├── trap/               # 异常与中断入口、InterruptGuard
├── timer.rs            # 时间读取与周期定时中断
├── collection/         # 链表等数据结构
├── entry.asm           # 系统启动汇编代码
└── bin/                # 应用程序目录
    ├── helloworld.rs   # Hello World 示例应用
    ├── heaptest.rs     # 堆内存测试
    ├── linked_list.rs  # 链表测试
    ├── thread_test.rs  # 多线程调度与 join 测试
    └── timer_test.rs   # 定时回调与嵌套中断保护测试
```

## 📦 核心模块

### 🖥️ 控制台模块 (`console.rs`)

- **功能**: 基于 QEMU virt 平台的 UART 串口输出
- **特性**:
  - 支持格式化打印 (`print!`, `println!`)
  - 直接寄存器操作，无依赖
  - 自动换行处理

### 🚨 错误处理模块 (`error.rs`)

- **功能**: 统一的错误处理和 panic 处理
- **特性**:
  - 全局 panic 处理器
  - 友好的错误信息输出
  - 系统安全关闭

### 🖥️ 系统功能模块 (`system.rs`)

- **功能**: 系统级功能
- **特性**:
  - 系统关机/重启
  - 内存布局打印
  - BSS 段清理
  - 内存段地址管理

### 🌱 堆内存分配器（[heap/](src/heap/)）

- **功能**: 基于 buddy_system_allocator 的堆管理
- **特性**:
  - 1MB 堆内存空间
  - 支持 Box 和 Vec 等动态分配
  - 内存分配错误处理
  - 分配、释放前关闭中断，释放堆锁后恢复原中断状态

### 🧵 线程模块（[thread/](src/thread/)）

- 每线程独立的 8 KiB 堆分配栈，TCB 记录保存现场的栈指针；整数切换现场及 `mepc`、`mstatus` 保存在线程栈上。
- 支持创建线程、通过句柄 `start()` 将线程设为就绪、主动 `yield_now()`、`join()` 等待及退出唤醒。
- 调度器锁保护线程状态；访问调度器前关闭中断，实际切换前释放锁。
- 初始化时选择调度方式：`thread::init(main_thread, None)` 使用协作式调度；`thread::init(main_thread, Some(10))` 启用 10 毫秒周期的定时抢占。
- `delay(ms)` 原地等待，不主动切换；`sleep(ms)` 在等待期间反复让出 CPU，目前不是由定时器到期唤醒的阻塞式睡眠。

### ⏱️ 定时器与中断（[timer.rs](src/timer.rs)、[trap/](src/trap/)）

- `timer::get_time()` 返回原始 tick，时钟频率为 10 MHz。
- `timer::init(interval_ms, callback)` 设置周期及回调，配置中断入口并开启定时中断。
- `InterruptGuard` 创建时保存并关闭全局中断，离开作用域时恢复原 MIE 状态，支持嵌套。
- 定时器只有一个回调槽位；线程抢占调度也使用这个定时器，再次初始化会替换原配置。
- 当前面向单核机器模式，不支持浮点、向量寄存器的上下文保存。

## 🚀 快速开始

### 环境要求

- Rust 工具链
- QEMU (qemu-system-riscv64)
- RISC-V 目标: `riscv64gc-unknown-none-elf`

### 安装目标

```bash
rustup target add riscv64gc-unknown-none-elf
```

### 编译项目

```bash
# 编译所有应用
make build

# 编译指定应用
make build-app APP=helloworld
```

### 运行应用

```bash
# 运行默认应用 (helloworld)
make run

# 运行指定应用
make run APP=helloworld
make run APP=heaptest
make run APP=thread_test
make run APP=timer_test

# 调试模式运行
make debug APP=helloworld
```

### 查看可用应用

```bash
make list-apps
```

## 📋 应用程序

### 🌍 Hello World (`helloworld`)

- **功能**: 基础示例应用
- **演示**: 串口输出和系统关机
- **运行**: `make run APP=helloworld`

### 🧪 堆内存测试 (`heaptest`)

- **功能**: 堆内存分配器测试
- **演示**: Box 和 Vec 动态分配
- **运行**: `make run APP=heaptest`

### 🧵 多线程测试（[thread_test](src/bin/thread_test.rs)）

- 创建多个子线程，主线程使用 `join()` 等待它们结束后继续执行。
- 当前示例通过 `thread::init(main_thread, Some(10))` 启用抢占；改为 `None` 可验证协作式调度。
- **运行**：`make run APP=thread_test`

### ⏱️ 定时器测试（[timer_test](src/bin/timer_test.rs)）

- 验证周期回调、重新初始化、嵌套中断保护及保护结束后的中断恢复。
- **运行**：`make run APP=timer_test`

### 🔗 链表测试（[linked_list](src/bin/linked_list.rs)）

- **运行**：`make run APP=linked_list`

## 🗺️ 内存布局

项目使用自定义链接脚本 (`memory.x`) 定义内存布局：

```
0x80000000 ┌─────────────┐
           │   .text     │ 代码段
           ├─────────────┤
           │  .rodata    │ 只读数据段
           ├─────────────┤
           │   .data     │ 已初始化数据段
           ├─────────────┤
           │    .bss     │ 未初始化数据段
           ├─────────────┤
           │   .stack    │ 启动栈 (64KB)
           └─────────────┘
```

线程各自的 8 KiB 栈通过堆分配，不在这里的启动栈段中。

## 🔧 构建配置

### 目标平台

- **架构**: RISC-V 64-bit (RV64GC)
- **机器**: QEMU virt
- **内存**: 128MB RAM
- **启动**: 无 bootloader，直接加载

### 依赖

```toml
[dependencies]
buddy_system_allocator = "0.6"
lazy_static = { version = "1.5", default-features = false, features = ["spin_no_std"] }
spin = { version = "0.9", default-features = false, features = ["mutex", "spin_mutex"] }
log = "*"
```

## 🎯 输出示例

### Hello World 应用输出

```
Hello, World!
```

### 堆内存测试输出

```
🧹 清空 BSS 段:
   开始地址: 0x80007000
   结束地址: 0x80107129
   段大小: 1048873 字节
✅ BSS 段清空完成
📋 内存段布局信息:
==================
🔧 .text 段:
   开始地址: 0x80000000
   结束地址: 0x80004c34
📖 .rodata 段:
   开始地址: 0x80005000
   结束地址: 0x80006100
💾 .data 段:
   开始地址: 0x80007000
   结束地址: 0x80007000
🗑️ .bss 段:
   开始地址: 0x80007000
   结束地址: 0x80107129
📚 .stack 段:
   开始地址: 0x80108000
   结束地址: 0x80118000
   栈顶地址: 0x80118000
==================
heap_test passed!
```

## 🔍 调试

### GDB 调试

```bash
# 启动调试模式
make debug APP=helloworld

# 在另一个终端连接 GDB
make gdb APP=helloworld
```

### 查看 ELF 信息

```bash
# 查看段信息
rust-readobj --sections target/riscv64gc-unknown-none-elf/release/helloworld

# 查看符号表
rust-objdump -t target/riscv64gc-unknown-none-elf/release/helloworld
```

## 📝 开发指南

### 添加新应用

1. 在 `src/bin/` 目录下创建新的 `.rs` 文件
2. 实现 `main()` 函数
3. 使用 `make run APP=your_app` 运行

### 添加新模块

1. 在 `src/` 目录下创建新的 `.rs` 文件
2. 在 `src/lib.rs` 中声明模块
3. 在应用程序中导入使用

### 修改内存布局

编辑 `memory.x` 文件调整段大小和地址。

## 🐛 已知问题与待办

### [TODO] 线程切走后再回收其栈

- [ ] 线程退出时先标记终止、唤醒等待者并保留待回收的 TCB 和栈；完成切换后，由其他线程或空闲线程回收。

当前退出流程会销毁 TCB 并释放其拥有的栈，但 CPU 此时仍在这块栈上执行，存在内存安全问题。关闭中断不能解决这个生命周期问题，必须延迟到切走后再释放。

相关代码：[调度器](src/thread/scheduler.rs)、[线程控制块](src/thread/tcb.rs)。

### [TODO] 线程 ID 复用导致旧句柄指向新线程

当前调度器使用线程表的槽位索引作为线程 ID，线程退出后会清空槽位，后续新线程可以复用该 ID。线程句柄和 `waiting_for` 仅记录 ID，无法区分同一槽位先后创建的不同线程，存在身份混淆（ABA）问题。

例如：保留线程 A（ID 为 2）的句柄 → A 退出 → 线程 B 复用 ID 2 → 用 A 的旧句柄调用 `join`，实际会等待 B。

当前协作式流程中，已经阻塞等待 A 的线程会在 A 退出时被立即唤醒并清除等待关系；这里的问题是槽位复用后再使用旧句柄。其他通过旧句柄按 ID 操作线程的接口也存在相同风险。

后续可引入“槽位索引 + 代数（generation）”作为完整线程身份：槽位复用时递增代数，槽位清空后仍保留代数；线程句柄、等待关系以及查找和唤醒逻辑均校验完整身份。也可先采用不复用 ID 的简化方案。此问题暂缓修复。

相关代码：[调度器](src/thread/scheduler.rs)、[线程句柄](src/thread/thread_handle.rs)、[线程接口](src/thread/mod.rs)、[线程控制块](src/thread/tcb.rs)。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 MIT 许可证。

---

**作者**: [hi-cky](https://github.com/hi-cky)
**项目**: RISC-V 单核多任务内核
**语言**: Rust
**架构**: RISC-V 64-bit
