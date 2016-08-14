use super::memory;
use std::fmt;
use std::io::{stdin, stdout};
use std::io::Write;
use super::instructions;

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum Reg16 {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,

    BC_ALT = 0b100,
    DE_ALT = 0b101,
    HL_ALT = 0b110
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum Reg16qq {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    AF = 0b11,

    AF_ALT = 0b111
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum Reg8 {
    A = 0b111,
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum Port {
    MEMORY = 0x7ffd,
    AY38912_REG14 = 0xfffd,
    AY38912_REG14_W = 0xbffd,
    FE = 0xfe
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum FlagCond {
    NZ = 0b000,
    Z  = 0b001,
    NC = 0b010,
    C  = 0b011,
    PO = 0b100,
    PE = 0b101,
    P  = 0b110,
    M  = 0b111
}
}

bitflags! {
    pub flags StatusIndicatorFlags: u8 {
        const CARRY_FLAG           = 0b00000001, // C
        const ADD_SUBTRACT_FLAG    = 0b00000010, // N
        const PARITY_OVERFLOW_FLAG = 0b00000100, // P/V
        const HALF_CARRY_FLAG      = 0b00010000, // H
        const ZERO_FLAG            = 0b01000000, // Z
        const SIGN_FLAG            = 0b10000000  // S
    }
}

bitflags! {
    pub flags OutputRegisters: u32 {
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

        const OALL = 0xFFFFFFFF,
    }
}

impl From<Reg16> for OutputRegisters {
    fn from(r: Reg16) -> OutputRegisters {
        match r {
            Reg16::BC => OB | OC,
            Reg16::DE => OD | OE,
            Reg16::HL => OH | OL,
            Reg16::SP => OSP,
            Reg16::BC_ALT => OB_ALT | OC_ALT,
            Reg16::DE_ALT => OD_ALT | OE_ALT,
            Reg16::HL_ALT => OH_ALT | OL_ALT
        }
    }
}

impl From<Reg16qq> for OutputRegisters {
    fn from(r: Reg16qq) -> OutputRegisters {
        match r {
            Reg16qq::BC => OB | OC,
            Reg16qq::DE => OD | OE,
            Reg16qq::HL => OH | OL,
            Reg16qq::AF     => OA | OF,
            Reg16qq::AF_ALT => OA_ALT | OF_ALT
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
            Reg8::L => OL
        }
    }
}

pub struct Cpu {
    // main register set
    a: u8, f: StatusIndicatorFlags,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,

    // alternate register set
    a_alt: u8, f_alt: StatusIndicatorFlags,
    b_alt: u8, c_alt: u8,
    d_alt: u8, e_alt: u8,
    h_alt: u8, l_alt: u8,

    // interrupt vector
    i: u8,

    // memory refresh
    r: u8,

    // index register X
    ix: u16,

    // index register Y
    iy: u16,

    // stack pointer
    sp: u16,

    // program counter
    pc: u16,

    // interrupt flip-flops
    iff1: bool,
    iff2: bool,

    // interrupt mode
    im: u8,

    memory: memory::Memory
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.output(OALL))
    }
}

impl Cpu {
    pub fn new(memory: memory::Memory) -> Cpu {
        Cpu {
            a: 0, f: StatusIndicatorFlags::empty(),
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            a_alt: 0, f_alt: StatusIndicatorFlags::empty(),
            b_alt: 0, c_alt: 0,
            d_alt: 0, e_alt: 0,
            h_alt: 0, l_alt: 0,
            i: 0,
            r: 0,
            ix: 0,
            iy: 0,
            sp: 0,
            pc: 0,
            iff1: false,
            iff2: false,
            im: 0,

            memory: memory
        }
    }

    pub fn output(&self, regs: OutputRegisters) -> String {
        let mut outstr = String::new();

        let astr = if regs.contains(OA) { format!(" {:02X} ", self.a) } else { String::from("    ") };
        let fstr = if regs.contains(OF) { format!(" {:02X} ", self.f.bits() as u8) } else { String::from("    ") };
        let fbinstr = if regs.contains(OF) { format!(" {:08b} ", self.f.bits()) } else { String::from("          ") };
        let aaltstr = if regs.contains(OA_ALT) { format!(" {:02X} ", self.a_alt) } else { String::from("    ") };
        let faltstr = if regs.contains(OF_ALT) { format!(" {:02X} ", self.f_alt.bits() as u8) } else { String::from("    ") };
        let bstr = if regs.contains(OB) { format!(" {:02X} ", self.b) } else { String::from("    ") };
        let cstr = if regs.contains(OC) { format!(" {:02X} ", self.c) } else { String::from("    ") };
        let baltstr = if regs.contains(OB_ALT) { format!(" {:02X} ", self.b_alt) } else { String::from("    ") };
        let caltstr = if regs.contains(OC_ALT) { format!(" {:02X} ", self.c_alt) } else { String::from("    ") };
        let dstr = if regs.contains(OD) { format!(" {:02X} ", self.d) } else { String::from("    ") };
        let estr = if regs.contains(OE) { format!(" {:02X} ", self.e) } else { String::from("    ") };
        let daltstr = if regs.contains(OD_ALT) { format!(" {:02X} ", self.d_alt) } else { String::from("    ") };
        let ealtstr = if regs.contains(OE_ALT) { format!(" {:02X} ", self.e_alt) } else { String::from("    ") };
        let hstr = if regs.contains(OH) { format!(" {:02X} ", self.h) } else { String::from("    ") };
        let lstr = if regs.contains(OL) { format!(" {:02X} ", self.l) } else { String::from("    ") };
        let haltstr = if regs.contains(OH_ALT) { format!(" {:02X} ", self.h_alt) } else { String::from("    ") };
        let laltstr = if regs.contains(OL_ALT) { format!(" {:02X} ", self.l_alt) } else { String::from("    ") };
        let ixstr = if regs.contains(OIX) { format!(" {:04X} ", self.ix) } else { String::from("      ") };
        let iystr = if regs.contains(OIY) { format!(" {:04X} ", self.iy) } else { String::from("      ") };
        let spstr = if regs.contains(OSP) { format!(" {:04X} ", self.sp) } else { String::from("      ") };
        let pcstr = format!(" {:04X} ", self.pc);

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
        let mut debug_on = true;

        loop {
            if debug_on {
                print!("z80> ");
                stdout().flush().unwrap();

                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                let input: String = input.trim().into();

                let cmd: Vec<&str> = input.split(" ").collect();

                match cmd[0].as_ref() {
                    "step" => self.run_instruction(),
                    "regs" => println!("{:?}", self),
                    "run"  => debug_on = false,
                    "exit" => break,
                    "mem"  => println!("{:#04X}", self.read_word(u16::from_str_radix(cmd[1], 16).unwrap())),
                    _ => println!("Unknown command")
                };
            } else {
                self.run_instruction();
                // Poor man's breakpoint:
                // if self.get_pc() == ???? { debug_on = true };
            }
        }
    }

    pub fn read_reg8(&self, reg: Reg8) -> u8 {
        let val = match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l
        };

        debug!("                Read value {:#04X} from register {:?}", val, reg);
        val
    }

    pub fn write_reg8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val
        }

        debug!("                Write value {:#04X} to register {:?}", val, reg);
    }

    pub fn read_reg16(&self, reg: Reg16) -> u16 {
        let val = match reg {
            Reg16::SP => self.sp,
            _ => {
                let (high, low) = match reg {
                    Reg16::BC => (self.b, self.c),
                    Reg16::DE => (self.d, self.e),
                    Reg16::HL => (self.h, self.l),
                    Reg16::BC_ALT => (self.b_alt, self.c_alt),
                    Reg16::DE_ALT => (self.d_alt, self.e_alt),
                    Reg16::HL_ALT => (self.h_alt, self.l_alt),
                    _ => unreachable!()
                };
                (((high as u16) << 8 ) | low as u16)
            }
        };

        debug!("                Read value {:#04X} from register {:?}", val, reg);
        val
    }

    pub fn write_reg16(&mut self, reg: Reg16, val: u16) {
        let (high, low) = (((val & 0xFF00) >> 8) as u8, (val & 0x00FF) as u8);
        match reg {
            Reg16::BC => { self.b = high; self.c = low; }
            Reg16::DE => { self.d = high; self.e = low; }
            Reg16::HL => { self.h = high; self.l = low; }
            Reg16::BC_ALT => { self.b_alt = high; self.c_alt = low; }
            Reg16::DE_ALT => { self.d_alt = high; self.e_alt = low; }
            Reg16::HL_ALT => { self.h_alt = high; self.l_alt = low; }
            Reg16::SP => { self.sp = val }
        }

        debug!("                Write value {:#06X} to register {:?}", val, reg);
    }

    pub fn read_reg16qq(&self, reg: Reg16qq) -> u16 {
        let (high, low) = match reg {
            Reg16qq::BC => (self.b, self.c),
            Reg16qq::DE => (self.d, self.e),
            Reg16qq::HL => (self.h, self.l),
            Reg16qq::AF     => (self.a,     self.f.bits() as u8),
            Reg16qq::AF_ALT => (self.a_alt, self.f_alt.bits() as u8)
        };
        let val = ((high as u16) << 8 ) | low as u16;

        debug!("                Read value {:#04X} from register {:?}", val, reg);
        val
    }

    pub fn write_reg16qq(&mut self, reg: Reg16qq, val: u16) {
        let (high, low) = (((val & 0xFF00) >> 8) as u8, (val & 0x00FF) as u8);
        match reg {
            Reg16qq::BC => { self.b = high; self.c = low; }
            Reg16qq::DE => { self.d = high; self.e = low; }
            Reg16qq::HL => { self.h = high; self.l = low; }
            Reg16qq::AF =>     { self.a = high;     self.f = StatusIndicatorFlags::from_bits_truncate(low); }
            Reg16qq::AF_ALT => { self.a_alt = high; self.f_alt = StatusIndicatorFlags::from_bits_truncate(low); }
        }

        debug!("                Write value {:#06X} to register {:?}", val, reg);
    }

    pub fn inc_pc(&mut self, val: u16) { self.pc += val; }
    pub fn set_pc(&mut self, val: u16) { self.pc = val; }
    pub fn get_pc(&self) -> u16 { self.pc }

    pub fn set_ix(&mut self, val: u16) { self.ix = val; }
    pub fn get_ix(&self) -> u16 { self.ix }
    pub fn set_iy(&mut self, val: u16) { self.iy = val; }
    pub fn get_iy(&self) -> u16 { self.iy }

    pub fn set_iff1(&mut self)   { self.iff1 = true;  }
    pub fn clear_iff1(&mut self) { self.iff1 = false; }
    pub fn set_iff2(&mut self)   { self.iff2 = true;  }
    pub fn clear_iff2(&mut self) { self.iff2 = false; }

    // TODO: Properly model interrupt modes
    pub fn set_im(&mut self, val: u8) { self.im = val; }

    pub fn set_flag(&mut self, flag: StatusIndicatorFlags) { self.f.insert(flag); }
    pub fn clear_flag(&mut self, flag: StatusIndicatorFlags) { self.f.remove(flag); }
    pub fn get_flag(&self, flag: StatusIndicatorFlags) -> bool { self.f.contains(flag) }

    pub fn run_instruction(&mut self) {
        debug!("*****************************************************\n");

        let i0 = self.read_word(self.pc);
        let i1 = self.read_word(self.pc + 1);
        let i3 = self.read_word(self.pc + 3);

        match (i0, i1) {
            (0xDD, 0xCB) => {
                self.pc += 2;
                &instructions::INSTR_TABLE_DDCB [i3 as usize].execute(self);
            },
            (0xDD, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_DD   [i1 as usize].execute(self);
            },
            (0xFD, 0xCB) => {
                self.pc += 2;
                &instructions::INSTR_TABLE_FDCB [i3 as usize].execute(self);
            },
            (0xFD, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_FD   [i1 as usize].execute(self);
            },
            (0xCB, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_CB   [i1 as usize].execute(self);
            },
            (0xED, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_ED   [i1 as usize].execute(self);
            },
            (_   , _   ) => {
                &instructions::INSTR_TABLE      [i0 as usize].execute(self);
            }
        }

        debug!("*****************************************************\n");
    }

    pub fn read_word(&self, addr: u16) -> u8 {
        self.memory.read_word(addr)
    }

    pub fn write_word(&mut self, addr: u16, val: u8) {
        self.memory.write_word(addr, val);
    }

    pub fn read_port(&self, port: Port) -> u8 {
        // TODO
        match port {
            Port::MEMORY => 0x0,
            Port::AY38912_REG14 => 0x0,
            Port::AY38912_REG14_W => unreachable!(),
            Port::FE => 0x0
        }
    }

    pub fn write_port(&mut self, port: Port, val: u8) {
        // TODO
        match port {
            Port::MEMORY => {
                let bank = val & 0b00000111;
                self.memory.change_bank(bank);

                let rombank = (val & 0b00010000) >> 4;
                self.memory.change_rom_bank(rombank);

                let screen = (val & 0b00001000) >> 3;
                if screen == 1 { panic!("Unhandled screen mode"); }

                let disable = (val & 0b00100000) >> 5;
                if disable == 1 { panic!("Unhandled disabled mode"); }
            }
            Port::AY38912_REG14 => (),
            Port::AY38912_REG14_W => (),
            Port::FE => ()
        }
    }
}


