use ::cpu::*;
use ::peripherals::Memory;

use std::io::{stdin, stdout};
use std::io::Write;

use std::borrow::Cow;
use std::str::{self, FromStr};

use nom::{IResult, space, digit, hex_digit};

use log::LogRecord;
use env_logger::LogBuilder;
use std::env;

use std::rc::Rc;
use std::cell::RefCell;


#[derive(Debug, Clone, Copy)]
enum Command {
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
            err => Err(format!("Unable to parse command: {:?}", err).into()),
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
            eof!(),
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

// TODO: I have a feeling this can be done in a better way, without the unwrap()
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
            Reg16::IR => OI | OR,
            Reg16::AF_ALT => OA_ALT | OF_ALT,
            Reg16::BC_ALT => OB_ALT | OC_ALT,
            Reg16::DE_ALT => OD_ALT | OE_ALT,
            Reg16::HL_ALT => OH_ALT | OL_ALT,
            Reg16::SP => OSP,
            Reg16::IX => OIX,
            Reg16::IY => OIY,
            Reg16::WZ => OWZ,
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
            Reg8::A_ALT => OA_ALT,
            Reg8::B_ALT => OB_ALT,
            Reg8::C_ALT => OC_ALT,
            Reg8::D_ALT => OD_ALT,
            Reg8::E_ALT => OE_ALT,
            Reg8::H_ALT => OH_ALT,
            Reg8::L_ALT => OL_ALT,
            Reg8::F_ALT => OF_ALT,
            Reg8::IXL | Reg8::IXH => OIX,
            Reg8::IYL | Reg8::IYH => OIY,
        }
    }
}

macro_rules! reg_str {
    ($regs: ident, $outreg: ident, $fmtstr: expr, $reg: expr, $nostr: expr) => {{
        if $regs.contains($outreg) {
            format!($fmtstr, $reg)
        } else {
            String::from($nostr)
        }
    }}
}

pub struct Debugger {
    cpu: Cpu,
    memory: Rc<RefCell<Memory>>,
}

impl Debugger {
    pub fn new(cpu: Cpu, memory: Rc<RefCell<Memory>>) -> Self {
        Debugger {
            cpu: cpu,
            memory: memory,
        }
    }

