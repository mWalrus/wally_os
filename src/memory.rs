use x86_64::{structures::paging::PageTable, VirtAddr};

// 1. This is unsafe because the caller must guarantee that the complete physical memory is mapped
//    to virtual memory at the passed offset.
// 2. This function must only be called once to avoid aliasing `&mut` references which is
//    undefined behavior
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
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
