use crate::{gdt, print, println};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    /// define the reference for the IDT
    /// we use lazy static as rust compiler doesn't like normal
    /// ways of creating a static reference
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // This is unsafe as the used index MUST be valid, otherwise the
        // exception may not trigger or be a different exception than desired.
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

/// Initialise the interrupt descriptor table
pub fn init_idt() {
    print!("Initialising IDT...");
    IDT.load();
    println!("[ok]");
}

/// exception handler for breakpoints
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// exception handler for double faults
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}