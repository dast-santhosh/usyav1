#ifndef LIBC_H
#define LIBC_H

#include <stdint.h>
#include <stddef.h>

// Basic POSIX types
typedef uint64_t pthread_t;

// System Calls
#define SYS_READ   0
#define SYS_WRITE  1
#define SYS_OPEN   2
#define SYS_MMAP   9
#define SYS_MUNMAP 11
#define SYS_CLONE  56
#define SYS_EXIT   60

// Memory allocation
void* malloc(size_t size);
void free(void* ptr);

// Threading
int pthread_create(pthread_t *thread, const void *attr, void *(*start_routine) (void *), void *arg);

// Basic IO
int write(int fd, const void *buf, size_t count);
void exit(int status);

#endif // LIBC_H
