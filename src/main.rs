#![no_std]
// removes the rust runtime
#![no_main]
use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

// no_mangle ensures that the compiler really outputs a function with the name "_start".
// if we dont specify this, the compiler will mangle this name to something random
#[no_mangle]
// since we removed the rust runtime we need to define our own entry point.
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}

// we have to implement our own panic handler since we no longer have access to the standard library
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
