//! 同步互斥相关系统调用
use trap::CURRENT_THREAD;

use crate::{sync::*, *};
use alloc::sync::Arc;

/// 为当前进程创建一个互斥锁，返回互斥锁的编号
pub fn sys_mutex_create() -> (usize, usize) {
    let current_proc = CURRENT_THREAD
        .get()
        .as_ref()
        .unwrap()
        .clone()
        .proc()
        .unwrap();
    let mutex = Arc::new(MutexBlocking::default());
    (current_proc.add_mutex(mutex), 0)
}

/// 为指定编号的互斥锁加锁
///
/// 成功返回0，失败返回usize::MAX
pub fn sys_mutex_lock(mutex_id: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    if let Some(mutex) = current_proc.mutexes().get_mut(mutex_id) {
        mutex.lock(mutex.clone(), current_thread);
    } else {
        return (usize::MAX, 0);
    }
    (0, 0)
}

/// 为指定编号的互斥锁解锁
///
/// 成功返回0，失败返回usize::MAX
pub fn sys_mutex_unlock(mutex_id: usize) -> (usize, usize) {
    let current_proc = CURRENT_THREAD
        .get()
        .as_ref()
        .unwrap()
        .clone()
        .proc()
        .unwrap();
    if let Some(mutex) = current_proc.mutexes().get(mutex_id) {
        mutex.unlock();
    } else {
        return (usize::MAX, 0);
    }
    (0, 0)
}

/// 创建信号量，返回id
pub fn sys_sem_create(n: usize) -> (usize, usize) {
    let current_proc = CURRENT_THREAD
        .get()
        .as_ref()
        .unwrap()
        .clone()
        .proc()
        .unwrap();
    let sem = Sem::new(n);
    (current_proc.add_sem(sem), 0)
}

/// 信号量增加一个资源
pub fn sys_sem_up(sem_id: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    if let Some(sem) = current_proc.sems().get_mut(sem_id) {
        sem.up();
    } else {
        return (usize::MAX, 0);
    }
    (0, 0)
}

/// 信号量消耗一个资源
pub fn sys_sem_down(sem_id: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    if let Some(sem) = current_proc.sems().get_mut(sem_id) {
        sem.down(sem.clone());
    } else {
        return (usize::MAX, 0);
    }
    (0, 0)
}

/// 创建条件变量，返回ID
pub fn sys_condvar_create() -> (usize, usize) {
    let current_proc = CURRENT_THREAD
        .get()
        .as_ref()
        .unwrap()
        .clone()
        .proc()
        .unwrap();
    let condvar = Condvar::new();
    (current_proc.add_condvar(condvar), 0)
}

/// 阻塞一个条件变量
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    if let Some(condvar) = current_proc.condvars().get_mut(condvar_id) {
        if let Some(mutex) = current_proc.mutexes().get_mut(mutex_id) {
            condvar.wait(mutex.clone(), condvar.clone());
            return (0, 0);
        } else {
            return (usize::MAX, 0);
        }
    } else {
        return (usize::MAX, 0);
    }
}

/// 唤醒条件变量上阻塞的一个线程
pub fn sys_condvar_signal(condvar_id: usize) -> (usize, usize) {
    let current_thread = CURRENT_THREAD.get().as_ref().unwrap().clone();
    let current_proc = current_thread.proc().unwrap();
    if let Some(condvar) = current_proc.condvars().get_mut(condvar_id) {
        condvar.signal();
        return (0, 0);
    } else {
        return (usize::MAX, 0);
    }
}
