use super::memory;
use num::FromPrimitive;
use std::fmt;

enum_from_primitive! {
enum RegPair {
    RegPairBC = 0b00,
    RegPairDE = 0b01,
    RegPairHL = 0b10,
    RegPairSP = 0b11
}
}

impl fmt::Debug for RegPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RegPair::RegPairBC => {write!(f, "BC")}
            &RegPair::RegPairDE => {write!(f, "DE")}
            &RegPair::RegPairHL => {write!(f, "HL")}
            &RegPair::RegPairSP => {write!(f, "SP")}
        }
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
                let nn =
                    (self.read_word(self.pc + 1) as u16) +
                    ((self.read_word(self.pc + 2) as u16) << 8);
                let regpair = RegPair::from_u8((instruction & 0b00110000) >> 4);
                match regpair {
                    Some(RegPair::RegPairBC) => {
                        self.bc = nn;
                    }
                    Some(RegPair::RegPairDE) => {
                        self.de = nn;
                    }
                    Some(RegPair::RegPairHL) => {
                        self.hl = nn;
                    }
                    Some(RegPair::RegPairSP) => {
                        self.sp = nn;
                    }
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
                    Some(RegPair::RegPairBC) => {
                        self.bc -= 1;
                    }
                    Some(RegPair::RegPairDE) => {
                        self.de -= 1;
                    }
                    Some(RegPair::RegPairHL) => {
                        self.hl -= 1;
                    }
                    Some(RegPair::RegPairSP) => {
                        self.sp -= 1;
                    }
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

