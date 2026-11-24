# RISC-V 64 位线程上下文切换：保存当前线程，恢复下一线程。
#
# 参数遵循 RISC-V 调用约定：
#   a0 = 当前线程的 *mut ThreadContext（保存位置）
#   a1 = 下一线程的 *const ThreadContext（恢复来源）
# ThreadContext 使用 #[repr(C)]，且 RV64 的 usize 占 8 字节：
#   偏移 0：ra；偏移 8：sp；偏移 16..104：s0..s11。
# 修改 Rust 结构体字段顺序或类型时，必须同步修改这里的偏移。
#
# 这里按普通函数调用约定切换，只保存 ra、sp 和被调用者保存寄存器 s0..s11。
# a0..a7、t0..t6 属于调用者保存寄存器，由调用方按需保存。
# 本实现不保存浮点寄存器、向量寄存器或中断现场，不能直接用作抢占式切换。

.altmacro
# 将 sn 保存到当前线程上下文的 s[n]，前两个槽位留给 ra 和 sp。
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
# 从下一线程上下文的 s[n] 恢复 sn。
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm

    .globl __switch
    .type __switch, @function
__switch:
    # 1. 保存当前线程的栈指针和返回地址。
    #    ra 是当前线程调用 __switch 后继续执行的位置。
    sd sp, 8(a0)
    sd ra, 0(a0)
    # 2. 展开为 12 条 sd 指令，保存 s0..s11；这不是运行时循环。
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # 3. 恢复下一线程的返回地址和 s0..s11。
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # 4. 切换到下一线程的栈；此后不能再依赖当前线程的栈内容。
    ld sp, 8(a1)
    # 5. ret 跳转到刚恢复的 ra，而不是返回刚才调用本函数的线程。
    #    已运行的线程：从上次调用 __switch 的返回位置继续。
    #    新线程：TCB::new 将 ra 初始化为 thread_entry，从统一入口开始。
    ret
