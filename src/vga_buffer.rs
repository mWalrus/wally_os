use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// https://en.wikipedia.org/wiki/Code_page_437
/// The width of the text buffer.
const BUFFER_WIDTH: usize = 80;
/// The height of the text buffer.
const BUFFER_HEIGHT: usize = 25;

// define a globally accessible reference to the a writer instance.
lazy_static! {
    // we have to wrap it in a mutex in order to be able to mutably borrow it for write operations.
    pub static ref WRITER: Mutex<VGAWriter> = Mutex::new(VGAWriter::default());
}

/// Enum to represent all available colors for the VGA text buffer
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// An abstraction of the bytes representing a foreground and background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// assures that the "newtype" has the exact same data structure as the underlying type.
// https://doc.rust-lang.org/rust-by-example/generics/new_types.html
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        // Let's say that the bg is LightGreen, i.e. 0xa/10u8, and we shift it 4 bits to
        // the left, we get 0xa0/160u8.
        // Let's also say that the fg color is Red, i.e. 0x4/4u8.
        // We then bitwise OR the bg and fg to get the final color code 0xa4/164u8.
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

/// A character that can be drawn in the VGA text buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    character: u8,
    color_code: ColorCode,
}

/// The memory map of the text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A safe wrapper around the [VGA Text Buffer](https://en.wikipedia.org/wiki/VGA_text_mode).
pub struct VGAWriter {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Default for VGAWriter {
    fn default() -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            // assign the buffer member a mutable pointer to the VGA text buffer
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }
}

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl VGAWriter {
    /// write a single byte to the text buffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Write an ASCII string to the text buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // only write supported bytes
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // for all other, unsupported bytes, print "■".
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Creates a new line at the bottom of the text buffer
    fn new_line(&mut self) {
        // move all characters one row up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                // no need for bounds-checking since the row range start from 1
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// re-implementation of the print and println macros from the standard library
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// our internal print function that locks the mutex and "prints"
// the arguments to the text buffer.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
