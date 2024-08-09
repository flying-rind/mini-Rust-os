use crate::{fs::ROOT_INODE, println};

/// 测试内核线程
pub fn kthread_test_entry(_ktid: usize) {
    let inode = ROOT_INODE.find("shell");
    assert!(inode.is_some());
    println!("[Test Kthread] Test kthread can use ROOT_INODE");
}
