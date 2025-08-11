use clap::{App, Arg};
use rcore_fs::dev::{DevError, Device};
use rcore_fs::vfs::FileSystem;
use rcore_fs_sfs::SimpleFileSystem;
use std::env;
use std::fs::{File, OpenOptions, read_dir};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;

struct BlockFile(Mutex<File>);

impl Device for BlockFile {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> rcore_fs::dev::Result<usize> {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset as _)).unwrap();
        file.read(buf).unwrap();
        Ok(buf.len())
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> rcore_fs::dev::Result<usize> {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset as _)).unwrap();
        file.write(buf).unwrap();
        Ok(buf.len())
    }

    fn sync(&self) -> Result<(), DevError> {
        let file = self.0.lock().unwrap();
        file.sync_all().unwrap();
        Ok(())
    }
}

fn main() {
    let rs_src_path = "../user-rs/src/bin";
    let rs_target_path = "../user-rs/target/x86_64/release/";
    let c_src_path = "../user-c/src";
    let c_target_path = "../user-c/bin/";

    println!(
        "rs_src_path = {}\nrs_target_path = {}",
        rs_src_path, rs_target_path
    );
    pub const USER_IMAGE_SIZE: usize = 16 * 1024 * 1024;
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}{}", rs_target_path, "fs.img"))
            .unwrap();
        f.set_len(USER_IMAGE_SIZE as _).unwrap();
        f
    })));
    // Debug
    println!("Finished creating block file");
    // 16MiB, at most 4095 files
    let sfs = SimpleFileSystem::create(block_file, USER_IMAGE_SIZE).expect("Failed to create sfs");
    let root_inode = sfs.root_inode();
    let rs_apps: Vec<_> = read_dir(rs_src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    println!("rs-apps: {:?}", rs_apps);

    let mut c_apps: Vec<_> = read_dir(c_target_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            // name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    println!("c-apps: {:?}", c_apps);

    // Busybox.
    // c_apps.push("busybox".to_string());

    // 将app挂载到文件系统

    for app in rs_apps {
        // load app data from host file system
        let mut host_file = File::open(format!("{}{}", rs_target_path, app)).unwrap();
        // debug
        println!("Loading app: {}", app);
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        // create a file in easy-fs
        let inode = root_inode
            .create(app.as_str(), rcore_fs::vfs::FileType::File, 0o777)
            .unwrap();
        // write data to easy-fs
        inode.write_at(0, all_data.as_slice()).unwrap();
    }

    // debug
    println!("Finished loading rust apps");

    for app in c_apps {
        // load app data from host file system
        let mut host_file = File::open(format!("{}{}", c_target_path, app)).unwrap();
        println!("Loading app: {}", app);
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        // create a file in easy-fs
        let inode = root_inode
            .create(app.as_str(), rcore_fs::vfs::FileType::File, 0o777)
            .unwrap();
        // write data to easy-fs
        inode.write_at(0, all_data.as_slice()).unwrap();
    }

    // debug
    println!("Finished loading all apps!");

    // list apps
    for app in root_inode.list().unwrap() {
        println!("{}", app);
    }
}
