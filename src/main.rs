extern crate z80emulib;

#[macro_use] extern crate log;
extern crate env_logger;
use log::{LogRecord};
use env_logger::LogBuilder;

use z80emulib::memory::*;
use z80emulib::cpu::*;

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

fn setup_logging() {
    let mut builder = LogBuilder::new();

    let format = |record: &LogRecord| { format!("{}", record.args()) };
    builder.format(format);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init().unwrap();
}

fn main() {
    setup_logging();

    let rom0_file_name = env::args().nth(1).unwrap();
    let rom1_file_name = env::args().nth(2).unwrap();

    let rom0 = read_bin(rom0_file_name);
    let rom1 = read_bin(rom1_file_name);

    let memory = Memory::new(rom0, rom1);

    let mut cpu = Cpu::new(memory);

    cpu.run();
}
