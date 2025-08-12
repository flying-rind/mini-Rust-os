#include <stdio.h>
#include <unistd.h>   // 提供 getcwd 系统调用
#include <stdlib.h>   // 提供 free 和 malloc
#include <limits.h>   // 定义 PATH_MAX（路径最大长度）
#define PATH_MAX 4096


int main() {
    // 方法1：使用固定大小的缓冲区（简单但可能不安全）
    char buf[PATH_MAX];
    if (getcwd(buf, sizeof(buf))) {
        printf("Current directory (fixed buffer): %s\n", buf);
    } else {
        perror("getcwd (fixed buffer) failed");
    }

    // 方法2：动态分配缓冲区（更灵活）
    char *cwd = NULL;
    long size = pathconf(".", _PC_PATH_MAX); // 获取路径最大长度
    if (size <= 0) {
        size = 4096; // 默认值（某些系统可能返回 -1）
    }

    cwd = malloc(size);
    if (cwd == NULL) {
        perror("malloc failed");
        return 1;
    }

    if (getcwd(cwd, size)) {
        printf("Current directory (dynamic buffer): %s\n", cwd);
    } else {
        perror("getcwd (dynamic buffer) failed");
    }

    free(cwd); // 释放动态内存
    return 0;
}