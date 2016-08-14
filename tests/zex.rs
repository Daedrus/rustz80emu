extern crate z80emulib;

#[macro_use] extern crate log;
extern crate env_logger;

#[cfg(test)]
mod test_zex {

    use z80emulib::memory::*;
    use z80emulib::cpu::*;
    use z80emulib::instructions::{self};

    use log::{LogRecord};
    use env_logger::LogBuilder;
    use std::env;


    static ZEXDOC: &'static [u8] = include_bytes!("zexdoc.com");

    // TODO: Reuse the function from main.rs
    fn setup_logging() {
        let mut builder = LogBuilder::new();

        let format = |record: &LogRecord| { format!("{}", record.args()) };
        builder.format(format);

        if env::var("RUST_LOG").is_ok() {
            builder.parse(&env::var("RUST_LOG").unwrap());
        }

        builder.init().unwrap();
    }

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

        // Manually call RET
        &instructions::INSTR_TABLE[0xC9].execute(cpu);
    }

    #[test]
    fn test_zexdoc() {
        setup_logging();

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

        loop {
            cpu.run_instruction();
            match cpu.get_pc() {
                0x0005 => { cpm_bdos(&mut cpu); }
                0x0000 => { break; }
                _      => { }
            }
        }
    }
}
