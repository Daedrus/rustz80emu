use super::cpu::{Cpu};
use std::io::{stdin, stdout};
use std::io::Write;

use std::borrow::Cow;
use std::str::{self, FromStr};

use nom::{IResult, eof, space, digit, hex_digit};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Step(u16),
    Mem(u16),
    Cont,
    Exit,
}

impl FromStr for Command {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match command(s.as_bytes()) {
            IResult::Done(_, c) => Ok(c),
            err => Err(format!("Unable to parse command: {:?}", err).into())
        }
    }
}

named!(
    command<Command>,
    chain!(
        c: alt_complete!(
            step |
            mem  |
            cont |
            exit) ~
            eof,
    || c));

named!(
    step<Command>,
    chain!(
        alt_complete!(tag!("step") | tag!("s")) ~
        count: opt!(preceded!(space, u16_parser)),
    || Command::Step(count.unwrap_or(1))));

named!(
    mem<Command>,
    chain!(
        alt_complete!(tag!("mem") | tag!("m")) ~
        addr: preceded!(space, u16_hex_parser),
    || Command::Mem(addr)));

named!(
    cont<Command>,
    map!(
        alt_complete!(tag!("cont") | tag!("c")),
    |_| Command::Cont));

named!(
    exit<Command>,
    map!(
        alt_complete!(tag!("exit") | tag!("e") | tag!("quit") | tag!("q")),
    |_| Command::Exit));

named!(u16_parser<u16>,
    map_res!(
        map_res!(
            digit,
            str::from_utf8),
        FromStr::from_str));

//TODO: I have a feeling this can be done in a better way, without the unwrap()
named!(u16_hex_parser<u16>,
    chain!(
        opt!(tag!("0x")) ~
        number: map_res!(
            hex_digit,
            str::from_utf8),
    || u16::from_str_radix(number, 16).unwrap()));


pub struct Debugger {
    cpu: Cpu
}

impl Debugger {
    pub fn new(cpu: Cpu) -> Debugger {
        Debugger {
            cpu: cpu
        }
    }

    pub fn run(&mut self) {
        loop {
            print!("z80> ");
            stdout().flush().unwrap();

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            let input: String = input.trim().into();

            match input.parse() {
                Ok(Command::Exit) =>
                    break,

                Ok(Command::Cont) =>
                    self.cpu.run(),

                Ok(Command::Step(count)) if count > 0 =>
                    for _ in 0..count { self.cpu.run_instruction() },

                Ok(Command::Mem(addr)) =>
                    println!("{:#04X}", self.cpu.read_word(addr)),

                _ =>
                    println!("Unknown command"),
            };
        }
    }
}
