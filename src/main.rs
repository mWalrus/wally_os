#![no_std]
// removes the rust runtime
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

use vga_buffer::Writer;

static HELLO: &[u8] = b"Hello World!";

// no_mangle ensures that the compiler really outputs a function with the name "_start".
// if we dont specify this, the compiler will mangle this name to something random
#[no_mangle]
// since we removed the rust runtime we need to define our own entry point.
pub extern "C" fn _start() -> ! {
    let mut writer = Writer::default();
    writer.write_string("Writing output with VGABuffer Writer");
    loop {}
}

// we have to implement our own panic handler since we no longer have access to the standard library
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
