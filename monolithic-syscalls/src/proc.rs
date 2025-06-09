//! 任务管理类系统调用
use crate::*;
use log::info;

impl Syscall<'_> {
    /// Fork current process, return child's PID.
    pub fn sys_fork(&mut self) -> SysResult {
        let new_thread = self.thread.fork(self.context);
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
        path: *const u8,
        argv: *const *const u8,
        envp: *const *const u8,
    ) -> SysResult {
        info!("exec: path: {:?}, argv: {:?}, envp: {:?}", path, argv, envp);
        let mut proc = self.process();
        let cur_tid = self.thread.tid;
        let path = check_n_clone_cstr(path)?;
        let args = check_n_clone_cstr_array(argv)?;
        let envs = check_n_clone_cstr_array(envp)?;

        if args.is_empty() {
            error!("exec: args is null");
            return Err(SysError::EINVAL);
        }

        info!("exec: path: {:?}, args: {:?}, envs: {:?}", path, args, envs);
        let inode = proc.lookup_inode(&path)?;
        Ok(proc.exec(&inode, cur_tid, args, envs)?)
    }
}
