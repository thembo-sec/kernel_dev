#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(custom_test_frameworks)]
#![test_runner(kernel_dev::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel_dev::allocator;
use kernel_dev::memory::{self, BootInfoFrameAllocator};
use x86_64::{structures::paging::Page, VirtAddr};

mod VGA_BUFFER;
mod serial;

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Booting...");

    kernel_dev::init_kernel();

    x86_64::instructions::interrupts::enable(); // set sti
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // new
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let hv = Box::new(41);
    println!("Heap value at: {:p}", hv);

    let mut vec = Vec::new();
    for i in 0..100{
        vec.push(i);
    }
    println!("Vec at {:p}", vec.as_slice());


    let ref_c = Rc::new(vec![1,2,3]);
    let cloned_ref = ref_c.clone();
    println!("Current reference counted is: {}", Rc::strong_count(&cloned_ref));
    core::mem::drop(ref_c);
    println!("Ref dropped, new ref count is: {}", Rc::strong_count(&cloned_ref));
    
    
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
