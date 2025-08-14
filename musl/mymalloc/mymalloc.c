#include <stdlib.h>
#include <stddef.h>
#include <errno.h>
#include "pthread_impl.h"

#define malloc __libc_malloc_impl
#define realloc __libc_realloc
#define free __libc_free

// #define SYS_malloc 10086
// #define SYS_realloc 10087
// #define SYS_free 10088
#define SYS_aligned_alloc 10089

// extern long ky_malloc(size_t);
// extern long ky_realloc(void *, size_t);
// extern void ky_free(void *);
// extern long ky_aligned_alloc(size_t, size_t);

long mymalloc(int n);

void *malloc(size_t n)
{
    long ptr = mymalloc(n);
    if (ptr <= 0) {
        errno = -ptr;
        return 0;
    }
    return (void *)ptr;
}

long myrealloc(void* p, size_t n);

void *realloc(void *p, size_t n)
{
    // long ptr = ky_realloc(p, n);
    long ptr = myrealloc(p, n);
    if (ptr <= 0) {
        errno = -ptr;
        return 0;
    }
    return (void *)ptr;
}

void myfree(void* p);

void free(void *p)
{
    myfree(p);
}

void *aligned_alloc(size_t align, size_t len)
{
    // long ptr = ky_aligned_alloc(align, len);
    long ptr = __syscall(SYS_aligned_alloc, align, len);
    if (ptr <= 0) {
        errno = -ptr;
        return 0;
    }
    return (void *)ptr;
}

void __malloc_donate(char *start, char *end)
{
    // we do nothing
}