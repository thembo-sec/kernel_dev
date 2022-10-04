#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Cast as a VGA buffer memory address as raw pointer
    let VGA_BUFFER = 0xb8000 as *mut u8;

    // iterate over bytes of hello string.
    for (i, &byte) in HELLO.iter().enumerate() {
        //unsafe block around memory write
        unsafe {
            *VGA_BUFFER.offset(i as isize * 2) = byte;
            *VGA_BUFFER.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
