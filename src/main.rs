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
    let rom0_file_name = env::args().nth(1).unwrap();
    let rom1_file_name = env::args().nth(2).unwrap();

    let rom0 = read_bin(rom0_file_name);
    let rom1 = read_bin(rom1_file_name);

    let memory = memory::Memory::new(rom0, rom1);

    let mut cpu = cpu::Cpu::new(memory);

    cpu.run();
}
