use super::cpu::{Cpu, Reg8, Reg16};
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


bitflags! {
    pub flags OutputRegisters: u32 {
        const ONONE = 0x00000000,

        const OA = 0x00000001,
        const OF = 0x00000002,
        const OB = 0x00000004,
        const OC = 0x00000008,
        const OD = 0x00000010,
        const OE = 0x00000020,
        const OH = 0x00000040,
        const OL = 0x00000080,

        const OA_ALT = 0x00000100,
        const OF_ALT = 0x00000200,
        const OB_ALT = 0x00000400,
        const OC_ALT = 0x00000800,
        const OD_ALT = 0x00001000,
        const OE_ALT = 0x00002000,
        const OH_ALT = 0x00004000,
        const OL_ALT = 0x00008000,

        const OIX = 0x00010000,
        const OIY = 0x00020000,
        const OSP = 0x00040000,
        const OPC = 0x00080000,
        const OWZ = 0x00100000,

        const OI = 0x00200000,
        const OR = 0x00400080,

        const OALL = 0xFFFFFFFF,
    }
}

impl From<Reg16> for OutputRegisters {
    fn from(r: Reg16) -> OutputRegisters {
        match r {
            Reg16::AF => OA | OF,
            Reg16::BC => OB | OC,
            Reg16::DE => OD | OE,
            Reg16::HL => OH | OL,
            Reg16::AF_ALT => OA_ALT | OF_ALT,
            Reg16::BC_ALT => OB_ALT | OC_ALT,
            Reg16::DE_ALT => OD_ALT | OE_ALT,
            Reg16::HL_ALT => OH_ALT | OL_ALT,
            Reg16::SP => OSP,
            Reg16::IX => OIX,
            Reg16::IY => OIY,
            Reg16::WZ => OWZ
        }
    }
}

impl From<Reg8> for OutputRegisters {
    fn from(r: Reg8) -> OutputRegisters {
        match r {
            Reg8::A => OA,
            Reg8::B => OB,
            Reg8::C => OC,
            Reg8::D => OD,
            Reg8::E => OE,
            Reg8::H => OH,
            Reg8::L => OL,
            Reg8::I => OI,
            Reg8::R => OR,
            Reg8::IXL | Reg8::IXH => OIX,
            Reg8::IYL | Reg8::IYH => OIY
        }
    }
}


pub struct Debugger {
    cpu: Cpu
}

impl Debugger {
    pub fn new(cpu: Cpu) -> Debugger {
        Debugger {
            cpu: cpu
        }
    }

