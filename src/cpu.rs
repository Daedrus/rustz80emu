use super::memory;
use num::FromPrimitive;
use std::fmt;

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
enum Reg16 {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
enum Reg8 {
    A = 0b111,
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101
}
}

pub struct Cpu {
    // main register set
    a: u8, f: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,

    // alternate register set
    a_alt: u8, f_alt: u8,
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

    memory: memory::Memory

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "
                         -----------
                     af: | {:02X} | {:02X} |
                     bc: | {:02X} | {:02X} |
                     de: | {:02X} | {:02X} |
                     hl: | {:02X} | {:02X} |
                         -----------
                     ir  | {:02X} | {:02X} |
                         -----------
                     ix  |   {:04X}  |
                     iy  |   {:04X}  |
                     sp  |   {:04X}  |
                     pc  |   {:04X}  |
                         -----------",
                      self.a, self.f,
                      self.b, self.c,
                      self.d, self.e,
                      self.h, self.l,
                      self.i, self.r,
                      self.ix,
                      self.iy,
                      self.sp,
                      self.pc)
    }
}

impl Cpu {
    pub fn new(memory: memory::Memory) -> Cpu {
        Cpu {
            a: 0, f: 0,
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            a_alt: 0, f_alt: 0,
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

            memory: memory
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_instruction();
        }
    }

    fn read_reg8(&self, reg: Reg8) -> u8 {
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

    fn write_reg8(&mut self, reg: Reg8, val: u8) {
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

    fn read_reg16(&self, reg: Reg16) -> u16 {
        let value = match reg {
            Reg16::SP => self.sp,
            _ => {
                let (high, low) = match reg {
                    Reg16::BC => (self.b, self.c),
                    Reg16::DE => (self.d, self.e),
                    Reg16::HL => (self.h, self.l),
                    _ => unreachable!()
                };
                (((high as u16) << 8 ) + low as u16)
            }
        };
        value
    }

    fn write_reg16(&mut self, reg: Reg16, val: u16) {
        let (high, low) = (((val & 0xFF00) >> 8) as u8, (val & 0x00FF) as u8);
        match reg {
            Reg16::BC => { self.b = high; self.c = low; }
            Reg16::DE => { self.d = high; self.e = low; }
            Reg16::HL => { self.h = high; self.l = low; }
            Reg16::SP => { self.sp = val }
        }
    }

    fn run_instruction(&mut self) {
        let instruction = self.read_word(self.pc);

        match instruction {
            instruction if instruction & 0b11000000 == 0b01000000 => {
                // TODO Better Option handling here
                let rt = Reg8::from_u8((instruction >> 3) & 0b111).unwrap();
                let rs = Reg8::from_u8( instruction       & 0b111).unwrap();
                let rsval = self.read_reg8(rs);
                self.write_reg8(rt, rsval);
                println!("{:#x}: LD {:?}, {:?}", self.pc, rt, rs);
                self.pc += 1;
            },

            0b11110011 => {
                self.iff1 = false;
                self.iff2 = false;
                println!("{:#x}: DI", self.pc);
                self.pc += 1;
            },

            0b00000001 | 0b00010001 | 0b00100001 | 0b00110001 => {
                let nn = (self.read_word(self.pc + 1) as u16) +
                        ((self.read_word(self.pc + 2) as u16) << 8);
                let regpair = Reg16::from_u8((instruction & 0b00110000) >> 4).unwrap();
                self.write_reg16(regpair, nn);

                println!("{:#x}: LD {:?}, ${:x}", self.pc, regpair, nn);
                self.pc += 3;
            },
            0b00001011 | 0b00011011 | 0b00101011 | 0b00111011 => {
                let regpair = Reg16::from_u8((instruction & 0b00110000) >> 4).unwrap();
                let oldregval = self.read_reg16(regpair);
                self.write_reg16(regpair, oldregval - 1);

                println!("{:#x}: DEC {:?}", self.pc, regpair);
                self.pc += 1;
            },
            _ => {
                panic!("Unrecognized instruction: {:#x}", instruction);
            }
        }
        println!("{:?}", self);
    }

    fn read_word(&self, addr: u16) -> u8 {
        self.memory.read_word(addr)
    }
}

