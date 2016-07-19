use super::memory;
use num::FromPrimitive;
use std::fmt;

enum_from_primitive! {
#[derive(Debug)]
enum RegPair {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11
}
}

#[derive(Debug)]
pub struct Cpu {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    af_: u16,
    bc_: u16,
    de_: u16,
    hl_: u16,
    ir: u16,
    ix: u16,
    iy: u16,
    sp: u16,
    pc: u16,

    iff1: bool,
    iff2: bool,

    memory: memory::Memory
}

impl Cpu {
    pub fn new(memory: memory::Memory) -> Cpu {
        Cpu {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            af_: 0,
            bc_: 0,
            de_: 0,
            hl_: 0,
            ir: 0,
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

    // TODO have a single function called write_reg that handles multiple types
    fn write_regpair(&mut self, reg: RegPair, val: u16) {
        match reg {
            RegPair::BC => { self.bc = val; }
            RegPair::DE => { self.de = val; }
            RegPair::HL => { self.hl = val; }
            RegPair::SP => { self.sp = val; }
        }
    }

    fn write_reg(&mut self, reg: Reg, val: u8) {
        match reg {
            Reg::A => { self.af |= (val as u16) << 8 }
            Reg::B => { self.bc |= (val as u16) << 8 }
            Reg::C => { self.bc |=  val as u16       }
            Reg::D => { self.de |= (val as u16) << 8 }
            Reg::E => { self.de |=  val as u16       }
            Reg::H => { self.hl |= (val as u16) << 8 }
            Reg::L => { self.hl |=  val as u16       }

        }
    }

    // TODO have a single function called read_reg that handles multiple types
    fn read_regpair(&mut self, reg: RegPair) -> u16 {
        match reg {
            RegPair::BC => { self.bc }
            RegPair::DE => { self.de }
            RegPair::HL => { self.hl }
            RegPair::SP => { self.sp }
        }
    }

    fn read_reg(&mut self, reg: Reg) -> u8 {
        match reg {
            Reg::A => { ((self.af & 0xFF00) >> 8) as u8 }
            Reg::B => { ((self.bc & 0xFF00) >> 8) as u8 }
            Reg::C => { ((self.bc & 0x00FF)     ) as u8 }
            Reg::D => { ((self.de & 0xFF00) >> 8) as u8 }
            Reg::E => { ((self.de & 0x00FF)     ) as u8 }
            Reg::H => { ((self.hl & 0xFF00) >> 8) as u8 }
            Reg::L => { ((self.hl & 0x00FF)     ) as u8 }
        }
    }

    fn run_instruction(&mut self) {
        let instruction = self.read_word(self.pc);

        match instruction {
            0b11110011 => {
                self.iff1 = false;
                self.iff2 = false;
                println!("{:#x}: DI", self.pc);
                self.pc += 1;
            },
            0b00000001 | 0b00010001 | 0b00100001 | 0b00110001 => {
                let nn = (self.read_word(self.pc + 1) as u16) +
                        ((self.read_word(self.pc + 2) as u16) << 8);
                let regpair = RegPair::from_u8((instruction & 0b00110000) >> 4);
                match regpair {
                    Some(RegPair::BC) => { self.bc = nn; }
                    Some(RegPair::DE) => { self.de = nn; }
                    Some(RegPair::HL) => { self.hl = nn; }
                    Some(RegPair::SP) => { self.sp = nn; }
                    _ => {
                        panic!("Error when parsing \"LD dd, nn\" instruction");
                    }
                }
                println!("{:#x}: LD {:?}, ${:x}", self.pc, regpair.unwrap(), nn);
                self.pc += 3;
            },
            0b00001011 | 0b00011011 | 0b00101011 | 0b00111011 => {
                let regpair = RegPair::from_u8((instruction & 0b00110000) >> 4);
                match regpair {
                    Some(RegPair::BC) => { self.bc -= 1; }
                    Some(RegPair::DE) => { self.de -= 1; }
                    Some(RegPair::HL) => { self.hl -= 1; }
                    Some(RegPair::SP) => { self.sp -= 1; }
                    _ => {
                        panic!("Error when parsing \"DEC ss\" instruction");
                    }
                }
                println!("{:#x}: DEC {:?}", self.pc, regpair.unwrap());
                self.pc += 1;
            }
            _ => {
                panic!("Unrecognized instruction: {:#x}", instruction);
            }
        }
    }

    fn read_word(&self, addr: u16) -> u8 {
        self.memory.read_word(addr)
    }
}

