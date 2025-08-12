#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <unistd.h>
#include <string.h>

int main() {
    size_t size = 4096; // 映射 1 页内存（通常 4KB）
    void *addr = mmap(
        NULL,               // 由内核选择地址
        size,               // 映射大小
        PROT_READ | PROT_WRITE, // 可读可写
        MAP_PRIVATE | MAP_ANONYMOUS, // 私有匿名映射（不关联文件）
        -1,                 // 匿名映射时文件描述符为 -1
        0                   // 偏移量
    );
    if (addr == MAP_FAILED) {
        perror("mmap failed");
        return 1;
    }
    printf("Mapped memory at: %p\n", addr);
    if (munmap(addr, size) ){
        perror("munmap failed");
        return 1;
    }
    printf("Memory unmapped successfully.\n");
    printf("mmap-test2 passed!\n");
    return 0;
}