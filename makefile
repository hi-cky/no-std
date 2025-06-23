# 🛠️ RISC-V 裸机系统构建脚本
# 
# 支持 lib + bin 结构的项目构建和运行
# 用法: make run APP=helloworld

# 工具链配置
QEMU = qemu-system-riscv64
RUSTC = cargo
TARGET = riscv64gc-unknown-none-elf
BUILD_DIR = target/$(TARGET)/release

# 默认应用名
APP ?= helloworld

# 根据应用名构建目标文件路径
KERNEL = $(BUILD_DIR)/$(APP)

# 🚀 运行应用程序
# 无bootloader，纯裸机开发
run: build
	$(QEMU) \
	-machine virt \
	-bios none \
	-nographic \
	-kernel $(KERNEL) \
	-serial mon:stdio

# 运行指定应用
# 用法: make run APP=helloworld
# 用法: make run APP=myapp

# 🐛 调试模式运行
debug: build
	$(QEMU) \
	-machine virt \
	-bios none \
	-nographic \
	-kernel $(KERNEL) \
	-serial mon:stdio \
	-S \
	-gdb tcp::1234

# 🔨 构建所有应用
build:
	$(RUSTC) build --release

# 构建指定应用
# 用法: make build APP=helloworld
build-app:
	$(RUSTC) build --release --bin $(APP)

# 🧹 清理构建产物
clean:
	cargo clean

# 📋 列出所有可用的应用
list-apps:
	@echo "📋 可用的应用:"
	@find src/bin -name "*.rs" -exec basename {} .rs \; | sort

# 🔍 GDB 调试器连接
gdb:
	riscv64-elf-gdb \
		-ex 'file $(KERNEL)' \
		-ex 'set arch riscv:rv64' \
		-ex 'target remote localhost:1234'

.PHONY: run build build-app clean gdb debug list-apps