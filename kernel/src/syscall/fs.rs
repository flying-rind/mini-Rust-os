//! 文件相关系统调用

use crate::*;
use fs::*;
use future::{executor, futures::WaitForKthread};
use requests_info::fsreqinfo::FsReqDescription;
use requests_info::CastBytes;
use task::CURRENT_THREAD;
use trap::{KthreadType, KTHREAD_MAP};

/// 当前进程打开文件
///
/// 异步系统调用，发送请求给fs内核线程并异步等待被唤醒
/// 所以不能直接使用寄存器传递返回fd，需要将fd指针传递
/// 给内核线程，服务完成后线程将fd写入用户态
///
/// 若fs线程不存在或发生其他错误，则返回(usize::MAX)
pub fn sys_open(path_ptr: usize, flags: usize, fd_ptr: usize) -> (usize, usize) {
    let fs_kthread = KTHREAD_MAP.get().get(&KthreadType::FS);
    match fs_kthread {
        Some(fs_kthread) => {
            let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
            let pid = current_thread.proc().unwrap().pid();
            // 构造fsreq
            let fsreq = FsReqDescription::Open(pid, path_ptr, flags as _, fd_ptr)
                .as_bytes()
                .to_vec();
            // 发送请求给fskthread
            let fs_kthread = fs_kthread.clone();
            let req_id = fs_kthread.add_request(fsreq);
            // 当前线程进入异步等待
            current_thread.set_state(trap::ThreadState::Waiting);
            // 生成等待协程
            executor::spawn(WaitForKthread::new(current_thread, fs_kthread, req_id));
            return (0, 0);
        }
        None => {
            println!("[Kernel] Error when sys_open, FS kthread not exist!");
            return (usize::MAX, 0);
        }
    }
}

/// 读取当前进程的fd对应的文件
///
/// 若是标准输入输出则直接读取，
/// 磁盘文件则发送请求给内核线程
///
/// 不存在此文件则返回usize::MAX
/// (0, 0)表示异步，返回值还未写入;
/// (read_size, 0)表示同步，可以直接使用返回值
pub fn sys_read(fd: usize, buf_ptr: usize, buf_len: usize, result_ptr: usize) -> (usize, usize) {
    let current_proc = current_proc();
    let file_table = current_proc.file_table();
    // [Debug]
    // println!("{}", file_table.len());
    let file_wrapper = file_table.get(fd);
    let file = if let Some(Some(f)) = file_wrapper {
        f.clone()
    // 不存在这个文件，直接返回不进行任何处理
    } else {
        return (usize::MAX, 0);
    };
    // 磁盘文件OSInode，则发送请求给fs内核线程
    if let Ok(_osinode) = file.clone().downcast_arc::<OSInode>() {
        let fs_kthread = KTHREAD_MAP.get().get(&KthreadType::FS);
        match fs_kthread {
            Some(fs_kthread) => {
                let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
                let pid = current_thread.proc().unwrap().pid();
                // 构造fsreq
                let fsreq = FsReqDescription::Read(pid, fd, buf_ptr, buf_len, result_ptr)
                    .as_bytes()
                    .to_vec();
                // 发送请求给fskthread
                let fs_kthread = fs_kthread.clone();
                let req_id = fs_kthread.add_request(fsreq);
                // 当前线程进入异步等待
                current_thread.set_state(trap::ThreadState::Waiting);
                // 生成等待协程
                executor::spawn(WaitForKthread::new(current_thread, fs_kthread, req_id));
                return (0, 0);
            }
            // fs-server线程不存在，返回usize::MAX
            None => {
                println!("[Kernel] Error when sys_open, FS kthread not exist!");
                return (usize::MAX, 0);
            }
        }
    // 管道，需要异步读取
    } else if let Ok(pipe) = file.clone().downcast_arc::<Pipe>() {
        pipe.async_read(pipe.clone(), buf_ptr, buf_len, result_ptr);
        return (0, 0);
    // 若是标准输入输出则直接读取不发送请求
    } else {
        let buf_ptr = buf_ptr as *mut u8;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, buf_len) };
        let read_size = file.read(buf);
        (read_size, 0)
    }
}

