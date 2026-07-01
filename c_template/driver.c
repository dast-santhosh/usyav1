#include "libc.h"

// Sample hardware driver running in Ring 3 C/C++ Space
// Features: Memory allocation and Thread spawning via Syscalls

void* worker_thread(void* arg) {
    const char* msg = "Worker Thread: Initializing virtual hardware...\n";
    write(1, msg, 48);
    
    // Simulate some work
    for (volatile int i = 0; i < 1000000; i++) {}
    
    const char* msg2 = "Worker Thread: Hardware ready.\n";
    write(1, msg2, 31);
    
    // In a real POSIX environment this would be pthread_exit
    return NULL;
}

void _start() {
    const char* start_msg = "Driver Main: Starting C-Based Hardware Driver in Ring 3\n";
    write(1, start_msg, 56);

    // 1. Allocate memory dynamically using syscall wrapper
    int* buffer = (int*)malloc(1024 * sizeof(int));
    if (buffer) {
        const char* malloc_msg = "Driver Main: Successfully allocated 4KB via sys_mmap!\n";
        write(1, malloc_msg, 54);
        
        // Write some data to prove memory is accessible
        buffer[0] = 42;
        
        free(buffer);
        const char* free_msg = "Driver Main: Memory freed via sys_munmap.\n";
        write(1, free_msg, 42);
    } else {
        const char* err_msg = "Driver Main: Allocation failed!\n";
        write(1, err_msg, 32);
    }

    // 2. Spawn a worker thread
    pthread_t thread;
    int res = pthread_create(&thread, NULL, worker_thread, NULL);
    if (res == 0) {
        const char* thread_msg = "Driver Main: Spawned worker thread via sys_clone.\n";
        write(1, thread_msg, 50);
    }

    // Wait for a bit (mocking pthread_join since we haven't implemented wait4/futex)
    for (volatile int i = 0; i < 5000000; i++) {}

    const char* exit_msg = "Driver Main: Exiting gracefully.\n";
    write(1, exit_msg, 33);
    exit(0);
}
