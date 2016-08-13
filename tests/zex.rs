extern crate z80emulib;

#[cfg(test)]
mod test_zex {
    use z80emulib::memory::*;
    use z80emulib::cpu::*;

    static ZEXDOC: &'static [u8] = include_bytes!("zexdoc.com");

    #[test]
    fn test_zexdoc() {
        let mut dummyrom0: Vec<u8> = vec![0; 16 * 1024];
        let dummyrom1: Vec<u8> = vec![0; 16 * 1024];

        for (i, byte) in ZEXDOC.iter().enumerate() {
            dummyrom0[i + 0x100] = *byte;
        }
        // The ZEXDOC test seems to get its SP from address 0x0006
        dummyrom0[0x0006] = 0x00;
        dummyrom0[0x0007] = 0xF0;

        let memory = Memory::new(dummyrom0, dummyrom1);

        let mut cpu = Cpu::new(memory);
        cpu.set_pc(0x0100);

        cpu.run();
    }
}
