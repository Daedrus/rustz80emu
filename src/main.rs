#[macro_use] extern crate enum_primitive;
extern crate num;
#[macro_use] extern crate bitflags;

mod cpu;
mod memory;
mod instructions;

use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

fn read_bin<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    let rom = read_bin(rom_file_name);
    let ram = vec![0; 4 * 1024];

    let mut memory = memory::Memory::new(rom, ram);

    let mut cpu = cpu::Cpu::new(memory);

    cpu.run();
}
