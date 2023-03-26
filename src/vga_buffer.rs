use volatile::Volatile;
// https://en.wikipedia.org/wiki/Code_page_437
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

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

impl VGAWriter {
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

    fn new_line(&mut self) {
        // TODO
    }
}
