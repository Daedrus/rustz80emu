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
}