    // TODO: Rewrite this mess
    fn output(&self, regs: OutputRegisters) -> String {
        let mut outstr = String::new();

        let astr =
            reg_str!(regs, OA    , " {:02X} ", self.cpu.read_reg8(Reg8::A)       , "    "      );
        let aaltstr =
            reg_str!(regs, OA_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::A_ALT)   , "    "      );
        let fstr =
            reg_str!(regs, OF    , " {:02X} ", self.cpu.get_flags().bits() as u8 , "    "      );
        let fbinstr =
            reg_str!(regs, OF    , " {:08b} ", self.cpu.get_flags().bits()       , "          ");
        let faltstr =
            reg_str!(regs, OF_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::F_ALT)   , "    "      );
        let bstr =
            reg_str!(regs, OB    , " {:02X} ", self.cpu.read_reg8(Reg8::B)       , "    "      );
        let baltstr =
            reg_str!(regs, OB_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::B_ALT)   , "    "      );
        let cstr =
            reg_str!(regs, OC    , " {:02X} ", self.cpu.read_reg8(Reg8::C)       , "    "      );
        let caltstr =
            reg_str!(regs, OC_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::C_ALT)   , "    "      );
        let dstr =
            reg_str!(regs, OD    , " {:02X} ", self.cpu.read_reg8(Reg8::D)       , "    "      );
        let daltstr =
            reg_str!(regs, OD_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::D_ALT)   , "    "      );
        let estr =
            reg_str!(regs, OE    , " {:02X} ", self.cpu.read_reg8(Reg8::E)       , "    "      );
        let ealtstr =
            reg_str!(regs, OE_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::E_ALT)   , "    "      );
        let hstr =
            reg_str!(regs, OH    , " {:02X} ", self.cpu.read_reg8(Reg8::H)       , "    "      );
        let haltstr =
            reg_str!(regs, OH_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::H_ALT)   , "    "      );
        let lstr =
            reg_str!(regs, OL    , " {:02X} ", self.cpu.read_reg8(Reg8::L)       , "    "      );
        let laltstr =
            reg_str!(regs, OL_ALT, " {:02X} ", self.cpu.read_reg8(Reg8::L_ALT)   , "    "      );
        let ixstr =
            reg_str!(regs, OIX   , " {:04X} ", self.cpu.read_reg16(Reg16::IX)    , "      "    );
        let iystr =
            reg_str!(regs, OIY   , " {:04X} ", self.cpu.read_reg16(Reg16::IY)    , "      "    );
        let spstr =
            reg_str!(regs, OSP   , " {:04X} ", self.cpu.read_reg16(Reg16::SP)    , "      "    );

        let pcstr = format!(" {:04X} ", self.cpu.get_pc());

        let tcyclesstr = format!("{:05}", self.cpu.tcycles);
        let istr = format!("{:02X}", self.cpu.read_reg8(Reg8::I));
        let rstr = format!("{:02X}", self.cpu.read_reg8(Reg8::R));

        let mem0str = format!("{}", self.memory.borrow().get_0000_bank());
        let mem4str = format!("{}", self.memory.borrow().get_4000_bank());
        let mem8str = format!("{}", self.memory.borrow().get_8000_bank());
        let memcstr = format!("{}", self.memory.borrow().get_c000_bank());

        let imstr = format!("{}", self.cpu.get_im());
        let iff1str = format!("{}", if self.cpu.get_iff1() {1} else {0});
        let iff2str = format!("{}", if self.cpu.get_iff2() {1} else {0});

        outstr.push_str("                    -----------        -----------\n");
        outstr.push_str("                af: |");
        outstr.push_str(&astr);
        outstr.push_str("|");
        outstr.push_str(&fstr);
        outstr.push_str("|   af': |");
        outstr.push_str(&aaltstr);
        outstr.push_str("|");
        outstr.push_str(&faltstr);
        outstr.push_str("|   tcycles: ");
        outstr.push_str(&tcyclesstr);
        outstr.push_str("    im: ");
        outstr.push_str(&imstr);
        outstr.push_str("\n");
        outstr.push_str("                bc: |");
        outstr.push_str(&bstr);
        outstr.push_str("|");
        outstr.push_str(&cstr);
        outstr.push_str("|   bc': |");
        outstr.push_str(&baltstr);
        outstr.push_str("|");
        outstr.push_str(&caltstr);
        outstr.push_str("|         i: ");
        outstr.push_str(&istr);
        outstr.push_str("     iff1: ");
        outstr.push_str(&iff1str);
        outstr.push_str("\n");
        outstr.push_str("                de: |");
        outstr.push_str(&dstr);
        outstr.push_str("|");
        outstr.push_str(&estr);
        outstr.push_str("|   de': |");
        outstr.push_str(&daltstr);
        outstr.push_str("|");
        outstr.push_str(&ealtstr);
        outstr.push_str("|         r: ");
        outstr.push_str(&rstr);
        outstr.push_str("     iff2: ");
        outstr.push_str(&iff2str);
        outstr.push_str("\n");
        outstr.push_str("                hl: |");
        outstr.push_str(&hstr);
        outstr.push_str("|");
        outstr.push_str(&lstr);
        outstr.push_str("|   hl': |");
        outstr.push_str(&haltstr);
        outstr.push_str("|");
        outstr.push_str(&laltstr);
        outstr.push_str("|\n");
        outstr.push_str("                    -----------        -----------\n");
        outstr.push_str("                    ----------         ------------\n");
        outstr.push_str("                ix: | ");
        outstr.push_str(&ixstr);
        outstr.push_str(" |         | SZ_H_PNC |");
        outstr.push_str("   0x0000: ROM ");
        outstr.push_str(&mem0str);
        outstr.push_str("\n");
        outstr.push_str("                iy: | ");
        outstr.push_str(&iystr);
        outstr.push_str(" |      f: |");
        outstr.push_str(&fbinstr);
        outstr.push_str("|");
        outstr.push_str("   0x4000: RAM ");
        outstr.push_str(&mem4str);
        outstr.push_str("\n");
        outstr.push_str("                sp: | ");
        outstr.push_str(&spstr);
        outstr.push_str(" |         ------------");
        outstr.push_str("   0x8000: RAM ");
        outstr.push_str(&mem8str);
        outstr.push_str("\n");
        outstr.push_str("                pc: | ");
        outstr.push_str(&pcstr);
        outstr.push_str(" |");
        outstr.push_str("                        0xC000: RAM ");
        outstr.push_str(&memcstr);
        outstr.push_str("\n");
        outstr.push_str("                    ----------\n");

        outstr
    }

    fn peek_at_next_instruction(&self) -> &Instruction {
        let curr_pc = self.cpu.get_pc();

        let i0 = self.memory.borrow().read_word(curr_pc);
        let i1 = self.memory.borrow().read_word(curr_pc + 1);
        let i3 = self.memory.borrow().read_word(curr_pc + 3);

        match (i0, i1) {
            (0xDD, 0xCB) => INSTR_TABLE_DDCB[i3 as usize],
            (0xDD, _   ) => INSTR_TABLE_DD[i1 as usize],
            (0xFD, 0xCB) => INSTR_TABLE_FDCB[i3 as usize],
            (0xFD, _   ) => INSTR_TABLE_FD[i1 as usize],
            (0xCB, _   ) => INSTR_TABLE_CB[i1 as usize],
            (0xED, _   ) => INSTR_TABLE_ED[i1 as usize],
            (_   , _   ) => INSTR_TABLE[i0 as usize],
        }
    }

    pub fn run(&mut self) {
        let mut builder = LogBuilder::new();

        let format = |record: &LogRecord| format!("{}", record.args());
        builder.format(format);

        if env::var("RUST_LOG").is_ok() {
            builder.parse(&env::var("RUST_LOG").unwrap());
        }

        builder.init().unwrap();

        loop {
            print!("z80> ");
            stdout().flush().unwrap();

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            let input: String = input.trim().into();

            match input.parse() {
                Ok(Command::Exit) => break,

                Ok(Command::Cont) => {
                    loop {
                        let (pre_regs, post_regs) =
                            self.peek_at_next_instruction().get_accessed_regs();
                        debug!("{}", self.output(pre_regs));
                        self.cpu.handle_interrupts();
                        self.cpu.run_instruction();
                        debug!("{}", self.output(post_regs));
                    }
                }

                Ok(Command::Step(count)) if count > 0 => {
                    for _ in 0..count {
                        let (pre_regs, post_regs) =
                            self.peek_at_next_instruction().get_accessed_regs();
                        debug!("{}", self.output(pre_regs));
                        self.cpu.handle_interrupts();
                        self.cpu.run_instruction();
                        debug!("{}", self.output(post_regs));
                    }
                }

                Ok(Command::Mem(addr)) => println!("{:#04X}", self.memory.borrow().read_word(addr)),

                _ => println!("Unknown command"),
            };
        }
    }
}
