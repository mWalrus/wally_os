use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// 1. This is unsafe because the caller must guarantee that the complete physical memory is mapped
//    to virtual memory at the passed offset.
// 2. This function must only be called once to avoid aliasing `&mut` references which is
//    undefined behavior
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // 1. read the physical frame of the active level 4 table from the CR3 register.
    let (level_4_table_frame, _) = Cr3::read();

    // 2. get the start address of the physical frame
    let phys = level_4_table_frame.start_address();
    // 3. add the offset to that address to get the start address of the virtual memory
    let virt = physical_memory_offset + phys.as_u64();
    // 4. convert the virtual address to a raw pointer
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // 5. create a mutable reference of the raw pointer because we will
    //    want to modify this table later
    &mut *page_table_ptr
}
pub struct EmptyFrameAllocator;
// unsafe because the implementer must guarantee that the allocator
// yeilds only unused frames. Otherwise, undefined behavior might occur,
// for example when two virtual pages are mapped to the same physical frame.
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}
