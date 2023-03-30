use crate::gdt;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    // this interrupt descriptor table tells the CPU where to find all the different handlers
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // add all the different handlers here
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
// ensure that the enum is "C-like", meaning that all members are just numbers (u8 in this case) under the hood.
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // 32
    Keyboard,             // 32 + 1 = 33
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // breakpoint interrupts are what most debuggers use in order to stop execution of code at a specified location.
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // https://en.wikipedia.org/wiki/Intel_8253
    // the hardware timer prints a dot asynchronously every tick.
    print!(".");
    unsafe {
        // tell the PICs that we are at the end of the timer interrupt.
        // this is done in order for the cpu to know when to continue to the next event.
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // here we use the pc_keyboard crate to help us interpret scancodes coming from the keyboard controller.
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    // initialize a static mutable keyboard object
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key, // US keyboard layout
                HandleControl::Ignore // ignore mapping things like `ctrl+[a-z]` to their unicode representations
            ));
    }

    // lock the keyboard mutex
    let mut keyboard = KEYBOARD.lock();
    // 0x60 is the port number of the PS/2 controller.
    // PS/2 is an old standard that was used before USB peripherals were a thing,
    // and while new hardware won't have an actual PS/2 controller, most still emulate one.
    let mut port = Port::new(0x60);

    // read the scancode of the pressed key from the PS/2 controller.
    // This is unsafe because the I/O port could have side effects that violate memory safety.
    let scancode: u8 = unsafe { port.read() };
    // add the scancode byte to the keyboard object which then generates a `KeyEvent` if successful
    if let Ok(Some(key_evt)) = keyboard.add_byte(scancode) {
        // process the above key event
        if let Some(key) = keyboard.process_keyevent(key_evt) {
            match key {
                DecodedKey::Unicode(character) => print!("{character}"),
                DecodedKey::RawKey(key) => print!("{key:?}"),
            }
        }
    }

    // let the CPU know we're done
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
}

use crate::hlt_loop;
use x86_64::structures::idt::PageFaultErrorCode;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {error_code:?}");
    println!("{stack_frame:#?}");
    hlt_loop();
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn breakpoint_exception() {
        x86_64::instructions::interrupts::int3();
    }
}
