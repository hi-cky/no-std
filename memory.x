/* 🗺️ 内存布局链接脚本
 * 
 * 定义 RISC-V 64 位系统的内存布局：
 * - RAM: 128MB 内存空间，起始地址 0x80000000
 * - 各段按 4KB 对齐
 */

MEMORY {
    RAM : ORIGIN = 0x80000000, LENGTH = 128M
}

ENTRY(_start)

SECTIONS {
    . = 0x80000000;
    
    /* 代码段 */
    .text : {
        __TEXT_START = .;
        *(.text.entry) 
        *(.text .text.*)
        __TEXT_END = .;
    } > RAM
    
    /* 只读数据段 */
    .rodata : ALIGN(4K) {
        __RODATA_START = .;
        *(.rodata .rodata.*)
        __RODATA_END = .;
    } > RAM
    
    /* 已初始化数据段 */
    .data : ALIGN(4K) {
        __DATA_START = .;
        *(.data .data.*)
        __DATA_END = .;
    } > RAM
    
    /* 未初始化数据段 */
    .bss : ALIGN(4K) {
        __BSS_START = .;
        *(.bss .bss.*)
        *(COMMON)
        __BSS_END = .;
    } > RAM
    
    /* 栈内存段 */
    .stack : ALIGN(4K) {
        __STACK_START = .;
        . += 64K;
        __STACK_TOP = .;
        __STACK_END = .;
    } > RAM
    
    /* 丢弃不需要的段 */
    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}