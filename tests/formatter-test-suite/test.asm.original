; ── CASE 1: Section and data declarations ────────────────────────────────
section .data
    msg         db  "Hello, World!", 10, 0
    msg_len     equ $ - msg
    name        db  "Alice", 0
    newline     db  10
    fmt_str     db  "Count: %d", 10, 0

section .bss
    buffer      resb    256
    counter     resq    1

; ── CASE 2: Text section — function with prologue/epilogue ────────────────
section .text
    global main
    extern printf
    extern exit

main:
    ; Function prologue
    push    rbp
    mov     rbp, rsp
    sub     rsp, 32

    ; ── CASE 3: System call (Linux x64) ──────────────────────────────────
    mov     rax, 1          ; sys_write
    mov     rdi, 1          ; stdout
    mov     rsi, msg        ; message
    mov     rdx, msg_len    ; length
    syscall

    ; ── CASE 4: Loop ──────────────────────────────────────────────────────
    mov     rcx, 10         ; loop counter
    xor     r8,  r8         ; accumulator = 0

.loop_start:
    add     r8, rcx
    dec     rcx
    jnz     .loop_start

    ; ── CASE 5: Function call ─────────────────────────────────────────────
    mov     rdi, fmt_str
    mov     rsi, r8
    xor     rax, rax
    call    printf

    ; ── CASE 6: Conditionals ──────────────────────────────────────────────
    cmp     r8, 50
    jge     .large
    jl      .small

.large:
    mov     rdi, large_msg
    jmp     .done

.small:
    mov     rdi, small_msg

.done:
    ; Function epilogue
    xor     rax, rax
    mov     rsp, rbp
    pop     rbp
    ret
