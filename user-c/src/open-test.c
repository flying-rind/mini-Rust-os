#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>      // 提供 openat 和相关标志（O_RDONLY 等）
#include <unistd.h>     // 提供 read, close, AT_FDCWD
#include <sys/stat.h>   // 提供文件状态检查
#include <errno.h>      // 错误处理

int main(int argc, char *argv[]) {
    const char *dir_path = "/";   // 目录路径
    const char *file_name = "dev";  // 文件名（相对于目录）

    // 1. 打开目录，获取文件描述符
    int dir_fd = open(dir_path, O_RDONLY | O_DIRECTORY);
    if (dir_fd == -1) {
        perror("open directory failed");
        return 1;
    }

    // 2. 使用 openat 打开目录下的文件
    int file_fd = openat(dir_fd, file_name, O_RDONLY);
    if (file_fd == -1) {
        perror("openat failed");
        close(dir_fd);
        return 1;
    }

    printf("File opened successfully via openat!\n");

    // 4. 关闭文件描述符
    close(file_fd);
    close(dir_fd);
    return 0;
}