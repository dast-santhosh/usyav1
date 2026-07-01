#include "libc.h"

// Raw syscall invocation using sysv64 ABI
static inline uint64_t syscall(uint64_t n, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5, uint64_t a6) {
    uint64_t ret;
    asm volatile (
        "mov %1, %%rax\n"
        "mov %2, %%rdi\n"
        "mov %3, %%rsi\n"
        "mov %4, %%rdx\n"
        "mov %5, %%r10\n"
        "mov %6, %%r8\n"
        "mov %7, %%r9\n"
        "syscall\n"
        "mov %%rax, %0\n"
        : "=r"(ret)
        : "r"(n), "r"(a1), "r"(a2), "r"(a3), "r"(a4), "r"(a5), "r"(a6)
        : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "rcx", "r11", "memory"
    );
    return ret;
}

int write(int fd, const void *buf, size_t count) {
    return (int)syscall(SYS_WRITE, (uint64_t)fd, (uint64_t)buf, (uint64_t)count, 0, 0, 0);
}

void exit(int status) {
    syscall(SYS_EXIT, (uint64_t)status, 0, 0, 0, 0, 0);
    while (1) {} // Should not reach here
}

void* malloc(size_t size) {
    // Syscall 9: sys_mmap
    // Simplified: passing size, returning address.
    uint64_t addr = syscall(SYS_MMAP, (uint64_t)size, 0, 0, 0, 0, 0);
    if (addr == 0 || addr == (uint64_t)-1) {
        return NULL;
    }
    return (void*)addr;
}

void free(void* ptr) {
    // Syscall 11: sys_munmap
    // Simplified: passing pointer. The kernel will need to track the size.
    syscall(SYS_MUNMAP, (uint64_t)ptr, 0, 0, 0, 0, 0);
}

int pthread_create(pthread_t *thread, const void *attr, void *(*start_routine) (void *), void *arg) {
    // Syscall 56: sys_clone
    // Passes the function pointer and argument.
    uint64_t tid = syscall(SYS_CLONE, (uint64_t)start_routine, (uint64_t)arg, 0, 0, 0, 0);
    if (tid == (uint64_t)-1) {
        return -1;
    }
    if (thread) {
        *thread = tid;
    }
    return 0;
}
