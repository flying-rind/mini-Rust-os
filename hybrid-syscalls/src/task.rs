//! 任务管理相关的系统调用
use super::*;
use future::{ThreadYield, WaitForProc};
use hal::SysError;
use hal::user::UserInOutPtr;
use hal::{check_n_clone_cstr, check_n_clone_cstr_array};
use log::info;
use user_syscall::SysResult;

impl Syscall<'_> {
    /// Exit the current thread.
    pub fn sys_exit(&mut self, exit_code: usize) -> SysResult {
        // 退出当前进程
        let cur = current_thread();
        let tid = cur.tid();
        info!("Thread exit, tid: {}, code: {}", tid, exit_code);
        cur.exit(exit_code);
        Ok(0)
    }

    /// Replaces the current ** process ** with a new process image
    ///
    /// `argv` is an array of argument strings passed to the new program.
    /// `envp` is an array of strings, conventionally of the form `key=value`,
    /// which are passed as environment to the new program.
    ///
    /// NOTICE: `argv` & `envp` can not be NULL (different from Linux)
    ///
    /// NOTICE: for multi-thread programs
    /// A call to any exec function from a process with more than one thread
    /// shall result in all threads being terminated and the new executable image
    /// being loaded and executed.
    pub fn sys_exec(
        &mut self,
        pathp: *const u8,
        argvp: *const *const u8,
        envp: *const *const u8,
    ) -> SysResult {
        let cur_proc = current_proc();
        let path = check_n_clone_cstr(pathp)?;
        let args = check_n_clone_cstr_array(argvp)?;
        let envs = check_n_clone_cstr_array(envp)?;
        let inode = cur_proc.lookup_inode(&path)?;
        info!("exec: path: {:?}, argv: {:?}, env: {:?}", path, args, envs);
        Ok(cur_proc.exec(&inode, args, envs)?)
    }

    /// Wait 4 the process exit.
    /// Return the PID. Currently no option argument yet so just wait for the process to exit.(FIXME, read zcore)
    ///
    /// FIXME: Refactor to simplify this function.
    /// See [wait(2)](https://man7.org/linux/man-pages/man2/waitpid.2.html)
    pub async fn sys_wait4(&mut self, pid: isize, mut _wstatus: UserInOutPtr<i32>) -> SysResult {
        if pid == 0 || pid == -1 {
            unimplemented!("Not suportted yet!")
        }
        let current_thread = current_thread();
        let waited_process = match PROCESS_MAP.get().get(&(pid as usize)) {
            Some(process) => process.clone(),
            None => {
                info!("Wait complete, pid: {}", pid);
                return Ok(pid as _);
            }
        };
        current_thread.set_state(ThreadState::Waiting);
        let wait4proc = WaitForProc::new(current_thread, waited_process);
        wait4proc.await;
        Ok(pid as _)
    }
    /// 当前线程放弃CPU
    ///
    /// FIXME: Modify
    pub fn sys_yield(&mut self) -> SysResult {
        let current_thread = current_thread();
        current_thread.set_state(ThreadState::Waiting);
        executor::spawn(ThreadYield::new(current_thread));
        Ok(0)
    }

    // /// 创建线程，返回tid
    // pub fn sys_thread_create(entry: usize, arg1: usize, arg2: usize) -> (usize, usize) {
    //     let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    //     let current_proc = current_thread.proc().unwrap();
    //     let tid = current_proc.alloc_tid();
    //     // 每两个用户栈之间隔一段空间
    //     let sp_base = USER_STACK_BASE + tid * 2 * USER_STACK_SIZE;
    //     let new_thread = Thread::new(
    //         Arc::downgrade(&current_proc),
    //         tid,
    //         entry,
    //         sp_base + USER_STACK_SIZE,
    //     );
    //     new_thread.set_state(ThreadState::Runnable);
    //     current_proc.add_thread(new_thread);
    //     (tid, 0)
    // }

    // /// 退出当前线程
    // ///
    // /// 设置为Exited状态等待调度器清理
    // pub fn sys_thread_exit() -> (usize, usize) {
    //     // 获取当前线程
    //     let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    //     current_thread.set_state(ThreadState::Exited);
    //     (0, 0)
    // }

    // /// 主线程等待tid线程
    // ///
    // /// 若不是主线程调用，就报错并返回usize::MAX
    // pub fn sys_thread_join(tid: usize) -> (usize, usize) {
    //     let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    //     let cur_tid = current_thread.tid();
    //     if cur_tid != 0 {
    //         println!(
    //             "[Kernel] Thread join failed, can only be called by root thread, current tid: {}",
    //             cur_tid
    //         );
    //         return (usize::MAX, 0);
    //     }
    //     // 获取tid对应的线程
    //     let waited_thread = current_thread.proc().unwrap().get_thread(tid);
    //     let waited_thread = match waited_thread {
    //         Some(waited_thread) => waited_thread,
    //         None => {
    //             // println!(
    //             //     "[Kernel] Thread join info: waited thread already exited!, tid: {}",
    //             //     tid
    //             // );
    //             return (usize::MAX, 0);
    //         }
    //     };
    //     // 创建等待协程
    //     current_thread.set_state(ThreadState::Waiting);
    //     executor::spawn(WaitForThread::new(current_thread, waited_thread));
    //     return (0, 0);
    // }

    /// 获取当前进程PID
    pub fn sys_get_pid(&mut self) -> SysResult {
        let current_thread = current_thread();
        Ok(current_thread.proc().unwrap().pid())
    }

    /// 获取当前线程tid
    pub fn sys_get_tid(&mut self) -> SysResult {
        let current_thread = current_thread();
        Ok(current_thread.tid())
    }

    /// 复制当前进程
    pub fn sys_fork(&mut self) -> SysResult {
        let current_thread = current_thread();
        let current_proc = current_thread.proc().unwrap();
        let thread = current_proc.fork();
        thread.clone().start(self.thread_fn);
        Ok(thread.proc().unwrap().pid())
    }

    //new
    pub fn sys_vfork(&mut self) -> SysResult {
        self.sys_fork()
    }

    //new
    pub fn sys_exit_group(&mut self, exit_code: usize) -> SysResult {
        let proc = self.process().clone();
        info!("exit_group: {:?}, code: {:?}", proc.pid(), exit_code);
        let threads = get_process_threads(&proc);
        for thread in threads {
            thread.exit(exit_code);
        }
        info!("exit_group: {:?}, code: {:?}", proc.pid(), exit_code);
        Ok(())
    }

    //new

    pub fn sys_set_tid_address(&mut self, tidptr: *mut u32) -> SysResult {
        if tidptr.is_null() {
            panic!()
        }
        info!("set_tid_address: {:?}", tidptr);
        let mut thread = &mut self.thread;
        thread.clear_child_tid = tidptr as usize;
        // 返回一个默认值 0 ，可根据实际需求修改
        Ok(0)
    }
}
