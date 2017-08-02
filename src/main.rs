extern crate z80emulib;

use z80emulib::machine::*;

use std::env;

fn main() {
    // TODO: Use a proper command line argument parser
    let mut start_in_debug: bool = false;

    for argument in env::args() {
        match &argument[..] {
            "start_in_debug" => start_in_debug = true,
            _ => {},
        }
    }

    let machine = Machine::new(start_in_debug);

    machine.run();
}
