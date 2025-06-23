# 🦀 RISC-V 裸机操作系统

一个基于 Rust 的 no_std 裸机操作系统项目，运行在 RISC-V 64 位架构上。项目采用 lib + bin 结构，支持多个应用程序。

## 🏗️ 项目结构

```
src/
├── lib.rs              # 库模块声明和项目概述
├── console.rs          # 串口控制台输出模块
├── error.rs            # 错误处理和 panic 处理器
├── system.rs           # 系统功能（关机、重启、内存布局等）
├── heap_allocator.rs   # 堆内存分配器
├── entry.asm           # 系统启动汇编代码
└── bin/                # 应用程序目录
    ├── helloworld.rs   # Hello World 示例应用
    └── heaptest.rs     # 堆内存测试应用
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

### 🌱 堆内存分配器 (`heap_allocator.rs`)
- **功能**: 基于 buddy_system_allocator 的堆管理
- **特性**:
  - 1MB 堆内存空间
  - 支持 Box 和 Vec 等动态分配
  - 内存分配错误处理

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
make build APP=helloworld
```

### 运行应用
```bash
# 运行默认应用 (helloworld)
make run

# 运行指定应用
make run APP=helloworld
make run APP=heaptest

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
           │   .stack    │ 栈内存 (64KB)
           └─────────────┘
```

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
make gdb
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

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 MIT 许可证。

---

**作者**: [hi-cky](https://github.com/hi-cky)  
**项目**: RISC-V 裸机操作系统  
**语言**: Rust  
**架构**: RISC-V 64-bit 