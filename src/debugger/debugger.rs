use ::cpu::*;
use ::peripherals::Memory;
use self::output_registers::*;

use std::io::{stdin, stdout};
use std::io::Write;

use std::borrow::Cow;
use std::str::{self, FromStr};

use nom::{IResult, space, digit, hex_digit};

use std::rc::Rc;
use std::cell::RefCell;


#[derive(Debug, Clone, Copy)]
enum Command {
    Step(u16),
    Mem(u16),
    MemRng(u16, u16),
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
    do_parse!(
        c: alt_complete!(
            step |
            mem  |
            cont |
            exit) >>
        eof!() >>

        (c)
    )
);

named!(
    step<Command>,
    do_parse!(
        alt_complete!(tag!("step") | tag!("s")) >>
        count: opt!(complete!(preceded!(space, u16_parser))) >>

        (Command::Step(count.unwrap_or(1)))
    )
);

named!(
    mem<Command>,
    do_parse!(
        alt_complete!(tag!("mem") | tag!("m")) >>
        addrs: many_m_n!(1, 2, preceded!(space, u16_hex_parser)) >>

        (match addrs.len() {
            1 => Command::Mem(addrs[0]),
            2 => Command::MemRng(addrs[0], addrs[1]),
            _ => unreachable!(),
        })
    )
);

named!(
    cont<Command>,
    do_parse!(
        alt_complete!(tag!("cont") | tag!("c")) >>

        (Command::Cont)
    )
);

named!(
    exit<Command>,
    do_parse!(
        alt_complete!(tag!("exit") | tag!("e") | tag!("quit") | tag!("q")) >>

        (Command::Exit)
    )
);

named!(u16_parser<u16>,
    map_res!(
        map_res!(
            digit,
            str::from_utf8),
        FromStr::from_str));

named!(u16_hex_parser<u16>,
    do_parse!(
        opt!(tag!("0x")) >>
        number: map_res!(
            hex_digit,
            str::from_utf8) >>
        u16number: expr_res!(u16::from_str_radix(number, 16)) >>
        (u16number)
    )
);


pub mod output_registers {

    use ::cpu::{Reg8, Reg16};

