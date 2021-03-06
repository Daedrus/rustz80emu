extern crate z80emulib;

#[cfg(test)]
mod test_zex {

    use z80emulib::cpu::*;
    use z80emulib::peripherals::*;
    use z80emulib::interconnect::*;

    use std::io::{stdout, Write};

    use std::rc::Rc;
    use std::cell::RefCell;


    static ZEXDOC: &'static [u8] = include_bytes!("zexdoc.com");
    static ZEXALL: &'static [u8] = include_bytes!("zexall.com");

    fn cpm_bdos(cpu: &mut Cpu) {
        match cpu.read_reg8(Reg8::C) {
            2 => {
                print!("{}", cpu.read_reg8(Reg8::E) as u8 as char);
            },
            9 => {
                let mut addr = cpu.read_reg16(Reg16::DE);
                loop {
                    let c = cpu.read_word(addr);
                    addr = addr.wrapping_add(1);
                    if c != b'$' { print!("{}", c as char) } else { break; }
                }
            },
            _ => unreachable!()
        }

        stdout().flush().unwrap();

        // Manually call RET
        &INSTR_TABLE[0xC9].execute(cpu);
    }

    fn test_rom(rom: &[u8]) {
        let mut dummyrom0 = vec![0; 16 * 1024].into_boxed_slice();

        for (i, byte) in rom.iter().enumerate() {
            dummyrom0[i + 0x100] = *byte;
        }

        let memory = Rc::new(RefCell::new(MemoryBuilder::new()
                        .rom0(dummyrom0)
                        .writable_rom(true)
                        .finalize()));
        let ay = Rc::new(RefCell::new(Ay::new()));
        let ula = Rc::new(RefCell::new(Ula::new(memory.clone())));

        let interconnect = Interconnect::new(
            memory.clone(),
            ay.clone(),
            ula.clone());

        let mut cpu = Cpu::new(interconnect);
        cpu.set_pc(0x0100);

        loop {
            cpu.run_instruction();
            match cpu.get_pc() {
                0x0005 => { cpm_bdos(&mut cpu); }
                0x0000 => { break; }
                _      => { }
            }
        }
    }

    #[test]
    fn test_zex() {
        test_rom(&ZEXDOC);
        test_rom(&ZEXALL);
    }
}
