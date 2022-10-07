#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
mod VGA_BUFFER;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Cast as a VGA buffer memory address as raw pointer
    println!("Booting...\nI should put a little spinny guy here to look cool.");

    #[cfg(test)]
    test_main();

    loop {}
}

// Iterate over all tests until
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    print!("Trivial assertion...");
    assert_eq!(1, 1);
    println!("[ok]")
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

    unsafe {
        let mut port = Port::new(0x4f);
        port.write(exit_code as u32);
    }
}
