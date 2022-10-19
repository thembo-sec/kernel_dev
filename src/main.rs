#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(custom_test_frameworks)]
#![test_runner(kernel_dev::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel_dev::memory::{active_level_4_table, translate_addr};
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;

mod VGA_BUFFER;
mod serial;

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Booting...");

    kernel_dev::init_kernel();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        // vga buffer page
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        // virt address mapped to physical adress 0
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    println!("Successfully booted.");
    kernel_dev::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    kernel_dev::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel_dev::test_panic_handler(info)
}
