use core::arch::global_asm;

use x86_64::structures::paging::PageTableFlags;

use crate::mem::memory_set::{MapArea, MemorySet};
use crate::mem::VirtAddr;

use xmas_elf::{
    program::{SegmentData, Type},
    {header, ElfFile},
};

pub const USTACK_SIZE: usize = 4096 * 4;
pub const USTACK_TOP: usize = 0x300000;

global_asm!(include_str!("../link_app.S"));

extern "C" {
    pub static _app_count: u64;
}

pub fn get_app_count() -> usize {
    unsafe { _app_count as _ }
}

pub fn get_app_name(app_id: usize) -> &'static str {
    unsafe {
        let app_0_start_ptr = (&_app_count as *const u64).add(1);
        assert!(app_id < get_app_count());
        let app_name = app_0_start_ptr.add(app_id * 2).read() as *const u8;
        let mut len = 0;
        while app_name.add(len).read() != b'\0' {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(app_name, len);
        core::str::from_utf8_unchecked(slice)
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    unsafe {
        let app_0_start_ptr = (&_app_count as *const u64).add(1);
        assert!(app_id < get_app_count());
        let app_start = app_0_start_ptr.add(app_id * 2 + 1).read() as usize;
        let app_end = app_0_start_ptr.add(app_id * 2 + 2).read() as usize;
        let app_size = app_end - app_start;
        core::slice::from_raw_parts(app_start as *const u8, app_size)
    }
}

pub fn list_apps() {
    serial_println!("/**** APPS ****");
    let app_count = get_app_count();
    for i in 0..app_count {
        let data = get_app_data(i);
        serial_println!(
            "{}: [{:?}, {:?})",
            get_app_name(i),
            data.as_ptr_range().start,
            data.as_ptr_range().end
        );
    }
    serial_println!("**************/");
}

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    (0..get_app_count())
        .find(|&i| get_app_name(i) == name)
        .map(get_app_data)
}

pub fn load_app(elf_data: &[u8]) -> (usize, MemorySet) {
    // let elf_data = get_app_data(id);
    let elf = ElfFile::new(elf_data).expect("invalid ELF file");
    assert_eq!(
        elf.header.pt1.class(),
        header::Class::SixtyFour,
        "64-bit ELF required"
    );
    assert_eq!(
        elf.header.pt2.type_().as_type(),
        header::Type::Executable,
        "ELF is not an executable object"
    );
    assert_eq!(
        elf.header.pt2.machine().as_machine(),
        header::Machine::X86_64,
        "invalid ELF arch"
    );
    let mut ms = MemorySet::new();
    for ph in elf.program_iter() {
        if ph.get_type() != Ok(Type::Load) {
            continue;
        }
        let va = VirtAddr(ph.virtual_addr() as _);
        let offset = va.page_offset();
        let area_start = va.align_down();
        let area_end = VirtAddr((ph.virtual_addr() + ph.mem_size()) as _).align_up();
        let data = match ph.get_data(&elf).unwrap() {
            SegmentData::Undefined(data) => data,
            _ => panic!("failed to get ELF segment data"),
        };

        let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        if ph.flags().is_write() {
            flags |= PageTableFlags::WRITABLE;
        }
        let mut area = MapArea::new(area_start, area_end.0 - area_start.0, flags);
        area.write_data(offset, data);
        ms.insert(area);
    }
    ms.insert(MapArea::new(
        VirtAddr(USTACK_TOP - USTACK_SIZE),
        USTACK_SIZE,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    ));

    (elf.header.pt2.entry_point() as usize, ms)
}
