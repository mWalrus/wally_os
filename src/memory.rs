use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
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

// pub struct EmptyFrameAllocator;

// // unsafe because the implementer must guarantee that the allocator
// // yeilds only unused frames. Otherwise, undefined behavior might occur,
// // for example when two virtual pages are mapped to the same physical frame.
// unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
//         None
//     }
// }

pub struct BootInfoFrameAllocator {
    // reference to the memory map passed by the bootloader
    // which comes from the BIOS/UEFI firmware in the first place
    memory_map: &'static MemoryMap,
    // number of the next frame that the allocator should return
    next: usize,
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // this is really cool
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

impl BootInfoFrameAllocator {
    // unsafe since we don't know if the usable frames of the memory
    // map were already used somewhere else
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get the usable regions from the memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());

        // transform to an iterator of frame start addresses.
        // First, we flatten the `Iterator<Item = Iterator<Item = u64>>`
        // into just an `Iterator<Item = u64>`.
        // Then, we step by 4KiB through each of the ranges to get the
        // start addresses of all the physical frames in all usable regions.
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrames`s from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

// WARN: This is a test function which can cause undefined behaviour!
//       Not safe for use.
// pub fn create_example_mapping(
//     page: Page,
//     mapper: &mut OffsetPageTable,
//     frame_allocator: &mut impl FrameAllocator<Size4KiB>,
// ) {
//     use x86_64::structures::paging::PageTableFlags as Flags;
//     // we will use the address for the vga text buffer because
//     // it's easy to test whether the mapping was created correctly.
//     // We just need to write to this newly mapped page and see
//     // whether the text appears on the screen.
//     let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
//     let flags = Flags::PRESENT | Flags::WRITABLE;
//     // this is unsafe because the caller must ensure that the frame is
//     // not already in use. Mapping to the same frame twice could result
//     // in undefined behavior
//     // FIXME: this is not safe, we do it only for testing
//     let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
//     // flush the page from the translation lookaside buffer to ensure
//     // that the newest mapping is being used.
//     map_to_result.expect("map_to failed").flush();
// }
