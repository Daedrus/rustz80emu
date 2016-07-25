use std::fmt;

pub struct Memory {
    rom: u8,
    ram_0x4000_0x7FFF: usize,
    ram_0x8000_0xBFFF: usize,
    ram_0xC000_0xFFFF: usize,

    rom0: Vec<u8>,
    rom1: Vec<u8>,
    bank: [Vec<u8>; 8],
}

impl Memory {
    pub fn new(_rom0: Vec<u8>, _rom1: Vec<u8>) -> Memory {
        Memory {
            rom: 0,
            ram_0x4000_0x7FFF: 5,
            ram_0x8000_0xBFFF: 2,
            ram_0xC000_0xFFFF: 0,

            rom0: _rom0,
            rom1: _rom1,

            bank: [
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024]
            ]
        }
    }

    pub fn read_word(&self, addr: u16) -> u8 {
        if addr >= 0x0000 && addr < 0x4000 {
            match self.rom {
                0 => self.rom0[addr as usize],
                1 => self.rom1[addr as usize],
                _ => unreachable!()
            }
        } else if addr >= 0xC000 && addr <= 0xFFFF {
            self.bank[self.ram_0xC000_0xFFFF][(addr - 0xC000) as usize]
        } else {
            panic!("Trying to read from unrecognized address: {:#x}", addr);
        }
    }

    pub fn write_word(&mut self, addr: u16, val: u8) {
        if addr >= 0xC000 && addr <= 0xFFFF {
            self.bank[self.ram_0xC000_0xFFFF][(addr - 0xC000) as usize] = val;
        } else {
            panic!("Trying to write to unrecognized address: {:#x}", addr);
        }
    }

    pub fn change_bank(&mut self, val: u8) {
        self.ram_0xC000_0xFFFF = val as usize;
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: Impl Debug for Memory")
    }
}
