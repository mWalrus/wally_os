#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
// tell the kernel that we want to use our own custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
pub mod serial;
#[macro_use]
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;

use core::fmt;
use core::panic::PanicInfo;

pub static TEST_SEP: &str = "..........";
pub struct Okay;
pub struct Failed(pub &'static str);

impl fmt::Display for Okay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[32m[ok]\x1b[0m")
    }
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1B[31m[{}]\x1b[0m", self.0)
    }
}

impl Default for Failed {
    fn default() -> Self {
        Self("failed")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}{}", core::any::type_name::<T>(), TEST_SEP);
        self();
        serial_println!("{}", Okay);
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("{}\n", Failed::default());
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
    loop {}
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
