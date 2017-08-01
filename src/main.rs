extern crate z80emulib;

use z80emulib::machine::*;

use std::env;

fn main() {
    let mut machine = Machine::new();

    machine.run();
}
