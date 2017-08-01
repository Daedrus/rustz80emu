extern crate z80emulib;

use z80emulib::interconnect::*;
use z80emulib::peripherals::*;
use z80emulib::cpu::*;
use z80emulib::debugger::*;

use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

use std::rc::Rc;
use std::cell::RefCell;

fn read_bin<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf.into_boxed_slice()
}

fn main() {
    let rom0_file_name = env::args().nth(1).unwrap();
    let rom1_file_name = env::args().nth(2).unwrap();

    let rom0 = read_bin(rom0_file_name);
    let rom1 = read_bin(rom1_file_name);

    let memory = Rc::new(RefCell::new(MemoryBuilder::new()
        .rom0(rom0)
        .rom1(rom1)
        .finalize()));
    let ay = Rc::new(RefCell::new(Ay::new()));
    let ula = Rc::new(RefCell::new(Ula::new(memory.clone())));

    let interconnect = Interconnect::new(
        memory.clone(),
        ay.clone(),
        ula.clone());

    let mut cpu = Cpu::new(interconnect);

    if env::var("RUST_LOG").is_ok() {
        let mut debugger = Debugger::new(cpu, memory.clone());
        debugger.run();
    } else {
        cpu.run();
    }
}
