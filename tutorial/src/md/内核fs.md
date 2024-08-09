# 内核接入easy-fs

```Rust
/// OS里操作的索引节点类型，封装了easy-fs中的Inode
pub struct OSInode {
    /// 是否可读
    readable: bool,
    /// 是否可写
    writable: bool,
    /// 偏移
    offset: Cell<usize>,
    /// 封装easy-fs中的Inode
    inode: Cell<Arc<Inode>>,
}
```
内核中封装了easy-fs中的Inode，使用OSInode结构来对单个文件进行操作。

```Rust
/// 全局变量：根节点的索引节点
pub static ROOT_INODE: Cell<Arc<Inode>> = unsafe { transmute([1u8; size_of::<Arc<Inode>>()]) };

/// 文件系统初始化,创建root inode
pub fn init() {
    let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
    unsafe {
        (ROOT_INODE.get() as *mut Arc<Inode>).write(Arc::new(EasyFileSystem::root_inode(&efs)));
    }
    println!("/****APPS****/");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}
```
为根节点创建Inode后，我们使用全局变量ROOT_INODE来创建或打开文件

```Rust
/// 根据OpenFlags打开根节点下的文件
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Rc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.find(name) {
            inode.clear();
            Some(Rc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            ROOT_INODE
                .create(name)
                .map(|inode| Rc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        ROOT_INODE.find(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Rc::new(OSInode::new(readable, writable, inode))
        })
    }
}
```
由于easy-fs为扁平化结构，所有文件都在根目录下，所以现在我们可以在文件系统中打开所有用户程序了。如在内核中创建root进程：

```Rust
// 创建根进程
fn main() {
    ...
    info!("[Kernel] Load root");
    let root_file = open_file("root", OpenFlags::RDONLY).unwrap();
    let elf_data = root_file.read_all();
    let root_process = Process::new(String::from("root"), &elf_data, None);
}
```
