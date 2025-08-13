# musl-libc支持

C 语言标准库(libc)是用户程序的最底层 API。目前几乎所有的用户程序都建立在某个 libc 之上,极少直接和系统调用打交道。而不同的 libc 所使用的系统调用也有细微区别,因此选择一个合适的 libc 就尤其重要。我们平时最常用的是 glibc,它功能全面但是过于复杂,一个从零实现的系统难以支持。musl-libc 是一个轻量级、快速、标准兼容的 C 标准库（libc）实现，专为 嵌入式系统、容器化环境和高性能应用 设计。相比 GNU C 库（glibc），musl 更加精简，强调 代码简洁性、正确性及可维护性，适用于静态链接和资源受限的环境。因此支持musl-libc是一个很好的选择。

由于musl-libc依赖于Linux系统调用，因此我们选择直接实现Linux系统调用，只要再实现Linux的部分abi兼容，就可以支持musl-libc了。

我们在已有的Linux开发环境下下载并编译好musl-libc，并编写一个简单的hello程序：

```C
#include <stdio.h>

int main(){
    int a = 100;
    printf("Hello world!\n");
    return 0;
}

```
使用musl提供的musl-gcc脚本来编译这个c程序，采用静态链接：

```bash
musl-gcc -static -o bin/hello src/hello.c
```
得到的二进制文件可以直接在Linux开发环境中运行，我们希望能够不加修改的在nCore中运行这个elf文件。可以使用`strace`命令来查看这个elf文件使用的系统调用，因此想要在nCore中运行hello，需要实现这些系统调用。

```bash
strace ./hello 
execve("./hello", ["./hello"], 0x7ffca3042430 /* 65 vars */) = 0
arch_prctl(ARCH_SET_FS, 0x406658)       = 0
set_tid_address(0x406790)               = 62822
ioctl(1, TIOCGWINSZ, {ws_row=29, ws_col=182, ws_xpixel=0, ws_ypixel=0}) = 0
writev(1, [{iov_base="", iov_len=0}, {iov_base="\n", iov_len=1}], 2
) = 1
writev(1, [{iov_base="Hello world!, a = 100", iov_len=21}, {iov_base="\n", iov_len=1}], 2Hello world!, a = 100
) = 22
exit_group(0)                           = ?
+++ exited with 0 +++
```

因此，理论上只要实现了相应的系统调用，我们就可以运行Linux生态中的任意C语言用户程序了。