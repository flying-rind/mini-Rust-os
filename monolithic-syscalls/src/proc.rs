//! 任务管理类系统调用
use crate::*;
use hal::SysError;
use hal::user::UserInOutPtr;
use hal::{check_n_clone_cstr, check_n_clone_cstr_array};
use log::info;
use monolithic_objects::get_thread;
use monolithic_objects::{
    PROCESSES,
    sync::{Event, wait_for_event},
};
use user_syscall::SysResult;

impl Syscall<'_> {
    /// Fork current process, return child's PID.
    pub fn sys_fork(&mut self) -> SysResult {
        let new_thread = self.thread.fork(self.context);
        let pid = new_thread.proc.lock().pid.0;
        info!("fork: {} -> {}", self.process().pid, pid);
        new_thread.start(self.thread_fn);
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
        Ok(proc.exec(&inode, cur_thread.clone(), args, envs, self.context)?)
    }

    /// Exit the current thread
    pub fn sys_exit(&mut self, exit_code: usize) -> SysResult {
        self.thread.exit(exit_code);
        Ok(0)
    }

    /// This system call terminates all threads in the calling process's
    /// thread group.
    ///
    /// [exit_group(2)](https://man7.org/linux/man-pages/man2/exit_group.2.html)
    ///
    /// FIXME: Exit ohter proc in group.
    pub fn sys_exit_group(&mut self, exit_code: usize) -> SysResult {
        let proc = self.process();
        info!("exit_group: {}, code: {}", proc.pid, exit_code);

        let tids = proc.threads.clone();
        drop(proc);
        for tid in tids {
            let thread = get_thread(tid).unwrap();
            thread.exit(exit_code);
        }
        Ok(0)
    }

    /// Wait 4 the process exit.
    /// Return the PID. Currently no option argument yet so just wait for the process to exit.(FIXME, read zcore)
    ///
    /// FIXME: Refactor to simplify this function.
    ///
    /// [wait(2)](https://man7.org/linux/man-pages/man2/waitpid.2.html)
    pub async fn sys_wait4(&mut self, pid: isize, mut wstatus: UserInOutPtr<i32>) -> SysResult {
        info!("wait4: pid: {}, code: {:?}", pid, wstatus);
        enum WaitForTarget {
            /// pid = -1, wait for any child.
            AnyChild,
            /// pid = 0, wait for any child that gpid = caller's gpid.
            AnyChildInGroup,
            /// pid > 0, wait for the child that pid = the given pid.
            Pid(usize),
        }
        let target = match pid {
            -1 => WaitForTarget::AnyChild,
            0 => WaitForTarget::AnyChildInGroup,
            p if p > 0 => WaitForTarget::Pid(p as _),
            _ => unimplemented!(),
        };
        loop {
            // FIXME, maybe can be simpler.
            let mut proc = self.process();
            let pgid = proc.pgid;
            // check if child exited yet.
            let mut res: Option<(monolithic_objects::Pid, usize)> = None;
            let exited = match target {
                WaitForTarget::AnyChild => {
                    for (pid, child) in &proc.children {
                        if let Some(c) = child.upgrade() {
                            let child_p = c.lock();
                            if child_p.exited() {
                                res = Some((*pid, child_p.exit_code));
                                break;
                            }
                        } else {
                            info!("Can not upgrade pid: {}", pid);
                        }
                    }
                    res
                }
                WaitForTarget::AnyChildInGroup => {
                    for (pid, child) in &proc.children {
                        if let Some(c) = child.upgrade() {
                            let child_p = c.lock();
                            if child_p.pgid != pgid {
                                continue;
                            }
                            if child_p.exited() {
                                res = Some((*pid, child_p.exit_code));
                                break;
                            }
                        } else {
                            info!("Can not upgrade pid: {}", pid);
                        }
                    }
                    res
                }
                WaitForTarget::Pid(wait_pid) => {
                    for (pid, child) in &proc.children {
                        if pid.0 != wait_pid {
                            continue;
                        }
                        if let Some(c) = child.upgrade() {
                            let child_p = c.lock();
                            if child_p.exited() {
                                res = Some((*pid, child_p.exit_code));
                                break;
                            }
                        } else {
                            info!("Can not upgrade pid: {}", pid);
                        }
                    }
                    res
                }
            };
            // Already exited, return.
            if let Some((pid, exit_code)) = exited {
                info!("Wait complete, pid: {}", pid);
                // Write exit_code to uspace
                wstatus.write(exit_code as i32)?;
                // Remove form process table
                let mut process_table = PROCESSES.write();
                process_table.remove(&pid);
                // remove from children table.
                proc.children.retain(|(p, _)| *p != pid);
                return Ok(pid.0);
            // Not exited yet. Check if argmument is valid.
            } else {
                let invalid = match target {
                    WaitForTarget::AnyChild => {
                        let children: Vec<_> = proc
                            .children
                            .iter()
                            .filter(|(_pid, child)| !child.upgrade().is_none())
                            .collect();
                        children.len() == 0
                    }
                    WaitForTarget::AnyChildInGroup => {
                        let children: Vec<_> = proc
                            .children
                            .iter()
                            .filter(|(_pid, child)| {
                                if let Some(child) = child.upgrade() {
                                    if child.lock().pgid == pgid {
                                        return true;
                                    }
                                }
                                return false;
                            })
                            .collect();
                        children.len() == 0
                    }
                    WaitForTarget::Pid(waitpid) => {
                        let children: Vec<_> = proc
                            .children
                            .iter()
                            .filter(|(_pid, child)| {
                                if let Some(child) = child.upgrade() {
                                    if child.lock().pid.0 == waitpid {
                                        return true;
                                    }
                                }
                                return false;
                            })
                            .collect();
                        children.len() == 0
                    }
                };
                if invalid {
                    info!("Wait: no valid child proc!");
                    return Err(SysError::ECHILD);
                }
                // Block and wait for proc to exit here.
                info!("wait4 not ready yet!");
                let bus = proc.eventbus.clone();
                drop(proc);
                wait_for_event(bus.clone(), Event::CHILD_PROCESS_QUIT).await;
                bus.lock().clear(Event::CHILD_PROCESS_QUIT);
            }
        }
    }

    /// The system call set_tid_address() sets the clear_child_tid value
    /// for the calling thread to tidptr.
    ///
    /// [set_tid_address(2)](https://man7.org/linux/man-pages/man2/set_tid_address.2.html)
    pub fn sys_set_tid_address(&mut self, tidptr: *mut u32) -> SysResult {
        info!("set_tid_address: {:?}", tidptr);
        self.thread.inner.lock().clear_child_tid = tidptr as usize;
        Ok(self.thread.tid)
    }
}
