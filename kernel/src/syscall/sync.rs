//! 同步互斥模块
use crate::{sync::*, *};

/// 为当前进程创建一个互斥锁
///
/// 根据blocking类型可以选择阻塞式或非阻塞式
///
/// 返回互斥锁的编号
pub fn sys_mutex_create(blocking: bool) -> isize {
    let p = &mut task::current().proc;
    let mutex: Box<dyn Mutex> = if blocking {
        Box::new(MutexBlocking::default())
    } else {
        Box::new(MutexSpin::default())
    };
    p.mutexes.push(mutex);
    (p.mutexes.len() - 1) as _
}

/// 为指定编号的互斥锁加锁
///
/// 成功返回0，失败返回-1
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let p = &mut task::current().proc;
    try_!(p.mutexes.get(mutex_id), -1).lock();
    0
}

/// 为指定编号的互斥锁解锁
///
/// 成功返回0，失败返回-1
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let p = &mut task::current().proc;
    try_!(p.mutexes.get(mutex_id), -1).unlock();
    0
}
