//! Paged VmObjects, holding a list of pages

use core::cell::RefCell;

use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::Mutex;

use crate::mem::vmar::VmMapping;
use hal::PhysFrame;

/// `VMObjectPaged`的弱引用类型
type WeakRef = Weak<VMObjectPaged>;

/// Types of `VMObjectPaged`
enum VMOType {
    /// The original mode.
    Origin,
    /// A snapshot of the parent node.
    Snapshot,
    /// Internal non-leaf node for snapshot.
    ///
    /// ```text
    ///    v---create_child
    ///    O       H <--- hidden node
    ///   /   =>  / \
    ///  S       O   S
    /// ```
    Hidden {
        /// The left child.
        left: WeakRef,
        /// The right child.
        right: WeakRef,
    },
}

/// The main VM object type, holding a list of pages.
pub struct VMObjectPaged {
    /// The lock that protects the `inner`
    /// This lock is shared between objects in the same clone tree to avoid deadlock
    lock: Arc<Mutex<()>>,
    inner: RefCell<VMObjectPagedInner>,
}

/// We always lock the lock before access to the Refcell, so it is actually sync
#[allow(unsafe_code)]
unsafe impl Sync for VMObjectPaged {}

/// `VMObjectPaged`的可变部分
struct VMObjectPagedInner {
    /// Owner identifier.
    owner: u64,
    type_: VMOType,
    /// Parent node.
    parent: Option<Arc<VMObjectPaged>>,
    /// The offset from parent.
    parent_offset: usize,
    /// The range limit from parent.
    parent_limit: usize,
    /// The size in bytes.
    size: usize,
    /// Physical frames of this VMO.
    frames: BTreeMap<usize, PageState>,
    /// All mappings to this VMO.
    mappings: Vec<Weak<VmMapping>>,
    // /// Cache Policy
    // cache_policy: CachePolicy,
    /// Is contiguous
    contiguous: bool,
    /// A weak reference to myself.
    self_ref: WeakRef,
    /// Sum of pin_count
    pin_count: usize,
}

/// Page state in VMO.
struct PageState {
    frame: PhysFrame,
    tag: PageStateTag,
    pin_count: u8,
}

/// The owner tag of pages in the node.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum PageStateTag {
    /// If the node is hidden, the page is shared by its 2 children.
    /// Otherwise, the page is owned by the node.
    Owned,
    /// The page is split to the left child and now owned by the right child.
    LeftSplit,
    /// The page is split to the right child and now owned by the left child.
    RightSplit,
}
