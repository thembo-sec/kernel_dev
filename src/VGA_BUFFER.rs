use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

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
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// writer type for writing ASCII to the screen
pub struct Writer {
    column_position: usize, //keeps track of current position in last row
    colour_code: ColourCode, //colours
    buffer: &'static mut Buffer, // VGA buffer refference, explicit lifetime for whole program
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), //call newline function
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    //check width
                    self.new_line();
                }

                //else move to next position
                let row = BUFFER_HEIGHT - 1;

                let col = self.column_position;

                let colour_code = self.colour_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    colour_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // iterate over each character in each row, and shift in one line up.
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();

                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);

        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        // write an empty character into each chunk of the row
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// This function will overwrite the previous column with a blank space
    fn backspace(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };
        // double check if the column is at 0, otherwise this will
        // cause an integer overflow.
        if self.column_position != 0 {
            self.column_position -= 1;
        }
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        self.buffer.chars[row][col].write(blank);
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                0x08 => self.backspace(), //if backspace
                // if character not supported print block byte.
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);

        Ok(())
    }
}

lazy_static! {
    // use a spinning mutex for this to enable simple lock.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        colour_code: ColourCode::new(Colour::Green, Colour::Black),
        //unsafe reference to the buffer
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

/*
This macro expands to a call of the _print function.
The $crate variable ensures that the macro also works from
outside the crate by expanding when it’s used in other crates.
*/

/// Print a string, with no newline at the end of it
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::VGA_BUFFER::_print(format_args!($($arg)*)));
}

/// Print a string with newline at the end of it
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*)=> ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// This function locks a static WRITER and calls write_fmt.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    // Execute the write in a way that does not allow interrupts
    // uring the write
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// test single printline
#[test_case]

fn test_println_simple() {
    println!("Test_println_simple output")
}

//test vga buffer over many lines
#[test_case]

fn test_println_lots() {
    for _ in 0..200 {
        println!("Test_println_simple output")
    }
}

/*
The function defines a test string, prints it using println,
and then iterates over the screen characters of the static WRITER,
which represents the VGA text buffer. Since println prints to
the last screen line and then immediately appends a newline,
the string should appear on line BUFFER_HEIGHT - 2.
 */

#[test_case]

fn test_println_output() {
    use x86_64::instructions::interrupts;
    let s = "This is a test string that prints on a single line";

    println!("{}", s);

    //count the number of iterations in the variable i, then
    //use for loading the screen character corresponding to c.

    interrupts::without_interrupts(|| {
        for (i, c) in s.chars().enumerate() {
            let screen_char =
                WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();

            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
