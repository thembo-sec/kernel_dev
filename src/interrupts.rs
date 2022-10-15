use crate::hlt_loop;
use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

lazy_static! {
    /// Define the reference for the IDT.
    /// We use lazy static as rust compiler doesn't like normal
    /// ways of creating a static reference.
    static ref IDT: InterruptDescriptorTable = {

        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // This is unsafe as the used index MUST be valid, otherwise the
        // exception may not trigger or be a different exception than desired.
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
        .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

/// Initialise the interrupt descriptor table.
pub fn init_idt() {
    print!("Initialising IDT...");

    IDT.load();

    println!("[ok]");
}

// *****************************************
//
// CPU Exceptions
//
// *****************************************

/// exception handler for breakpoints
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

/// exception handler for double faults
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Exception handler for page faults.
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    //CR2 is set on page fault and contains address that caused it
    println!("Accessed Address:{:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

// *****************************************
//
// Hardware Interrupts
//
// *****************************************

/*                   ____________                          ____________
Real Time Clock --> |            |   Timer -------------> |            |
ACPI -------------> |            |   Keyboard-----------> |            |      _____
Available --------> | Secondary  |----------------------> | Primary    |     |     |
Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
Primary ATA ------> |            |   Floppy disk -------> |            |
Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
*/

/// Set pic interrupt vector numbers
/// start at 32, as 0-31 are used by the CPU exceptions
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Set static mutex for pics. Wrong offsets can cause undefined behaviour
/// so they are wrapped in an unsafe block.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// This contains the indexes of the interrupts that will hit our PIC.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    RTC = PIC_2_OFFSET,
}

// Not sure if this is necessary as the enum is already as a u8?
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// This function handles the timer intrrupts that occur. It incorporates the
/// notify end of interrupt function so that the PIC can continue to recieve
/// interrupts after the first one.
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    // TODO implement this in a way that only requires a single unsafe block
    // around an abstracted function.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// This function handles the keyboard interrupts.
extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    use pc_keyboard::{
        layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1,
    };
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        /// This is our keyboard static struct.
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                layouts::Us104Key,
                ScancodeSet1,
                HandleControl::Ignore
            ));
    }

    //TODO abstract keyboard read functions into seperate function.
    let mut keyboard = KEYBOARD.lock(); // Lock the mutex on the keyboard

    //read from the ps/2 controller (i/o port 0x60)
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn _RTC_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::RTC.as_u8());
    }
}
