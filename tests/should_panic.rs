#![no_std]
#![no_main]

use core::panic::PanicInfo;
use wally_os::{exit_qemu, serial_print, serial_println, Failed, Okay, QemuExitCode, TEST_SEP};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("{}", Okay);
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("{}", Failed("test did not panic"));
    exit_qemu(QemuExitCode::Failure);
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

fn should_fail() {
    serial_print!("should_panic::should_fail{}", TEST_SEP);
    assert_eq!(0, 1);
}
