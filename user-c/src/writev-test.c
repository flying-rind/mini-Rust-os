#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/uio.h>  // for writev and struct iovec
#include <unistd.h>   // for STDOUT_FILENO

int main() {
    // 定义要写入的多个字符串
    const char *str1 = "Hello, ";
    const char *str2 = "writev ";
    const char *str3 = "system call!\n";

    // 初始化 iovec 结构体数组（每个元素对应一个缓冲区）
    struct iovec iov[3];
    iov[0].iov_base = (void *)str1;  // 缓冲区指针
    iov[0].iov_len = strlen(str1);   // 缓冲区长度
    iov[1].iov_base = (void *)str2;
    iov[1].iov_len = strlen(str2);
    iov[2].iov_base = (void *)str3;
    iov[2].iov_len = strlen(str3);

    // 调用 writev 一次性写入所有字符串到标准输出（文件描述符 1）
    ssize_t bytes_written = writev(STDOUT_FILENO, iov, 3);
    if (bytes_written == -1) {
        perror("writev failed");
        return 1;
    }

    printf("\nTotal bytes written: %zd\n", bytes_written);
    return 0;
}