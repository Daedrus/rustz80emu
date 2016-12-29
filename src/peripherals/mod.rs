mod memory;
mod ay;
mod ula;

pub use peripherals::memory::*;
pub use peripherals::ula::*;
pub use peripherals::ay::*;

pub trait Peripheral {
    fn read_port(&self, port: u16) -> u8;
    fn write_port(&mut self, port: u16, val: u8);
}
