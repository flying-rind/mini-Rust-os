//! 任务管理模块

mod manager;
mod process;
mod sleep;
mod thread;

pub use self::{manager::*, process::*, sleep::*, thread::*};
use crate::{fs::*, *};

// Dirty hack. rustc is unhappy about zero value in VecDeque.
static THREAD_MANAGER: Cell<ThreadManager> =
    unsafe { transmute([1u8; size_of::<ThreadManager>()]) };
/// 根进程
static ROOT_PROC: Cell<usize> = zero();

pub fn init() -> ! {
    assert_eq!(size_of::<Thread>(), THREAD_SIZE);
    unsafe { (THREAD_MANAGER.get() as *mut ThreadManager).write(ThreadManager::default()) }
    let root = Box::leak(Box::new(Process {
        pid: new_id(),
        files: vec![
            Some(Rc::new(Stdin)),
            Some(Rc::new(Stdout)),
            Some(Rc::new(Stdout)),
        ],
        ..Process::default()
    }));
    // [Debug]
    // print!("Debug, root.vm = {:#?}", root.vm);
    *ROOT_PROC.get() = root as *mut _ as _;
    Thread::new(
        root,
        |_| {
            let cur = current();
            // 回收已退出子进程
            loop {
                my_x86_64::disable_interrupts();
                cur.proc.waitpid(-1);
                my_x86_64::enable_interrupts_and_hlt();
            }
        },
        0,
    );
    let shell = root.fork();
    shell.exec("user_shell", Vec::new());
    unsafe {
        context_switch(&mut Context::default(), &THREAD_MANAGER.get().dequeue().ctx);
    }
    unreachable!("Error, cannot reach here");
}

/// 获取根进程
pub fn root_proc() -> ProcPtr {
    unsafe { transmute(*ROOT_PROC) }
}

/// 从当前栈顶获取当前线程
pub fn current() -> &'static mut Thread {
    unsafe { &mut *((my_x86_64::read_rsp() & !(THREAD_SIZE - 1)) as *mut _) }
}

/// 当前线程放弃CPU
pub fn current_yield() {
    THREAD_MANAGER.get().resched();
}

/// 将当前线程设置为阻塞状态，在将其唤醒前，确保其不参与调度
///
/// 然后调度下一个就绪线程
pub fn sched_block() {
    current().state = ThreadState::Blocking;
    THREAD_MANAGER.get().resched();
}

/// 唤醒线程，将其加入就绪线程队列
pub fn sched_unblock(t: &mut Thread) {
    t.state = ThreadState::Runnable;
    THREAD_MANAGER.get().enqueue(t);
}
