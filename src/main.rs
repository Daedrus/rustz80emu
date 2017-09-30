use std::path::Path;

extern crate z80emulib;
use z80emulib::machine::*;
use z80emulib::utils::read_bin;

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
    opts.optopt(
        "s",
        "snapshot",
        "Load a snapshot instead of booting from the default ROMs",
        "PATH");

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

    if let Some(snapshot_path) = matches.opt_str("s") {
        let snapshot_file = read_bin(Path::new(&snapshot_path));
        if let Some((header, data)) = z80emulib::snapshot::parse(&snapshot_file[..]) {
            let machine = Machine::from_snapshot(start_in_debug, &header, data);

            machine.run();
        }
    } else {
        let machine = Machine::new(start_in_debug);

        machine.run();
    }
}
