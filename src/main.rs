// don't include the standard library
#![no_std]
// removes the rust runtime
#![no_main]
// tell the kernel that we want to use our own custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(wally_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use wally_os::println;

// we have to implement our own panic handler since we no longer have access to the standard library
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[failed]\n");
    println!("Error: {}\n", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    wally_os::test_panic_handler(info)
}

// no_mangle ensures that the compiler really outputs a function with the name "_start".
// if we dont specify this, the compiler will mangle this name to something random
#[no_mangle]
// since we removed the rust runtime we need to define our own entry point.
// `_start` is the default entry point on most systems.
pub extern "C" fn _start() -> ! {
    // writer.write_string("Writing output with VGABuffer Writer");
    println!("Testing formatting: {} and {}", 42 + 18, 1.0 / 3.0);
    println!("Epic new line B)");

    // panic!("This is a panic message!!!");
    // NOTE: uncomment when removing the panic above
    loop {}
}
