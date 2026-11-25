# RV64 线程切换：TCB 只保存 sp，寄存器现场放在线程自己的栈上。
# a0 = 当前线程 sp 字段的地址，a1 = 下一线程保存的 sp 值。
#
# 栈帧共 128 字节，保持 16 字节对齐（TCB::new 必须使用相同布局）：
#   0: ra；8..96: s0..s11；104: mepc；112: mstatus；120: 对齐填充。
# switch 是普通函数调用，调用者保存寄存器由 Rust 调用方按 ABI 处理；
# 从中断调用时，被打断的 ra、a0..a7、t0..t6 已由 trap.S 保存在更上层栈中。
# 当前不支持浮点、向量现场，也不保存定时器等系统共享寄存器。

.altmacro
.macro SAVE_SN n
    sd s\n, (\n+1)*8(sp)
.endm
.macro LOAD_SN n
    ld s\n, (\n+1)*8(sp)
.endm

    .section .text
    .globl __switch
    .type __switch, @function
__switch:
    # 读取本次切换前的 mstatus，同时关中断，保护换栈过程。
    # t0、t1 属于调用者保存的临时寄存器，可以用于搬运 CSR。
    csrrci t0, mstatus, 8
    addi sp, sp, -128
    sd ra, 0(sp)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    csrr t1, mepc
    sd t1, 104(sp)
    sd t0, 112(sp)

    # 存下当前现场的位置，然后直接换到下一线程的现场。
    sd sp, 0(a0)
    mv sp, a1

    ld ra, 0(sp)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    ld t1, 104(sp)
    csrw mepc, t1
    ld t0, 112(sp)
    addi sp, sp, 128
    # 栈和普通寄存器全部恢复后，再恢复目标线程自己的中断开关状态。
    # 中断中的线程保持关中断，最终由其 mret 恢复；新线程初始 MIE 为 1。
    csrw mstatus, t0
    ret
