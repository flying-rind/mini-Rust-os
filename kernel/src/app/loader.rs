use core::arch::global_asm;

use x86_64::structures::paging::PageTableFlags;

use crate::mem::memory_set::{MapArea, MemorySet};
use crate::mem::VirtAddr;

use xmas_elf::{
    program::{SegmentData, Type},
    {header, ElfFile},
};

global_asm!(include_str!("../link_app.S"));

pub const USTACK_SIZE: usize = 4096 * 4;
pub const USTACK_TOP: usize = 0x4000_0000_0000;

extern "C" {
    pub static _app_count: usize;
}

pub fn get_app_count() -> usize {
    unsafe { _app_count }
}

pub fn get_app_name(app_id: usize) -> &'static str {
    unsafe {
        let app_0_start_ptr = (&_app_count as *const usize).add(1);
        assert!(app_id < get_app_count());
        let name = *app_0_start_ptr.add(app_id * 2) as *const u8;
        let mut len = 0;
        while *name.add(len) != b'\0' {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(name, len);
        core::str::from_utf8_unchecked(slice)
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    assert!(app_id < get_app_count());
    unsafe {
        let app_0_start_ptr = (&_app_count as *const usize).add(1);
        let app_start = *app_0_start_ptr.add(app_id * 2 + 1);
        let app_end = *app_0_start_ptr.add(app_id * 2 + 2);
        let app_size = app_end - app_start;
        core::slice::from_raw_parts(app_start as _, app_size)
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

pub fn load_app(app_id: usize) -> (usize, MemorySet) {
    assert!(app_id < get_app_count());

    let elf_data = get_app_data(app_id);
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
        // crate::arch::flush_icache_all();
    }
    ms.insert(MapArea::new(
        VirtAddr(USTACK_TOP - USTACK_SIZE),
        USTACK_SIZE,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    ));

    (elf.header.pt2.entry_point() as usize, ms)
}
