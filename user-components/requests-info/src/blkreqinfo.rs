//! Blk device request.

use crate::CastBytes;

type BlockId = usize;
type Buf = usize;
type BufLen = usize;

pub enum BlkReqDescription {
    ReadBlock(BlockId, Buf, BufLen),
    WriteBlock(BlockId, Buf, BufLen),
}

impl CastBytes for BlkReqDescription {}