    //TODO: Rewrite this mess
    pub fn output(&self, regs: OutputRegisters) -> String {
        let mut outstr = String::new();

        let astr = if regs.contains(OA) { format!(" {:02X} ", self.cpu.a) } else { String::from("    ") };
        let fstr = if regs.contains(OF) { format!(" {:02X} ", self.cpu.f.bits() as u8) } else { String::from("    ") };
        let fbinstr = if regs.contains(OF) { format!(" {:08b} ", self.cpu.f.bits()) } else { String::from("          ") };
        let aaltstr = if regs.contains(OA_ALT) { format!(" {:02X} ", self.cpu.a_alt) } else { String::from("    ") };
        let faltstr = if regs.contains(OF_ALT) { format!(" {:02X} ", self.cpu.f_alt.bits() as u8) } else { String::from("    ") };
        let bstr = if regs.contains(OB) { format!(" {:02X} ", self.cpu.b) } else { String::from("    ") };
        let cstr = if regs.contains(OC) { format!(" {:02X} ", self.cpu.c) } else { String::from("    ") };
        let baltstr = if regs.contains(OB_ALT) { format!(" {:02X} ", self.cpu.b_alt) } else { String::from("    ") };
        let caltstr = if regs.contains(OC_ALT) { format!(" {:02X} ", self.cpu.c_alt) } else { String::from("    ") };
        let dstr = if regs.contains(OD) { format!(" {:02X} ", self.cpu.d) } else { String::from("    ") };
        let estr = if regs.contains(OE) { format!(" {:02X} ", self.cpu.e) } else { String::from("    ") };
        let daltstr = if regs.contains(OD_ALT) { format!(" {:02X} ", self.cpu.d_alt) } else { String::from("    ") };
        let ealtstr = if regs.contains(OE_ALT) { format!(" {:02X} ", self.cpu.e_alt) } else { String::from("    ") };
        let hstr = if regs.contains(OH) { format!(" {:02X} ", self.cpu.h) } else { String::from("    ") };
        let lstr = if regs.contains(OL) { format!(" {:02X} ", self.cpu.l) } else { String::from("    ") };
        let haltstr = if regs.contains(OH_ALT) { format!(" {:02X} ", self.cpu.h_alt) } else { String::from("    ") };
        let laltstr = if regs.contains(OL_ALT) { format!(" {:02X} ", self.cpu.l_alt) } else { String::from("    ") };
        let ixstr = if regs.contains(OIX) { format!(" {:04X} ", self.cpu.ix) } else { String::from("      ") };
        let iystr = if regs.contains(OIY) { format!(" {:04X} ", self.cpu.iy) } else { String::from("      ") };
        let spstr = if regs.contains(OSP) { format!(" {:04X} ", self.cpu.sp) } else { String::from("      ") };
        let pcstr = format!(" {:04X} ", self.cpu.pc);

        outstr.push_str("                    -----------           -----------\n");
        outstr.push_str("                af: |"); outstr.push_str(&astr); outstr.push_str("|"); outstr.push_str(&fstr);
        outstr.push_str("|   af_alt: |"); outstr.push_str(&aaltstr); outstr.push_str("|"); outstr.push_str(&faltstr);
        outstr.push_str("|\n");
        outstr.push_str("                bc: |"); outstr.push_str(&bstr); outstr.push_str("|"); outstr.push_str(&cstr);
        outstr.push_str("|   bc_alt: |"); outstr.push_str(&baltstr); outstr.push_str("|"); outstr.push_str(&caltstr);
        outstr.push_str("|\n");
        outstr.push_str("                de: |"); outstr.push_str(&dstr); outstr.push_str("|"); outstr.push_str(&estr);
        outstr.push_str("|   de_alt: |"); outstr.push_str(&daltstr); outstr.push_str("|"); outstr.push_str(&ealtstr);
        outstr.push_str("|\n");
        outstr.push_str("                hl: |"); outstr.push_str(&hstr); outstr.push_str("|"); outstr.push_str(&lstr);
        outstr.push_str("|   hl_alt: |"); outstr.push_str(&haltstr); outstr.push_str("|"); outstr.push_str(&laltstr);
        outstr.push_str("|\n");
        outstr.push_str("                    -----------           -----------\n");
        outstr.push_str("                    ----------            ------------\n");
        outstr.push_str("                ix: | "); outstr.push_str(&ixstr);
        outstr.push_str(" |            | SZ_H_PNC |\n");
        outstr.push_str("                iy: | "); outstr.push_str(&iystr);
        outstr.push_str(" |         f: |"); outstr.push_str(&fbinstr);
        outstr.push_str("|\n");
        outstr.push_str("                sp: | "); outstr.push_str(&spstr);
        outstr.push_str(" |            ------------\n");
        outstr.push_str("                pc: | "); outstr.push_str(&pcstr);
        outstr.push_str(" |\n");
        outstr.push_str("                    ----------\n");

        outstr
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
                    loop {
                        let (pre_regs, post_regs) = self.cpu.decode_instruction().get_accessed_regs();
                        debug!("{}", self.output(pre_regs));
                        self.cpu.run_instruction();
                        debug!("{}", self.output(post_regs));
                    },

                Ok(Command::Step(count)) if count > 0 =>
                    for _ in 0..count {
                        let (pre_regs, post_regs) = self.cpu.decode_instruction().get_accessed_regs();
                        debug!("{}", self.output(pre_regs));
                        self.cpu.run_instruction();
                        debug!("{}", self.output(post_regs));
                    },

                Ok(Command::Mem(addr)) =>
                    println!("{:#04X}", self.cpu.read_word(addr)),

                _ =>
                    println!("Unknown command"),
            };
        }
    }
}
