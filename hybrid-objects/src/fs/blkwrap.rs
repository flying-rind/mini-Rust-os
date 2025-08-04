//! Wrap the blk driver to use blk kthread.
use crate::CURRENT_THREAD;
use crate::KTHREAD_MAP;
use crate::KthreadType;
use crate::Scheduler;
use crate::ThreadState;
use crate::current_kthread;
use crate::current_thread;
use crate::future::KthreadWait4Kthread;
use crate::future::WaitForKthread;
use rcore_fs::dev::BlockDevice;
use requests_info::CastBytes;
use requests_info::blkreqinfo::BlkReqDescription;

/// Wrap the blk driver, send request to blk kthread.
pub struct BlockDriverWrapper();

impl BlockDriverWrapper {
    /// Send read request to blk kthread.
    pub fn send_read_req(&self, block_id: usize, buf: &mut [u8]) {
        let blk_kthread = KTHREAD_MAP.get().get(&KthreadType::BLK);
        match blk_kthread {
            Some(kthread) => {
                let cur_thread = CURRENT_THREAD.get().clone();
                let cur_kthread = current_kthread();
                // Construct req.
                let blkreq =
                    BlkReqDescription::ReadBlock(block_id, buf.as_mut_ptr() as _, buf.len())
                        .as_bytes()
                        .to_vec();
                let req_id = kthread.clone().add_request(blkreq);
                if let Some(cur_thread) = cur_thread {
                    cur_thread.set_state(ThreadState::Waiting);
                    let wait4kthread = WaitForKthread::new(cur_thread, kthread.clone(), req_id);
                    executor::spawn(wait4kthread);
                }
                cur_kthread.set_state(crate::KthreadState::Idle);
                let kwait4k = KthreadWait4Kthread::new(cur_kthread, kthread.clone(), req_id);
                executor::spawn(kwait4k);
            }
            None => {
                error!("[Kernel] Error when read blk, Blk kthread not exist!");
            }
        }
    }

    /// Send write request to blk kthread.
    pub fn send_write_req(&self, block_id: usize, buf: &[u8]) {
        let blk_kthread = KTHREAD_MAP.get().get(&KthreadType::BLK);
        match blk_kthread {
            Some(kthread) => {
                let cur_thread = current_thread();
                // Construct req.
                let blkreq = BlkReqDescription::WriteBlock(block_id, buf.as_ptr() as _, buf.len())
                    .as_bytes()
                    .to_vec();
                let req_id = kthread.clone().add_request(blkreq);
                cur_thread.set_state(ThreadState::Waiting);
                let wait4kthread = WaitForKthread::new(cur_thread, kthread.clone(), req_id);
                let fs_kthread = KTHREAD_MAP.get().get(&KthreadType::FS).cloned().unwrap();
                let kwait4k = KthreadWait4Kthread::new(fs_kthread, kthread.clone(), req_id);
                executor::spawn(wait4kthread);
                executor::spawn(kwait4k);
            }
            None => {
                error!("[Kernel] Error when read blk, Blk kthread not exist!");
            }
        }
    }
}

impl BlockDevice for BlockDriverWrapper {
    const BLOCK_SIZE_LOG2: u8 = 9; // 512
    fn read_at(&self, block_id: usize, buf: &mut [u8]) -> rcore_fs::dev::Result<()> {
        self.send_read_req(block_id, buf);
        Scheduler::yield_current_kthread();
        Ok(())
    }

    fn write_at(&self, block_id: usize, buf: &[u8]) -> rcore_fs::dev::Result<()> {
        self.send_write_req(block_id, buf);
        Scheduler::yield_current_kthread();
        Ok(())
    }

    fn sync(&self) -> rcore_fs::dev::Result<()> {
        Ok(())
    }
}
