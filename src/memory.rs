use std::fmt;

pub struct Memory {
    rom: u8,
    ram_0x4000_0x7FFF: usize,
    ram_0x8000_0xBFFF: usize,
    ram_0xC000_0xFFFF: usize,

    rom0: Vec<u8>,
    rom1: Vec<u8>,
    bank: [Vec<u8>; 8],

    writable_rom: bool
}

impl Memory {
    pub fn read_word(&self, addr: u16) -> u8 {
        let mut val = 0;
        if addr <= 0x3FFF {
            match self.rom {
                0 => { val = self.rom0[addr as usize]; },
                1 => { val = self.rom1[addr as usize]; },
                _ => unreachable!()
            }
        } else if addr >= 0x4000 && addr <= 0x7FFF {
            val = self.bank[self.ram_0x4000_0x7FFF][(addr - 0x4000) as usize];
        } else if addr >= 0x8000 && addr <= 0xBFFF {
            val = self.bank[self.ram_0x8000_0xBFFF][(addr - 0x8000) as usize];
        } else if addr >= 0xC000 {
            val = self.bank[self.ram_0xC000_0xFFFF][(addr - 0xC000) as usize];
        } else {
            panic!("Trying to read from unrecognized address: {:#x}", addr);
        }

        debug!("                Read value {:#04X} from address {:#06X}", val, addr);
        val
    }

    pub fn write_word(&mut self, addr: u16, val: u8) {
        if addr >= 0xC000 {
            self.bank[self.ram_0xC000_0xFFFF][(addr - 0xC000) as usize] = val;
        } else if addr >= 0x8000 && addr <= 0xBFFF {
            self.bank[self.ram_0x8000_0xBFFF][(addr - 0x8000) as usize] = val;
        } else if addr >= 0x4000 && addr <= 0x7FFF {
            self.bank[self.ram_0x4000_0x7FFF][(addr - 0x4000) as usize] = val;
        } else {
            if self.writable_rom {
                match self.rom {
                    0 => { self.rom0[addr as usize] = val; },
                    1 => { self.rom1[addr as usize] = val; },
                    _ => unreachable!()
                }
            } else {
                panic!("Trying to write to unrecognized address: {:#x}", addr);
            }
        }

        debug!("                Write value {:#04X} to address {:#06X}", val, addr);
    }

    pub fn change_bank(&mut self, val: u8) {
        self.ram_0xC000_0xFFFF = val as usize;
    }

    pub fn change_rom_bank(&mut self, val: u8) {
        self.rom = val;
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "ram_0xC000_0xFFFF: {}
                     ram_0x8000_0xBFFF: {}
                     ram_0x4000_0x7FFF: {}
                     rom: {}
                    ",
                    self.ram_0xC000_0xFFFF,
                    self.ram_0x8000_0xBFFF,
                    self.ram_0x4000_0x7FFF,
                    self.rom)
    }
}

pub struct MemoryBuilder {
    rom: u8,
    ram_0x4000_0x7FFF: usize,
    ram_0x8000_0xBFFF: usize,
    ram_0xC000_0xFFFF: usize,

    rom0: Vec<u8>,
    rom1: Vec<u8>,
    bank: [Vec<u8>; 8],

    writable_rom: bool
}

impl MemoryBuilder {
    pub fn new() -> MemoryBuilder {
        MemoryBuilder {
            rom: 0,
            ram_0x4000_0x7FFF: 5,
            ram_0x8000_0xBFFF: 2,
            ram_0xC000_0xFFFF: 0,

            rom0: vec![0; 16 * 1024],
            rom1: vec![0; 16 * 1024],

            bank: [
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024],
                vec![0; 16 * 1024]
            ],

            writable_rom: false
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
            ram_0x4000_0x7FFF: self.ram_0x4000_0x7FFF,
            ram_0x8000_0xBFFF: self.ram_0x8000_0xBFFF,
            ram_0xC000_0xFFFF: self.ram_0xC000_0xFFFF,

            rom0: self.rom0,
            rom1: self.rom1,
            bank: self.bank,

            writable_rom: self.writable_rom
        }
    }
}
