extern crate z80emulib;

#[macro_use] extern crate text_io;

#[cfg(test)]
mod test_fuse {

    use z80emulib::memory::*;
    use z80emulib::cpu::*;

    use std::io::prelude::*;
    use std::fs::File;
    use std::io::{stdin, stdout};

    macro_rules! read_u8_hex {
        ($i: ident) => {{
            let val: String = read!("{}", $i);
            u8::from_str_radix(&val, 16).unwrap()
        }}
    }

    macro_rules! read_u16_hex {
        ($i: ident) => {{
            let val: String = read!("{}", $i);
            u16::from_str_radix(&val, 16).unwrap()
        }}
    }

    fn regs_setup(file: &File, cpu: &mut Cpu) -> Option<(String, u8, u64)> {
        let mut file = file.bytes().map(|ch| ch.unwrap());

        let testdesc: String = read!("{}", file);
        if testdesc.is_empty() { return None }

        cpu.write_reg16(Reg16::AF, read_u16_hex!(file));
        cpu.write_reg16(Reg16::BC, read_u16_hex!(file));
        cpu.write_reg16(Reg16::DE, read_u16_hex!(file));
        cpu.write_reg16(Reg16::HL, read_u16_hex!(file));
        cpu.write_reg16(Reg16::AF_ALT, read_u16_hex!(file));
        cpu.write_reg16(Reg16::BC_ALT, read_u16_hex!(file));
        cpu.write_reg16(Reg16::DE_ALT, read_u16_hex!(file));
        cpu.write_reg16(Reg16::HL_ALT, read_u16_hex!(file));
        cpu.write_reg16(Reg16::IX, read_u16_hex!(file));
        cpu.write_reg16(Reg16::IY, read_u16_hex!(file));
        cpu.write_reg16(Reg16::SP, read_u16_hex!(file));
        cpu.set_pc(read_u16_hex!(file));

        cpu.write_reg8(Reg8::I, read_u8_hex!(file));
        cpu.write_reg8(Reg8::R, read_u8_hex!(file));
        if read_u8_hex!(file) == 0 { cpu.clear_iff1(); } else { cpu.set_iff1(); }
        if read_u8_hex!(file) == 0 { cpu.clear_iff2(); } else { cpu.set_iff2(); }
        cpu.set_im(read_u8_hex!(file));

        let halted = read_u8_hex!(file);

        let tstate: String = read!("{}", file);
        let tstate = u64::from_str_radix(&tstate, 16).unwrap();

        Some((testdesc, halted, tstate))
    }

    fn memory_setup(file: &File, cpu: &mut Cpu) {
        let mut file = file.bytes().map(|ch| ch.unwrap());

        loop {
            let addr: String = read!("{}", file);
            if addr == "-1" { break; }

            let addr = u16::from_str_radix(&addr, 16).unwrap();
            let mut idx = 0;

            loop {
                let memval: String = read!("{}", file);
                if memval == "-1" { break; }

                let memval = u8::from_str_radix(&memval, 16).unwrap();

                cpu.zero_cycle_write_word(addr + idx, memval);

                idx = idx + 1;
            }
        }

        let emptyline: String = read!("{}", file);
    }

    //#[test]
    fn test_fuse() {
        let file = File::open("tests/tests.in").unwrap();

        let dummyrom0: Vec<u8> = vec![0; 16 * 1024];
        let dummyrom1: Vec<u8> = vec![0; 16 * 1024];

        let memory = MemoryBuilder::new()
                        .rom0(dummyrom0)
                        .rom1(dummyrom1)
                        .writable_rom(true)
                        .finalize();

        let mut cpu = Cpu::new(memory);

        loop {
            let mut tcycle_lim: u64 = 0;

            match regs_setup(&file, &mut cpu) {
                Some((testname, _, tcycles)) => {
                    println!("{}", testname);
                    tcycle_lim = tcycles;
                },

                None => {
                    break
                }
            }
            memory_setup(&file, &mut cpu);

            loop {
                cpu.run_instruction();
                if cpu.tcycles > tcycle_lim { break }
            }

            print!("{:04x} {:04x} {:04x} {:04x} ",
                   cpu.read_reg16(Reg16::AF), cpu.read_reg16(Reg16::BC),
                   cpu.read_reg16(Reg16::DE), cpu.read_reg16(Reg16::HL));
            print!("{:04x} {:04x} {:04x} {:04x} ",
                   cpu.read_reg16(Reg16::AF_ALT), cpu.read_reg16(Reg16::BC_ALT),
                   cpu.read_reg16(Reg16::DE_ALT), cpu.read_reg16(Reg16::HL_ALT));
            println!("{:04x} {:04x} {:04x} {:04x}",
                   cpu.read_reg16(Reg16::IX), cpu.read_reg16(Reg16::IY),
                   cpu.read_reg16(Reg16::SP), cpu.get_pc());
            print!("{:02x} {:02x} {} {} ",
                   cpu.read_reg8(Reg8::I), cpu.read_reg8(Reg8::R),
                   if cpu.get_iff1() { 1 } else { 0 },
                   if cpu.get_iff2() { 1 } else { 0 });
            println!("{} 0 {}\n",
                   cpu.get_im(), cpu.tcycles);

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            let input: String = input.trim().into();

            cpu.reset();
        }
    }
}
