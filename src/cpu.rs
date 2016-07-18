use super::memory;

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
                println!("{:#x}: DI", self.pc);
                self.iff1 = false;
                self.iff1 = false;
                self.pc += 1;
            },
            _ => {
                panic!("Unrecognized instruction: {:#x}", instruction);
            }
        }

    }

    fn read_word(&self, addr: u16) -> u8 {
        self.memory.read_word(addr)
    }
}

