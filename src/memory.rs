use std::fmt;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Memory {
    rom: u8,
    ram_0x4000_0x7fff: usize,
    ram_0x8000_0xbfff: usize,
    ram_0xc000_0xffff: usize,

    rom0: Vec<u8>,
    rom1: Vec<u8>,
    bank: [Vec<u8>; 8],

    writable_rom: bool,
}

impl Memory {
    pub fn read_word(&self, addr: u16) -> u8 {
        let val = match addr {
            0x0000...0x3FFF => {
                match self.rom {
                    0 => self.rom0[addr as usize],
                    1 => self.rom1[addr as usize],
                    _ => unreachable!(),
                }
            }
            0x4000...0x7FFF => self.bank[self.ram_0x4000_0x7fff][(addr - 0x4000) as usize],
            0x8000...0xBFFF => self.bank[self.ram_0x8000_0xbfff][(addr - 0x8000) as usize],
            0xC000...0xFFFF => self.bank[self.ram_0xc000_0xffff][(addr - 0xC000) as usize],
            _ => unreachable!(),
        };

        // debug!("                Read value {:#04X} from address {:#06X}", val, addr);
        val
    }

    pub fn write_word(&mut self, addr: u16, val: u8) {
        if addr >= 0xC000 {
            self.bank[self.ram_0xc000_0xffff][(addr - 0xC000) as usize] = val;
        } else if addr >= 0x8000 && addr <= 0xBFFF {
            self.bank[self.ram_0x8000_0xbfff][(addr - 0x8000) as usize] = val;
        } else if addr >= 0x4000 && addr <= 0x7FFF {
            self.bank[self.ram_0x4000_0x7fff][(addr - 0x4000) as usize] = val;
        } else {
            if self.writable_rom {
                match self.rom {
                    0 => {
                        self.rom0[addr as usize] = val;
                    }
                    1 => {
                        self.rom1[addr as usize] = val;
                    }
                    _ => unreachable!(),
                }
            } else {
                panic!("Trying to write to unrecognized address: {:#x}", addr);
            }
        }

        // debug!("                Write value {:#04X} to address {:#06X}", val, addr);
    }

    pub fn change_bank(&mut self, val: u8) {
        self.ram_0xc000_0xffff = val as usize;
    }

    pub fn change_rom_bank(&mut self, val: u8) {
        self.rom = val;
    }

    pub fn clear(&mut self) {
        if self.writable_rom {
            for x in self.rom0.iter_mut() {
                *x = 0;
            }
            for x in self.rom1.iter_mut() {
                *x = 0;
            }
        }
        for bank in self.bank.iter_mut() {
            for x in bank.iter_mut() {
                *x = 0;
            }
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,
                 "ram_0xc000_0xffff: {}
                  ram_0x8000_0xbfff: {}
                  ram_0x4000_0x7fff: {}
                  rom: {} ",
                 self.ram_0xc000_0xffff,
                 self.ram_0x8000_0xbfff,
                 self.ram_0x4000_0x7fff,
                 self.rom)
    }
}

pub struct MemoryBuilder {
    rom: u8,
    ram_0x4000_0x7fff: usize,
    ram_0x8000_0xbfff: usize,
    ram_0xc000_0xffff: usize,

    rom0: Vec<u8>,
    rom1: Vec<u8>,
    bank: [Vec<u8>; 8],

    writable_rom: bool,
}

impl MemoryBuilder {
    pub fn new() -> MemoryBuilder {
        MemoryBuilder {
            rom: 0,
            ram_0x4000_0x7fff: 5,
            ram_0x8000_0xbfff: 2,
            ram_0xc000_0xffff: 0,

            rom0: vec![0; 16 * 1024],
            rom1: vec![0; 16 * 1024],

            bank: [vec![0; 16 * 1024],
                   vec![0; 16 * 1024],
                   vec![0; 16 * 1024],
                   vec![0; 16 * 1024],
                   vec![0; 16 * 1024],
                   vec![0; 16 * 1024],
                   vec![0; 16 * 1024],
                   vec![0; 16 * 1024]],

            writable_rom: false,
        }
    }

    pub fn rom0(mut self, mem: Vec<u8>) -> MemoryBuilder {
        self.rom0 = mem;
        self
    }

    pub fn rom1(mut self, mem: Vec<u8>) -> MemoryBuilder {
        self.rom1 = mem;
        self
    }

    pub fn writable_rom(mut self, val: bool) -> MemoryBuilder {
        self.writable_rom = val;
        self
    }

    pub fn finalize(self) -> Memory {
        Memory {
            rom: self.rom,
            ram_0x4000_0x7fff: self.ram_0x4000_0x7fff,
            ram_0x8000_0xbfff: self.ram_0x8000_0xbfff,
            ram_0xc000_0xffff: self.ram_0xc000_0xffff,

            rom0: self.rom0,
            rom1: self.rom1,
            bank: self.bank,

            writable_rom: self.writable_rom,
        }
    }
}
