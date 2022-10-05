#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
    //store VGA colours as an enum with representation u8
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

// struct contains full colour byte including fore and back
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColourCode(u8);

//impl for the colourcode struct, we can move the background bitwise by four as colours only
//need to be u4
impl ColourCode {
    fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    colour_code: ColourCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// writer type for writing ASCII to the screen
pub struct Writer {
    column_position: usize,      //keeps track of current position in last row
    colour_code: ColourCode,     //colours
    buffer: &'static mut Buffer, // VGA buffer refference, explicit lifetime for whole program
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let colour_code = self.colour_code;
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    colour_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // TODO
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                //printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // if character not supported print block byte.
                _ => self.write_byte(0xfe),
            }
        }
    }
}

pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        colour_code: ColourCode::new(Colour::Green, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_string("Hello World");
}
