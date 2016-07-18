use super::memory;

const REG_PAIR_BC:u8 = 0b00;
const REG_PAIR_DE:u8 = 0b01;
const REG_PAIR_HL:u8 = 0b10;
const REG_PAIR_SP:u8 = 0b11;

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
                match (instruction & 0b00110000) >> 4 {
                    REG_PAIR_BC => {
                        self.bc = nn;
                        println!("{:#x}: LD BC, ${:x}", self.pc, nn);
                    }
                    REG_PAIR_DE => {
                        self.de = nn;
                        println!("{:#x}: LD DE, ${:x}", self.pc, nn);
                    }
                    REG_PAIR_HL => {
                        self.hl = nn;
                        println!("{:#x}: LD HL, ${:x}", self.pc, nn);
                    }
                    REG_PAIR_SP => {
                        self.sp = nn;
                        println!("{:#x}: LD SP, ${:x}", self.pc, nn);
                    }
                    _ => {
                        panic!("Error when parsing \"LD dd, nn\" instruction");
                    }
                }
                self.pc += 3;
            },
            0b00001011 | 0b00011011 | 0b00101011 | 0b00111011 => {
                match (instruction & 0b00110000) >> 4 {
                    REG_PAIR_BC => {
                        self.bc -= 1;
                        println!("{:#x}: DEC BC", self.pc);
                    }
                    REG_PAIR_DE => {
                        self.de -= 1;
                        println!("{:#x}: DEC DE", self.pc);
                    }
                    REG_PAIR_HL => {
                        self.hl -= 1;
                        println!("{:#x}: DEC HL", self.pc);
                    }
                    REG_PAIR_SP => {
                        self.sp -= 1;
                        println!("{:#x}: DEC SP", self.pc);
                    }
                    _ => {
                        panic!("Error when parsing \"DEC ss\" instruction");
                    }
                }
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

