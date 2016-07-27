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
        writeln!(f, "
                                               ------------
                         ------                | SZ_H_PNC |
                     a:  | {:02X} |             f: | {:08b} |
                         ------                ------------

                         -----------           -----------
                     bc: | {:02X} | {:02X} |   bc_alt: | {:02X} | {:02X} |
                     de: | {:02X} | {:02X} |   de_alt: | {:02X} | {:02X} |
                     hl: | {:02X} | {:02X} |   hl_alt: | {:02X} | {:02X} |
                         -----------           -----------

                         -----------
                     ir: | {:02X} | {:02X} |
                         -----------

                         ----------
                     ix: |  {:04X}  |
                     iy: |  {:04X}  |
                     sp: |  {:04X}  |
                     pc: |  {:04X}  |
                         ----------

                     {:?}",
                      self.a,
                      self.f.bits(),
                      self.b, self.c, self.b_alt, self.c_alt,
                      self.d, self.e, self.d_alt, self.e_alt,
                      self.h, self.l, self.h_alt, self.l_alt,
                      self.i, self.r,
                      self.ix,
                      self.iy,
                      self.sp,
                      self.pc,
                      self.memory)
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
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l
        }
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
    }

    pub fn read_reg16(&self, reg: Reg16) -> u16 {
        let value = match reg {
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
        value
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
    }

    pub fn read_reg16qq(&self, reg: Reg16qq) -> u16 {
        let (high, low) = match reg {
            Reg16qq::BC => (self.b, self.c),
            Reg16qq::DE => (self.d, self.e),
            Reg16qq::HL => (self.h, self.l),
            Reg16qq::AF => (self.a, self.f.bits() as u8)
        };
        ((high as u16) << 8 ) | low as u16
    }

    pub fn write_reg16qq(&mut self, reg: Reg16qq, val: u16) {
        let (high, low) = (((val & 0xFF00) >> 8) as u8, (val & 0x00FF) as u8);
        match reg {
            Reg16qq::BC => { self.b = high; self.c = low; }
            Reg16qq::DE => { self.d = high; self.e = low; }
            Reg16qq::HL => { self.h = high; self.l = low; }
            Reg16qq::AF => { self.a = high; self.f = StatusIndicatorFlags::from_bits_truncate(low); }
        }
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

    fn run_instruction(&mut self) {
        let instruction = self.read_word(self.pc);

        let instrs = &instructions::INSTR_TABLE[instruction as usize];

        instrs.execute(self);
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
            Port::MEMORY => 0x42,
            Port::AY38912_REG14 => 0x42,
            Port::AY38912_REG14_W => unreachable!(),
            Port::FE => 0x42
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


