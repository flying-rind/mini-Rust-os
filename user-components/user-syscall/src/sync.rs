//! 同步互斥类用户系统调用封装
use super::*;

/// 创建互斥锁，返回编号
pub fn mutex_create() -> usize {
    let (mutex_id, _) = sys_mutex_create();
    mutex_id
}

/// 互斥锁上锁，成功返回True，否则返回False
pub fn mutex_lock(mutex_id: usize) -> bool {
    let (ret, _) = sys_mutex_lock(mutex_id);
    ret == 0
}

/// 互斥锁解锁，成功返回true，否则false
pub fn mutex_unlock(mutex_id: usize) -> bool {
    let (ret, _) = sys_mutex_unlock(mutex_id);
    ret == 0
}

/// 创建信号量，返回id
pub fn sem_create(n: usize) -> usize {
    let (sem_id, _) = sys_sem_create(n);
    sem_id
}

/// 增加信号量资源，返回是否成功
pub fn sem_up(sem_id: usize) -> bool {
    let (ret, _) = sys_sem_up(sem_id);
    ret == 0
}

/// 减少信号量资源，返回是否成功
pub fn sem_down(sem_id: usize) -> bool {
    let (ret, _) = sys_sem_down(sem_id);
    ret == 0
}

/// 创建条件变量，返回编号
pub fn condvar_create() -> usize {
    let (condvar_id, _) = sys_condvar_create();
    condvar_id
}

/// 当前线程阻塞条件变量
pub fn condvar_wait(condvar_id: usize, mutex_id: usize) -> bool {
    let (ret, _) = sys_condvar_wait(condvar_id, mutex_id);
    ret == 0
}

/// 唤醒条件变量阻塞的线程
pub fn condvar_signal(condvar_id: usize) -> bool {
    let (ret, _) = sys_condvar_signal(condvar_id);
    ret == 0
}
