#![no_std]
#![no_main]
#![allow(non_snake_case)]
use core::panic::PanicInfo;
mod VGA_BUFFER;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World! Goodbye";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Cast as a VGA buffer memory address as raw pointer
    println!("Hello Arden, I just implemented my own macro, \nwith a lot of tutorial handholding.");
    loop {}
}
