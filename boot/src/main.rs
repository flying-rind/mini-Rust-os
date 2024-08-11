extern crate getopts;
use getopts::Options;
use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 获取内核ELF文件路径
    let kernel_path = if args.len() > 1 && Path::new(&args[1]).exists() {
        // 第一个参数为有效路径，说明是在kernel目录下执行cargo test，触发kernel/.cargo/config.toml中的runner
        // 参数由cargo设置，详见：https://doc.rust-lang.org/nightly/cargo/reference/config.html#targettriplerunner
        PathBuf::from(&args[1])
    } else {
        // 否则，说明是在boot目录下执行cargo run，读取kernel ELF文件目录
        PathBuf::from("../kernel/target/x86_64/debug/kernel")
    };

    // 创建UEFI启动镜像
    let uefi_path = PathBuf::from("target/uefi.img");
    bootloader::UefiBoot::new(&kernel_path)
        .create_disk_image(&uefi_path)
        .unwrap();

    // 创建BIOS启动镜像
    let bios_path = PathBuf::from("target/bios.img");
    bootloader::BiosBoot::new(&kernel_path)
        .create_disk_image(&bios_path)
        .unwrap();

    // 定义命令行参数
    let mut opts = Options::new();
    opts.optflag("", "bios", "use bios firmware");
    opts.optflag("", "uefi", "use uefi firmware (default)");
    opts.optflag("", "gdb", "use gdb debug");
    opts.optflag("h", "help", "print this help menu");

    // 解析命令行参数
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    // 打印帮助文档
    if matches.opt_present("h") {
        let program = args[0].clone();
        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    }

    // 准备qemu模拟器命令
    let mut qemu_cmd = std::process::Command::new("qemu-system-x86_64");

    // 根据命令行参数指定启动固件的类型
    if matches.opt_present("bios") {
        qemu_cmd
            .arg("-drive")
            .arg(format!("format=raw,file={}", bios_path.to_str().unwrap()));
    } else {
        qemu_cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        qemu_cmd
            .arg("-drive")
            .arg(format!("format=raw,file={}", uefi_path.to_str().unwrap()));
    }
    // 从文件fs.img启动
    qemu_cmd
        .arg("-drive")
        .arg("file=../user/target/x86_64/release/fs.img,if=none,id=fsimg");

    // 添加ahci设备
    qemu_cmd.arg("-device").arg("ahci,id=ahci0");

    // 添加硬盘
    qemu_cmd
        .arg("-device")
        .arg("ide-hd,drive=fsimg,bus=ahci0.0");

    //  添加virtio-gpu设备
    // qemu_cmd.arg("-device").arg("virtio-net-pci");

    // 设置内存大小
    qemu_cmd.arg("-m").arg("8G");

    // 添加串口设备
    qemu_cmd.arg("-serial").arg("mon:stdio");

    // 去掉图形界面
    qemu_cmd.arg("-nographic");

    // 执行测试时支持退出qemu操作
    qemu_cmd
        .arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    // 启用gdb调试
    if matches.opt_present("gdb") {
        // 给qemu传递参数"-S -s"，使qemu停在第一条指令
        qemu_cmd.arg("-S").arg("-s");
    }

    // 执行qemu模拟器命令
    let mut child = qemu_cmd.spawn().unwrap();
    child.wait().unwrap();
}
