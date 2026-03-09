# src/trap/trap.S
#
# 机器模式 trap 入口（简化版）：保存必要寄存器 -> 调用 Rust trap_handler -> 恢复寄存器

    .section .text.trap
    .globl __trap_entry
    .type __trap_entry, @function
__trap_entry:
    # 保存调用者保存寄存器（简化：保存 ra 和 t0-t6, a0-a7）
    addi sp, sp, -(1 + 7 + 8)*8
    sd ra, 0*8(sp)
    sd t0, 1*8(sp)
    sd t1, 2*8(sp)
    sd t2, 3*8(sp)
    sd t3, 4*8(sp)
    sd t4, 5*8(sp)
    sd t5, 6*8(sp)
    sd t6, 7*8(sp)
    sd a0, 8*8(sp)
    sd a1, 9*8(sp)
    sd a2, 10*8(sp)
    sd a3, 11*8(sp)
    sd a4, 12*8(sp)
    sd a5, 13*8(sp)
    sd a6, 14*8(sp)
    sd a7, 15*8(sp)

    # 调用 Rust trap handler
    call trap_handler

    # 恢复寄存器
    ld ra, 0*8(sp)
    ld t0, 1*8(sp)
    ld t1, 2*8(sp)
    ld t2, 3*8(sp)
    ld t3, 4*8(sp)
    ld t4, 5*8(sp)
    ld t5, 6*8(sp)
    ld t6, 7*8(sp)
    ld a0, 8*8(sp)
    ld a1, 9*8(sp)
    ld a2, 10*8(sp)
    ld a3, 11*8(sp)
    ld a4, 12*8(sp)
    ld a5, 13*8(sp)
    ld a6, 14*8(sp)
    ld a7, 15*8(sp)
    addi sp, sp, (1 + 7 + 8)*8

    mret
