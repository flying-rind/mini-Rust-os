#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <unistd.h>
#include <string.h>

int main() {
    size_t size = 4096; // 映射 1 页内存（通常 4KB）
    const char *test_str = "Hello, mmap with musl-libc!";
    size_t str_len = strlen(test_str) + 1; // 包含 NULL 终止符

    // 1. 使用 mmap 分配匿名内存（可读可写，私有映射）
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

    // 2. 写入数据到映射区域
    memcpy(addr, test_str, str_len);
    printf("Wrote to memory: \"%s\"\n", (char *)addr);

    // 3. 验证读取
    if (strcmp(addr, test_str) == 0) {
        printf("Read verification passed.\n");
    } else {
        printf("Read verification failed!\n");
    }

    // 4. 释放内存
    if (munmap(addr, size) ){
        perror("munmap failed");
        return 1;
    }

    printf("Memory unmapped successfully.\n");
    return 0;
}