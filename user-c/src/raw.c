// 文件名: raw.c
// 编译: gcc -nostdlib -static raw.c -o raw

// 定义系统调用号和常量
#define SYS_WRITE 1       // sys_write 的系统调用号 (x86_64)
#define STDOUT_FILENO 1   // 标准输出的文件描述符

typedef unsigned long size_t; 

// 自定义系统调用封装 (x86_64 架构)
static long my_syscall(long num, long arg1, long arg2, long arg3) {
    long ret;
    asm volatile (
        "syscall"
        : "=a"(ret)
        : "a"(num), "D"(arg1), "S"(arg2), "d"(arg3)
        : "rcx", "r11", "memory"
    );
    return ret;
}

// 自定义的 "write" 函数
static void my_write(int fd, const char *buf, size_t count) {
    my_syscall(SYS_WRITE, fd, (long)buf, count);
}

// 程序入口 (_start 替代 main，因为无 libc)
void _start() {
    const char msg[] = "Hello, World!\n";
    my_write(STDOUT_FILENO, msg, sizeof(msg) - 1);  // 排除末尾的 '\0'

    // 直接调用 sys_exit 退出
    my_syscall(60, 0, 0, 0);  // SYS_EXIT = 60
}