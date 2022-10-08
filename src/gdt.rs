//! Module for the global descriptor table.

use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// This defines the double fault stack
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// This struct defines our segment selectors
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    /// Define our global descriptor table
    static ref GDT: (GlobalDescriptorTable, Selectors)= {
        let mut gdt = GlobalDescriptorTable::new();
        // access the CS TSS registers
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors {code_selector, tss_selector})
    };
}

/// Initialise the global descriptor table
pub fn init_gdt() {
    crate::print!("Intialising GDT...");
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load(); // load the GDT
    unsafe {
        // reload the CPU's CS to the desired location
        CS::set_reg(GDT.1.code_selector);
        // tell the cpu to use the TSS in our GDT
        load_tss(GDT.1.tss_selector);
    }
    crate::println!("[ok]")
}

lazy_static! {
    /// Use for creating a Task State Segment with and interrupt
    /// stack table.
    static ref TSS: TaskStateSegment = {
        // Janky shit for now as we use workarounds to allocate a stack
        // before we do memory management. Possible to get a page fault
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}
