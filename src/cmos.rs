#![no_std]

use x86_64::instructions::port::Port;

use core::{
    cmp::Ordering,
    fmt::{Display, Formatter, Result},
    usize,
};

#[derive(Debug)]
pub struct CMOS {
    address_port: Port<u8>,
    data_port: Port<u8>,
}

impl CMOS {
    /// Create a new CMOS struct
    pub unsafe fn new() -> CMOS {
        CMOS {
            address_port: Port::new(0x70),
            data_port: Port::new(0x71),
        }
    }

    //Read all registers in cmos
    pub fn read_all(&mut self, output: &mut [u8; 128]) {
        for i in 0..128 {
            self.address_port.write(i);
            output[i as usize] = self.data_port.read();
        }
    }

    pub fn read(&mut self, reg: u8) -> u8 {
        self.address_port.write(reg);
        self.data_port.read()
    }
}
