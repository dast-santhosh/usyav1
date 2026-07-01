global start
extern kernel_main

section .text
bits 32
start:
    ; 1. Set up a basic stack
    mov esp, stack_top

    ; NOTE: Page tables (Identity mapping) and Long Mode enablement 
    ; (PAE, EFER.LME, CR0.PG) must be initialized before this GDT jump
    ; if booting from a raw 32-bit Multiboot state.

    ; 2. Load the 64-bit GDT
    lgdt [gdt64.pointer]

    ; 3. Jump to long mode (64-bit) by doing a far jump to the code segment
    jmp gdt64.code:long_mode_start

bits 64
long_mode_start:
    ; 4. Clear all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; 5. Call the Rust kernel_main
    call kernel_main

    ; 6. If kernel_main returns, halt the CPU to prevent undefined behavior
    cli
.hang:
    hlt
    jmp .hang

section .rodata
align 8
; Global Descriptor Table (64-bit)
gdt64:
    dq 0 ; Zero entry (mandatory)
.code: equ $ - gdt64
    ; Executable (43), User Segment (44), Present (47), 64-bit flag (53)
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) 
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .bss
align 4096
stack_bottom:
    resb 16384 ; 16 KB stack
stack_top:
