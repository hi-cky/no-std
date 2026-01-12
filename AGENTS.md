# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust `no_std` bare‑metal project targeting QEMU `virt` on RV64GC (RISC‑V 64). Core code lives in `src/`, while runnable “apps” are Cargo binaries in `src/bin/`.

- `src/lib.rs`: crate root (`#![no_std]`, `global_asm!` entry wiring)
- `src/entry.asm`: boot entry code
- `src/console.rs`: UART console + `print!`/`println!`
- `src/system.rs`: shutdown/reboot, BSS clear, memory layout helpers
- `src/heap/`: heap allocator integration (`buddy_system_allocator`)
- `src/collection/`: data structures (e.g. linked list)
- `src/bin/*.rs`: application entrypoints (`helloworld`, `heaptest`, `linked_list`, …)
- `memory.x`: linker script; `.cargo/config.toml` passes `-Tmemory.x`

## Build, Test, and Development Commands

Toolchain is pinned to nightly via `rust-toolchain.toml`. The default target is set in `.cargo/config.toml` to `riscv64gc-unknown-none-elf`.

- `make build`: build release artifacts (all bins)
- `make run APP=helloworld`: run a specific app in QEMU
- `make debug APP=helloworld`: run QEMU paused with GDB stub (`tcp::1234`)
- `make gdb`: connect `riscv64-elf-gdb` to the stub
- `make list-apps`: list available apps from `src/bin/`
- `cargo clippy --target riscv64gc-unknown-none-elf --lib`: lint library code (matches `rust-analyzer.toml`)

## Coding Style & Naming Conventions

- Keep the crate `no_std`: avoid `std` and OS syscalls; prefer `core`/`alloc` patterns.
- Follow Rust naming: `snake_case` items, `UpperCamelCase` types, `SCREAMING_SNAKE_CASE` constants.
- Format with `cargo fmt` (rustfmt defaults). Keep `unsafe` blocks small and explain the hardware/ABI assumption.

## Testing Guidelines

This target cannot use the default Rust `test` harness. Validate changes via “smoke test” apps:
- Extend `src/bin/heaptest.rs` for allocator work.
- Add a new `src/bin/<case>.rs` to reproduce and verify behavior in QEMU.

## Commit & Pull Request Guidelines

- Commits: short, imperative subject; emoji prefixes are acceptable. Prefer a scope when useful (e.g., `app: add linked_list`).
- PRs: include a clear description (what/why), the command(s) you ran (e.g., `make run APP=...`), and paste relevant QEMU output. Link issues when applicable.

## Agent-Specific Notes

- 新增/修改代码请补充必要的中文注释（尤其是 `unsafe`、内存布局、寄存器地址相关逻辑）。
- Avoid introducing non-reproducible toolchain changes; keep patches minimal and target-focused.

