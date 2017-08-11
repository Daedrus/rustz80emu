extern crate z80emulib;
use z80emulib::machine::*;

extern crate getopts;
use getopts::Options;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optflag(
        "d",
        "debug",
        "Start the emulator with debugger on, break on first instruction");
    opts.optflag(
        "h",
        "help",
        "Print this help menu (all other options are ignored)");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", &args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    let mut start_in_debug = false;
    if matches.opt_present("d") {
        start_in_debug = true;
    }

    let machine = Machine::new(start_in_debug);

    machine.run();
}
