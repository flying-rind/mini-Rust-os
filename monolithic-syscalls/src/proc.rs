//! 任务管理类系统调用
use crate::*;
use log::info;
use monolithic_objects::{THREADS, task::ThreadState};

impl Syscall<'_> {
    /// Fork current process, return child's PID.
    pub fn sys_fork(&mut self) -> SysResult {
        let new_thread = self.thread.fork(self.thread());
        let pid = new_thread.proc.lock().pid.0;
        let future = (self.thread_fn)(self.thread.clone());
        executor::spawn(future);
        Ok(pid)
    }

    /// Same as fork for now.
    pub fn sys_vfork(&mut self) -> SysResult {
        self.sys_fork()
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
        info!(
            "exec: pathp: {:?}, argvp: {:?}, envp: {:?}",
            pathp, argvp, envp
        );
        let cur_thread = self.thread;
        let mut proc = cur_thread.proc.lock();
        let path = check_n_clone_cstr(pathp)?;
        let args = check_n_clone_cstr_array(argvp)?;
        let envs = check_n_clone_cstr_array(envp)?;

        if args.is_empty() {
            error!("exec: args is null");
            return Err(SysError::EINVAL);
        }

        info!("exec: path: {:?}, args: {:?}, envs: {:?}", path, args, envs);
        let inode = proc.lookup_inode(&path)?;
        Ok(proc.exec(&inode, cur_thread.clone(), args, envs)?)
    }

    /// Exit the current thread
    pub fn sys_exit(&mut self, exit_code: usize) -> SysResult {
        let tid = self.thread.tid;
        info!("Thread exit, tid: {}, code: {}", tid, exit_code);
        // Delete tid ref in process.
        let mut proc = self.process();
        proc.threads.retain(|&id| id != tid);
        // Delete arc ref in THREAD table;
        let mut threads_table = THREADS.write();
        threads_table.remove(&tid);
        // for last thread, eixt the process
        if proc.threads.len() == 0 {
            proc.exit(exit_code);
        };
        drop(proc);
        self.thread.inner.lock().state = ThreadState::Exited;
        Ok(0)
    }
}
