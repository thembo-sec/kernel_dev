#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod VGA_BUFFER;
pub mod gdt;
pub mod interrupts;
pub mod serial;

/// Initialises the kernel, to be called at the entry point of main
pub fn init_kernel() {
    println!("Initialising kernel...");
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // set sti
    println!("Kernel initiased successfully.");
}

/// This trait and its implmentation allows testable functions
/// to know their own names and print them when being tested, this
/// should save the hassle of printing 'testing...' for each one
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// This will handle test panics by printing failed and information
/// related to the panic. It will then exit qemu.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// Use port 0x4f, write exit code into qemu if exiting the kernel
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    //This function is unsafe because the I/O port could have side effects that violate memory safety.
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