    bitflags! {
        pub struct OutputRegisters: u32 {
            const ONONE = 0x00000000;

            const OA = 0x00000001;
            const OF = 0x00000002;
            const OB = 0x00000004;
            const OC = 0x00000008;
            const OD = 0x00000010;
            const OE = 0x00000020;
            const OH = 0x00000040;
            const OL = 0x00000080;

            const OA_ALT = 0x00000100;
            const OF_ALT = 0x00000200;
            const OB_ALT = 0x00000400;
            const OC_ALT = 0x00000800;
            const OD_ALT = 0x00001000;
            const OE_ALT = 0x00002000;
            const OH_ALT = 0x00004000;
            const OL_ALT = 0x00008000;

            const OIX = 0x00010000;
            const OIY = 0x00020000;
            const OSP = 0x00040000;
            const OPC = 0x00080000;
            const OWZ = 0x00100000;

            const OI = 0x00200000;
            const OR = 0x00400080;

            const OALL = 0xFFFFFFFF;
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
}

pub struct Debugger {
    cpu: Rc<RefCell<Cpu>>,
    memory: Rc<RefCell<Memory>>,
}

impl Debugger {
    pub fn new(cpu: Rc<RefCell<Cpu>>, memory: Rc<RefCell<Memory>>) -> Self {
        Debugger {
            cpu: cpu,
            memory: memory,
        }
    }

    fn output(&self, regs: OutputRegisters) -> String {
        macro_rules! reg_str {
            ($regs: ident, $outreg: ident, $fmtstr: expr, $reg: expr, $nostr: expr) => {{
                if $regs.contains($outreg) {
                    format!($fmtstr, $reg)
                } else {
                    String::from($nostr)
                }
            }}
        }

        format!(
            "                 -----------        -----------
             af: |{}|{}|   af': |{}|{}|   tcycles: {}    im: {}
             bc: |{}|{}|   bc': |{}|{}|         i: {}     iff1: {}
             de: |{}|{}|   de': |{}|{}|         r: {}     iff2: {}
             hl: |{}|{}|   hl': |{}|{}|
                 -----------        -----------
                 ----------         ------------
             ix: | {} |         | SZ_H_PNC |   0x0000: ROM {}
             iy: | {} |      f: |{}|   0x4000: ROM {}
             sp: | {} |         ------------   0x8000: ROM {}
             pc: | {} |                        0xC000: ROM {}
                 ----------",
            reg_str!(regs, OA    , " {:02X} ", self.cpu.borrow().read_reg8(Reg8::A)       , "    "      ),
            reg_str!(regs, OF    , " {:02X} ", self.cpu.borrow().get_flags().bits() as u8 , "    "      ),
            reg_str!(regs, OA_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::A_ALT)   , "    "      ),
            reg_str!(regs, OF_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::F_ALT)   , "    "      ),
            format!("{:05}", self.cpu.borrow().tcycles),
            format!("{}", self.cpu.borrow().get_im()),
            reg_str!(regs, OB    , " {:02X} ", self.cpu.borrow().read_reg8(Reg8::B)       , "    "      ),
            reg_str!(regs, OC    , " {:02X} ", self.cpu.borrow().read_reg8(Reg8::C)       , "    "      ),
            reg_str!(regs, OB_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::B_ALT)   , "    "      ),
            reg_str!(regs, OC_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::C_ALT)   , "    "      ),
            format!("{:02X}", self.cpu.borrow().read_reg8(Reg8::I)),
            format!("{}", if self.cpu.borrow().get_iff1() {1} else {0}),
            reg_str!(regs, OD    , " {:02X} ", self.cpu.borrow().read_reg8(Reg8::D)       , "    "      ),
            reg_str!(regs, OE    , " {:02X} ", self.cpu.borrow().read_reg8(Reg8::E)       , "    "      ),
            reg_str!(regs, OD_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::D_ALT)   , "    "      ),
            reg_str!(regs, OE_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::E_ALT)   , "    "      ),
            format!("{:02X}", self.cpu.borrow().read_reg8(Reg8::R)),
            format!("{}", if self.cpu.borrow().get_iff2() {1} else {0}),
            reg_str!(regs, OH    , " {:02X} ", self.cpu.borrow().read_reg8(Reg8::H)       , "    "      ),
            reg_str!(regs, OL    , " {:02X} ", self.cpu.borrow().read_reg8(Reg8::L)       , "    "      ),
            reg_str!(regs, OH_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::H_ALT)   , "    "      ),
            reg_str!(regs, OL_ALT, " {:02X} ", self.cpu.borrow().read_reg8(Reg8::L_ALT)   , "    "      ),
            reg_str!(regs, OIX   , " {:04X} ", self.cpu.borrow().read_reg16(Reg16::IX)    , "      "    ),
            format!("{}", self.memory.borrow().get_0000_bank()),
            reg_str!(regs, OIY   , " {:04X} ", self.cpu.borrow().read_reg16(Reg16::IY)    , "      "    ),
            reg_str!(regs, OF    , " {:08b} ", self.cpu.borrow().get_flags().bits()       , "          "),
            format!("{}", self.memory.borrow().get_4000_bank()),
            reg_str!(regs, OSP   , " {:04X} ", self.cpu.borrow().read_reg16(Reg16::SP)    , "      "    ),
            format!("{}", self.memory.borrow().get_8000_bank()),
            format!(" {:04X} ", self.cpu.borrow().get_pc()),
            format!("{}", self.memory.borrow().get_c000_bank()))
    }

    fn peek_at_next_instruction(&self) -> &Instruction {
        let curr_pc = self.cpu.borrow().get_pc();

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
                        let (pre_regs, post_regs, instr_str) = {
                            let next_instr = self.peek_at_next_instruction();
                            let accessed_regs = next_instr.get_accessed_regs();
                            (accessed_regs.0,
                             accessed_regs.1,
                             next_instr.get_string(&self.cpu.borrow(), &self.memory.borrow()))
                        };
                        println!("{}", self.output(pre_regs));
                        println!("{}", instr_str);
                        self.cpu.borrow_mut().handle_interrupts();
                        self.cpu.borrow_mut().run_instruction();
                        println!("{}", self.output(post_regs));
                    }
                }

                Ok(Command::Step(count)) if count > 0 => {
                    for _ in 0..count {
                        let (pre_regs, post_regs, instr_str) = {
                            let next_instr = self.peek_at_next_instruction();
                            let accessed_regs = next_instr.get_accessed_regs();
                            (accessed_regs.0,
                             accessed_regs.1,
                             next_instr.get_string(&self.cpu.borrow(), &self.memory.borrow()))
                        };
                        println!("{}", self.output(pre_regs));
                        println!("{}", instr_str);
                        self.cpu.borrow_mut().handle_interrupts();
                        self.cpu.borrow_mut().run_instruction();
                        println!("{}", self.output(post_regs));
                    }
                }

                Ok(Command::MemRng(addrstart, addrend)) => {
                    let realaddrstart =
                        if addrstart % 16 == 0 {
                            addrstart
                        } else {
                            addrstart - (addrstart % 16)
                        };
                    let realaddrend =
                        if addrend % 16 == 0 {
                            addrend
                        } else {
                            addrend + (16 - (addrend % 16))
                        };
                    for addr in realaddrstart..realaddrend {
                        if addr % 16 == 0 {
                            println!();
                            print!("{:#06X}: ", addr);
                        }
                        print!("{:02X} ", self.memory.borrow().read_word(addr));
                    }
                    println!();
                }

                Ok(Command::Mem(addr)) => println!("{:#04X}", self.memory.borrow().read_word(addr)),

                _ => println!("Unknown command"),
            };
        }
    }
}