/// 写入当前进程的fd对应的文件
///
/// 若是标准输入输出则直接写入
/// 磁盘文件则发送请求给内核线程
///
/// 出错则返回usize::MAX
/// (0, 0)表示异步，返回值还未写入;
/// (read_size, 0)表示同步，可以直接使用返回值
pub fn sys_write(fd: usize, buf_ptr: usize, buf_len: usize, result_ptr: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    let file_table = current_proc.file_table();
    let file = if let Some(Some(file)) = file_table.get(fd) {
        file.clone()
    // 不存在这个文件，直接返回不进行任何处理
    } else {
        return (usize::MAX, 0);
    };
    // 磁盘文件OSInode，则发送请求给fs内核线程
    if let Ok(_osinode) = file.clone().downcast_arc::<OSInode>() {
        let fs_kthread = KTHREAD_MAP.get().get(&KthreadType::FS);
        match fs_kthread {
            Some(fs_kthread) => {
                let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
                let pid = current_thread.proc().unwrap().pid();
                // 构造fsreq
                let fsreq = FsReqDescription::Write(pid, fd, buf_ptr, buf_len, result_ptr)
                    .as_bytes()
                    .to_vec();
                // 发送请求给fskthread
                let fs_kthread = fs_kthread.clone();
                let req_id = fs_kthread.add_request(fsreq);
                // 当前线程进入异步等待
                current_thread.set_state(trap::ThreadState::Waiting);
                // 生成等待协程
                executor::spawn(WaitForKthread::new(current_thread, fs_kthread, req_id));
                return (0, 0);
            }
            // fs-server线程不存在，返回usize::MAX
            None => {
                println!("[Kernel] Error when sys_open, FS kthread not exist!");
                return (usize::MAX, 0);
            }
        }
    // 若是标准输入输出或管道则直接写入（同步）不发送请求
    } else {
        let buf_ptr = buf_ptr as *const u8;
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr, buf_len) };
        let read_size = file.write(buf);
        (read_size, 0)
    }
}

/// 当前进程关闭描述符为fd的文件
///
/// 成功返回0，否则返回usize::MAX
pub fn sys_close(fd: usize) -> (usize, usize) {
    let current_proc = current_proc();
    let file_table = current_proc.file_table();
    if let Some(file) = file_table.get_mut(fd) {
        if core::mem::replace(file, None).is_some() {
            return (0, 0);
        }
    }
    (usize::MAX, 0)
}

/// 创建管道，返回读端和写端的fd
pub fn sys_pipe() -> (usize, usize) {
    let current_proc = current_proc();
    let (read_end, write_end) = make_pipe();
    let (read_fd, write_fd) = (
        current_proc.add_file(read_end),
        current_proc.add_file(write_end),
    );
    (read_fd, write_fd)
}

/// 复制一份文件，一般与close一起使用
///
/// 若文件不存在则返回usize::MAX
pub fn sys_dup(fd: usize) -> (usize, usize) {
    let current_proc = current_proc();
    let file_table = current_proc.file_table();
    let file = if let Some(Some(f)) = file_table.get(fd) {
        f.clone()
    } else {
        return (usize::MAX, 0);
    };
    (current_proc.add_file(file), 0)
}

/// 列出可用用户app
pub fn sys_ls() -> (usize, usize) {
    let step = 7;
    let apps = ROOT_INODE.ls();
    for i in (0..apps.len()).step_by(step) {
        for j in i..i + step {
            if j < apps.len() {
                print!("{:<20}", apps[j]);
            } else {
                break;
            }
        }
        println!("");
    }
    (0, 0)
}
