// don't include the standard library
#![no_std]
// removes the rust runtime
#![no_main]
// tell the kernel that we want to use our own custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(wally_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use wally_os::println;

// we have to implement our own panic handler since we no longer have access to the standard library
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[failed]\n");
    println!("Error: {}\n", info);
    wally_os::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    wally_os::test_panic_handler(info)
}

// This macro ensures that the kernel's entry point has the correct signature.
// If we don't do this check and we take any arbitrary arguments, we would still compile just fine
// but, since it has the incorrect signature, would cause undefined behaviour at runtime.
entry_point!(kernel_main);

// we no longer need to mark this function as `extern "C"` or use the `#[no_mangle]` attribute on
// this function anymore since all of that is handled internally in the `entry_point` macro.

// BootInfo: since the kernel needs access to the memory map and offset to be able to handle memory
// we ask the bootloader to pass this information along to us since we won't be able to access this
// later.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // writer.write_string("Writing output with VGABuffer Writer");
    println!("Testing formatting: {} and {}", 42 + 18, 1.0 / 3.0);
    println!("Epic new line B)");

    wally_os::init();

    ///////////////////////////////////////////////
    // invoke a breakpoint exception
    // x86_64::instructions::interrupts::int3();
    ///////////////////////////////////////////////

    ///////////////////////////////////////////////
    // trigger a page fault by writing to and invalid memory address.
    // this alone does not cause a double fault, but a page fault.
    // The reason a double fault occurs is because we have not implemented
    // a page fault handler for the IDT.
    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // }
    ///////////////////////////////////////////////

    ///////////////////////////////////////////////
    // Trigger a page fault by trying to assign a value to a memory address
    // outside of our kernels memory region.
    // let ptr = 0xdeadbeaf as *mut u32;
    // unsafe {
    //     *ptr = 42;
    // }
    ///////////////////////////////////////////////

    ///////////////////////////////////////////////
    // an easy stack overflow trigger
    // fn stack_overflow() {
    //     stack_overflow();
    // }
    // stack_overflow();
    ///////////////////////////////////////////////

    ///////////////////////////////////////////////
    // the page fault error's stack frame gives us an instruction pointer.
    // If we swap out 0xdeadbeaf for the instruction pointer and try to read
    // from and write to this address, we should be able to read without
    // triggering a page fault!
    // Writing is still not permitted due to a protection violation.
    // This means that the address is present and valid, but we are not allowed
    // to write to it.
    // let ptr = 0x2075f7 as *mut u32;
    // unsafe {
    //     let _x = *ptr;
    //     println!("reading from address {ptr:x?} worked");
    // }
    // unsafe {
    //     *ptr = 42;
    //     println!("writing to address {ptr:x?} worked");
    // }
    ///////////////////////////////////////////////

    ///////////////////////////////////////////////
    // read the start address of the level 4 page table
    // use x86_64::registers::control::Cr3;
    // let (level_4_page_table, _) = Cr3::read();
    // println!(
    //     "Level 4 page table at: {table:?}",
    //     table = level_4_page_table.start_address()
    // );
    ///////////////////////////////////////////////

    ///////////////////////////////////////////////
    // use wally_os::memory::active_level_4_table;
    // use x86_64::VirtAddr;
    // // get the memory offset from the boot info
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // // init the table
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };
    // // print all used page table entries
    // for (i, entry) in l4_table.iter().enumerate() {
    //     use x86_64::structures::paging::PageTable;
    //     if !entry.is_unused() {
    //         println!("L4 Entry {i}: {entry:?}");
    //         // get physical address from the entry and convert it
    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };
    //         // print non-empty entries of the level 3 table
    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!("    L3 Entry {i}: {entry:?}");
    //             }
    //         }
    //     }
    // }
    ///////////////////////////////////////////////

    ///////////////////////////////////////////////
    // try out our virtual -> physical address translation function
    use wally_os::memory::translate_addr;
    use x86_64::VirtAddr;
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_002_1a10,
        // virtual address mapped to physical address 0
        // since our current implementation does not support huge pages,
        // this address translation panics.
        boot_info.physical_memory_offset,
    ];
    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{virt:?} -> {phys:?}");
    }
    ///////////////////////////////////////////////

    println!("didn't crash B)");

    wally_os::hlt_loop()
}
