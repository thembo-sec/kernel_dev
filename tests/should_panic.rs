//! This file contains the test handler and tests that should panic

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel_dev::{exit_qemu, serial_println, QemuExitCode, Testable};

#[panic_handler]

fn panic(_info: &PanicInfo) -> ! {

    serial_println!("[ok]");

    exit_qemu(QemuExitCode::Success);

    loop {}
}

#[no_mangle]

pub extern "C" fn _start() -> ! {

    should_fail.run(); //use the testable trait because its easier
    serial_println!("[test did not panic");

    exit_qemu(QemuExitCode::Failed); //exit if tests don't panic.
    loop {}
}

fn should_fail() {

    assert_eq!(0, 1);
}
