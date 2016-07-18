use std::fmt;

pub struct Memory {
    rom: Vec<u8>,
    ram: Vec<u8>
}

impl Memory {
    pub fn new(rom: Vec<u8>, ram: Vec<u8>) -> Memory {
        Memory {
            rom: rom,
            ram: ram
        }
    }

    pub fn read_word(&self, addr: u16) -> u8 {
        if addr >= 0x0000 && addr < 0x4000 {
            self.rom[addr as usize]
        } else {
            panic!("Unrecognized address: {:#x}", addr);
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: Impl Debug for Memory")
    }
}
