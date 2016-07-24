use super::cpu::*;

pub trait Instruction {
    fn execute(&self, &mut Cpu);
}

struct Instruction_UNSUPPORTED;

impl Instruction for Instruction_UNSUPPORTED {
    fn execute(&self, cpu: &mut Cpu) {
        panic!("Unsupported instruction {:#x}", cpu.read_word(cpu.get_pc()));
    }
}

struct Instruction_DEC {
    regpair: Reg16
}

impl Instruction for Instruction_DEC {
    fn execute(&self, cpu: &mut Cpu) {
        let oldregval = cpu.read_reg16(self.regpair);
        cpu.write_reg16(self.regpair, oldregval - 1);

        println!("{:#x}: DEC {:?}", cpu.get_pc(), self.regpair);
        cpu.inc_pc(1);
    }
}

struct Instruction_LD_NN {
    regpair: Reg16
}

impl Instruction for Instruction_LD_NN {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        cpu.write_reg16(self.regpair, nn);

        println!("{:#x}: LD {:?}, ${:x}", cpu.get_pc(), self.regpair, nn);
        cpu.inc_pc(3);
    }
}

struct Instruction_LD_R_R {
    rt: Reg8,
    rs: Reg8
}

impl Instruction for Instruction_LD_R_R {
    fn execute(&self, cpu: &mut Cpu) {
        let rsval = cpu.read_reg8(self.rs);
        cpu.write_reg8(self.rt, rsval);
        println!("{:#x}: LD {:?}, {:?}", cpu.get_pc(), self.rt, self.rs);
        cpu.inc_pc(1);
    }
}

struct Instruction_OR_R {
    r: Reg8
}

impl Instruction for Instruction_OR_R {
    fn execute(&self, cpu: &mut Cpu) {
        let orval = cpu.read_reg8(self.r) | cpu.read_reg8(Reg8::A);
        cpu.write_reg8(Reg8::A, orval);
        if orval.count_ones() % 2 == 0 {
            cpu.set_flag(PARITY_OVERFLOW_FLAG);
        } else {
            cpu.clear_flag(PARITY_OVERFLOW_FLAG);
        }
        if orval == 0 {
            cpu.set_flag(ZERO_FLAG);
        } else {
            cpu.clear_flag(ZERO_FLAG);
        }
        if orval < 0 {
            cpu.set_flag(SIGN_FLAG);
        } else {
            cpu.clear_flag(SIGN_FLAG);
        }
        println!("{:#x}: OR {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

struct Instruction_DI;

impl Instruction for Instruction_DI {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.clear_iff1();
        cpu.clear_iff2();
        println!("{:#x}: DI", cpu.get_pc());
        cpu.inc_pc(1);
    }
}

struct Instruction_JR_NZ;

impl Instruction for Instruction_JR_NZ {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (curr_pc as i16 + offset as i16) as u16;
        if cpu.get_flag(ZERO_FLAG) {
            cpu.inc_pc(2);
        } else {
            cpu.set_pc(target);
        }
        println!("{:#x}: JR NZ {:#06X}", cpu.get_pc(), target);
    }
}

struct Instruction_JP_NN;

impl Instruction for Instruction_JP_NN {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        cpu.set_pc(nn);
    }
}

pub const INSTR_TABLE: [&'static Instruction; 256] = [
    &Instruction_UNSUPPORTED, /* 0b00000000 */
    &Instruction_LD_NN {      /* 0b00000001 */
        regpair: Reg16::BC
    },
    &Instruction_UNSUPPORTED, /* 0b00000010 */
    &Instruction_UNSUPPORTED, /* 0b00000011 */
    &Instruction_UNSUPPORTED, /* 0b00000100 */
    &Instruction_UNSUPPORTED, /* 0b00000101 */
    &Instruction_UNSUPPORTED, /* 0b00000110 */
    &Instruction_UNSUPPORTED, /* 0b00000111 */
    &Instruction_UNSUPPORTED, /* 0b00001000 */
    &Instruction_UNSUPPORTED, /* 0b00001001 */
    &Instruction_UNSUPPORTED, /* 0b00001010 */
    &Instruction_DEC {        /* 0b00001011 */
        regpair: Reg16::BC
    },
    &Instruction_UNSUPPORTED, /* 0b00001100 */
    &Instruction_UNSUPPORTED, /* 0b00001101 */
    &Instruction_UNSUPPORTED, /* 0b00001110 */
    &Instruction_UNSUPPORTED, /* 0b00001111 */
    &Instruction_UNSUPPORTED, /* 0b00010000 */
    &Instruction_LD_NN {      /* 0b00010001 */
        regpair: Reg16::DE
    },
    &Instruction_UNSUPPORTED, /* 0b00010010 */
    &Instruction_UNSUPPORTED, /* 0b00010011 */
    &Instruction_UNSUPPORTED, /* 0b00010100 */
    &Instruction_UNSUPPORTED, /* 0b00010101 */
    &Instruction_UNSUPPORTED, /* 0b00010110 */
    &Instruction_UNSUPPORTED, /* 0b00010111 */
    &Instruction_UNSUPPORTED, /* 0b00011000 */
    &Instruction_UNSUPPORTED, /* 0b00011001 */
    &Instruction_UNSUPPORTED, /* 0b00011010 */
    &Instruction_DEC {        /* 0b00011011 */
        regpair: Reg16::DE
    },
    &Instruction_UNSUPPORTED, /* 0b00011100 */
    &Instruction_UNSUPPORTED, /* 0b00011101 */
    &Instruction_UNSUPPORTED, /* 0b00011110 */
    &Instruction_UNSUPPORTED, /* 0b00011111 */
    &Instruction_JR_NZ      , /* 0b00100000 */
    &Instruction_LD_NN {      /* 0b00100001 */
        regpair: Reg16::HL
    },
    &Instruction_UNSUPPORTED, /* 0b00100010 */
    &Instruction_UNSUPPORTED, /* 0b00100011 */
    &Instruction_UNSUPPORTED, /* 0b00100100 */
    &Instruction_UNSUPPORTED, /* 0b00100101 */
    &Instruction_UNSUPPORTED, /* 0b00100110 */
    &Instruction_UNSUPPORTED, /* 0b00100111 */
    &Instruction_UNSUPPORTED, /* 0b00101000 */
    &Instruction_UNSUPPORTED, /* 0b00101001 */
    &Instruction_UNSUPPORTED, /* 0b00101010 */
    &Instruction_DEC {        /* 0b00101011 */
        regpair: Reg16::HL
    },
    &Instruction_UNSUPPORTED, /* 0b00101100 */
    &Instruction_UNSUPPORTED, /* 0b00101101 */
    &Instruction_UNSUPPORTED, /* 0b00101110 */
    &Instruction_UNSUPPORTED, /* 0b00101111 */
    &Instruction_UNSUPPORTED, /* 0b00110000 */
    &Instruction_LD_NN {      /* 0b00110001 */
        regpair: Reg16::SP
    },
    &Instruction_UNSUPPORTED, /* 0b00110010 */
    &Instruction_UNSUPPORTED, /* 0b00110011 */
    &Instruction_UNSUPPORTED, /* 0b00110100 */
    &Instruction_UNSUPPORTED, /* 0b00110101 */
    &Instruction_UNSUPPORTED, /* 0b00110110 */
    &Instruction_UNSUPPORTED, /* 0b00110111 */
    &Instruction_UNSUPPORTED, /* 0b00111000 */
    &Instruction_UNSUPPORTED, /* 0b00111001 */
    &Instruction_UNSUPPORTED, /* 0b00111010 */
    &Instruction_DEC {        /* 0b00111011 */
        regpair: Reg16::SP
    },
    &Instruction_UNSUPPORTED, /* 0b00111100 */
    &Instruction_UNSUPPORTED, /* 0b00111101 */
    &Instruction_UNSUPPORTED, /* 0b00111110 */
    &Instruction_UNSUPPORTED, /* 0b00111111 */
    &Instruction_LD_R_R {     /* 0b01000000 *//*TODO: Valid?*/
        rt: Reg8::B,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01000001 */
        rt: Reg8::B,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01000010 */
        rt: Reg8::B,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01000011 */
        rt: Reg8::B,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01000100 */
        rt: Reg8::B,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01000101 */
        rt: Reg8::B,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01000110 */
    &Instruction_LD_R_R {     /* 0b01000111 */
        rt: Reg8::B,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01001000 */
        rt: Reg8::C,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01001001 *//*TODO: Valid?*/
        rt: Reg8::C,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01001010 */
        rt: Reg8::C,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01001011 */
        rt: Reg8::C,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01001100 */
        rt: Reg8::C,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01001101 */
        rt: Reg8::C,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01001110 */
    &Instruction_LD_R_R {     /* 0b01001111 */
        rt: Reg8::C,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01010000 */
        rt: Reg8::D,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01010001 */
        rt: Reg8::D,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01010010 *//*TODO: Valid?*/
        rt: Reg8::D,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01010011 */
        rt: Reg8::D,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01010100 */
        rt: Reg8::D,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01010101 */
        rt: Reg8::D,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01010110 */
    &Instruction_LD_R_R {     /* 0b01010111 */
        rt: Reg8::D,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01011000 */
        rt: Reg8::E,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01011001 */
        rt: Reg8::E,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01011010 */
        rt: Reg8::E,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01011011 *//*TODO: Valid?*/
        rt: Reg8::E,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01011100 */
        rt: Reg8::E,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01011101 */
        rt: Reg8::E,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01011110 */
    &Instruction_LD_R_R {     /* 0b01011111 */
        rt: Reg8::E,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01100000 */
        rt: Reg8::H,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01100001 */
        rt: Reg8::H,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01100010 */
        rt: Reg8::H,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01100011 */
        rt: Reg8::H,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01100100 *//*TODO: Valid?*/
        rt: Reg8::H,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01100101 */
        rt: Reg8::H,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01100110 */
    &Instruction_LD_R_R {     /* 0b01100111 */
        rt: Reg8::H,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01101000 */
        rt: Reg8::L,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01101001 */
        rt: Reg8::L,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01101010 */
        rt: Reg8::L,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01101011 */
        rt: Reg8::L,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01101100 */
        rt: Reg8::L,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01101101 *//*TODO: Valid?*/
        rt: Reg8::L,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01101110 */
    &Instruction_LD_R_R {     /* 0b01101111 */
        rt: Reg8::L,
        rs: Reg8::A
    },
    &Instruction_UNSUPPORTED, /* 0b01110000 */
    &Instruction_UNSUPPORTED, /* 0b01110001 */
    &Instruction_UNSUPPORTED, /* 0b01110010 */
    &Instruction_UNSUPPORTED, /* 0b01110011 */
    &Instruction_UNSUPPORTED, /* 0b01110100 */
    &Instruction_UNSUPPORTED, /* 0b01110101 */
    &Instruction_UNSUPPORTED, /* 0b01110110 */
    &Instruction_UNSUPPORTED, /* 0b01110111 */
    &Instruction_LD_R_R {     /* 0b01111000 */
        rt: Reg8::A,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01111001 */
        rt: Reg8::A,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01111010 */
        rt: Reg8::A,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01111011 */
        rt: Reg8::A,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01111100 */
        rt: Reg8::A,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01111101 */
        rt: Reg8::A,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01111110 */
    &Instruction_LD_R_R {     /* 0b01111111 *//*TODO: Valid?*/
        rt: Reg8::A,
        rs: Reg8::A
    },
    &Instruction_UNSUPPORTED, /* 0b10000000 */
    &Instruction_UNSUPPORTED, /* 0b10000001 */
    &Instruction_UNSUPPORTED, /* 0b10000010 */
    &Instruction_UNSUPPORTED, /* 0b10000011 */
    &Instruction_UNSUPPORTED, /* 0b10000100 */
    &Instruction_UNSUPPORTED, /* 0b10000101 */
    &Instruction_UNSUPPORTED, /* 0b10000110 */
    &Instruction_UNSUPPORTED, /* 0b10000111 */
    &Instruction_UNSUPPORTED, /* 0b10001000 */
    &Instruction_UNSUPPORTED, /* 0b10001001 */
    &Instruction_UNSUPPORTED, /* 0b10001010 */
    &Instruction_UNSUPPORTED, /* 0b10001011 */
    &Instruction_UNSUPPORTED, /* 0b10001100 */
    &Instruction_UNSUPPORTED, /* 0b10001101 */
    &Instruction_UNSUPPORTED, /* 0b10001110 */
    &Instruction_UNSUPPORTED, /* 0b10001111 */
    &Instruction_UNSUPPORTED, /* 0b10010000 */
    &Instruction_UNSUPPORTED, /* 0b10010001 */
    &Instruction_UNSUPPORTED, /* 0b10010010 */
    &Instruction_UNSUPPORTED, /* 0b10010011 */
    &Instruction_UNSUPPORTED, /* 0b10010100 */
    &Instruction_UNSUPPORTED, /* 0b10010101 */
    &Instruction_UNSUPPORTED, /* 0b10010110 */
    &Instruction_UNSUPPORTED, /* 0b10010111 */
    &Instruction_UNSUPPORTED, /* 0b10011000 */
    &Instruction_UNSUPPORTED, /* 0b10011001 */
    &Instruction_UNSUPPORTED, /* 0b10011010 */
    &Instruction_UNSUPPORTED, /* 0b10011011 */
    &Instruction_UNSUPPORTED, /* 0b10011100 */
    &Instruction_UNSUPPORTED, /* 0b10011101 */
    &Instruction_UNSUPPORTED, /* 0b10011110 */
    &Instruction_UNSUPPORTED, /* 0b10011111 */
    &Instruction_UNSUPPORTED, /* 0b10100000 */
    &Instruction_UNSUPPORTED, /* 0b10100001 */
    &Instruction_UNSUPPORTED, /* 0b10100010 */
    &Instruction_UNSUPPORTED, /* 0b10100011 */
    &Instruction_UNSUPPORTED, /* 0b10100100 */
    &Instruction_UNSUPPORTED, /* 0b10100101 */
    &Instruction_UNSUPPORTED, /* 0b10100110 */
    &Instruction_UNSUPPORTED, /* 0b10100111 */
    &Instruction_UNSUPPORTED, /* 0b10101000 */
    &Instruction_UNSUPPORTED, /* 0b10101001 */
    &Instruction_UNSUPPORTED, /* 0b10101010 */
    &Instruction_UNSUPPORTED, /* 0b10101011 */
    &Instruction_UNSUPPORTED, /* 0b10101100 */
    &Instruction_UNSUPPORTED, /* 0b10101101 */
    &Instruction_UNSUPPORTED, /* 0b10101110 */
    &Instruction_UNSUPPORTED, /* 0b10101111 */
    &Instruction_OR_R {       /* 0b10110000 */
        r: Reg8::B
    },
    &Instruction_OR_R {       /* 0b10110001 */
        r: Reg8::C
    },
    &Instruction_OR_R {       /* 0b10110010 */
        r: Reg8::D
    },
    &Instruction_OR_R {       /* 0b10110011 */
        r: Reg8::E
    },
    &Instruction_OR_R {       /* 0b10110100 */
        r: Reg8::H
    },
    &Instruction_OR_R {       /* 0b10110101 */
        r: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b10110110 */
    &Instruction_OR_R {       /* 0b10110111 */
        r: Reg8::A
    },
    &Instruction_UNSUPPORTED, /* 0b10111000 */
    &Instruction_UNSUPPORTED, /* 0b10111001 */
    &Instruction_UNSUPPORTED, /* 0b10111010 */
    &Instruction_UNSUPPORTED, /* 0b10111011 */
    &Instruction_UNSUPPORTED, /* 0b10111100 */
    &Instruction_UNSUPPORTED, /* 0b10111101 */
    &Instruction_UNSUPPORTED, /* 0b10111110 */
    &Instruction_UNSUPPORTED, /* 0b10111111 */
    &Instruction_UNSUPPORTED, /* 0b11000000 */
    &Instruction_UNSUPPORTED, /* 0b11000001 */
    &Instruction_UNSUPPORTED, /* 0b11000010 */
    &Instruction_JP_NN      , /* 0b11000011 */
    &Instruction_UNSUPPORTED, /* 0b11000100 */
    &Instruction_UNSUPPORTED, /* 0b11000101 */
    &Instruction_UNSUPPORTED, /* 0b11000110 */
    &Instruction_UNSUPPORTED, /* 0b11000111 */
    &Instruction_UNSUPPORTED, /* 0b11001000 */
    &Instruction_UNSUPPORTED, /* 0b11001001 */
    &Instruction_UNSUPPORTED, /* 0b11001010 */
    &Instruction_UNSUPPORTED, /* 0b11001011 */
    &Instruction_UNSUPPORTED, /* 0b11001100 */
    &Instruction_UNSUPPORTED, /* 0b11001101 */
    &Instruction_UNSUPPORTED, /* 0b11001110 */
    &Instruction_UNSUPPORTED, /* 0b11001111 */
    &Instruction_UNSUPPORTED, /* 0b11010000 */
    &Instruction_UNSUPPORTED, /* 0b11010001 */
    &Instruction_UNSUPPORTED, /* 0b11010010 */
    &Instruction_UNSUPPORTED, /* 0b11010011 */
    &Instruction_UNSUPPORTED, /* 0b11010100 */
    &Instruction_UNSUPPORTED, /* 0b11010101 */
    &Instruction_UNSUPPORTED, /* 0b11010110 */
    &Instruction_UNSUPPORTED, /* 0b11010111 */
    &Instruction_UNSUPPORTED, /* 0b11011000 */
    &Instruction_UNSUPPORTED, /* 0b11011001 */
    &Instruction_UNSUPPORTED, /* 0b11011010 */
    &Instruction_UNSUPPORTED, /* 0b11011011 */
    &Instruction_UNSUPPORTED, /* 0b11011100 */
    &Instruction_UNSUPPORTED, /* 0b11011101 */
    &Instruction_UNSUPPORTED, /* 0b11011110 */
    &Instruction_UNSUPPORTED, /* 0b11011111 */
    &Instruction_UNSUPPORTED, /* 0b11100000 */
    &Instruction_UNSUPPORTED, /* 0b11100001 */
    &Instruction_UNSUPPORTED, /* 0b11100010 */
    &Instruction_UNSUPPORTED, /* 0b11100011 */
    &Instruction_UNSUPPORTED, /* 0b11100100 */
    &Instruction_UNSUPPORTED, /* 0b11100101 */
    &Instruction_UNSUPPORTED, /* 0b11100110 */
    &Instruction_UNSUPPORTED, /* 0b11100111 */
    &Instruction_UNSUPPORTED, /* 0b11101000 */
    &Instruction_UNSUPPORTED, /* 0b11101001 */
    &Instruction_UNSUPPORTED, /* 0b11101010 */
    &Instruction_UNSUPPORTED, /* 0b11101011 */
    &Instruction_UNSUPPORTED, /* 0b11101100 */
    &Instruction_UNSUPPORTED, /* 0b11101101 */
    &Instruction_UNSUPPORTED, /* 0b11101110 */
    &Instruction_UNSUPPORTED, /* 0b11101111 */
    &Instruction_UNSUPPORTED, /* 0b11110000 */
    &Instruction_UNSUPPORTED, /* 0b11110001 */
    &Instruction_UNSUPPORTED, /* 0b11110010 */
    &Instruction_DI,          /* 0b11110011 */
    &Instruction_UNSUPPORTED, /* 0b11110100 */
    &Instruction_UNSUPPORTED, /* 0b11110101 */
    &Instruction_UNSUPPORTED, /* 0b11110110 */
    &Instruction_UNSUPPORTED, /* 0b11110111 */
    &Instruction_UNSUPPORTED, /* 0b11111000 */
    &Instruction_UNSUPPORTED, /* 0b11111001 */
    &Instruction_UNSUPPORTED, /* 0b11111010 */
    &Instruction_UNSUPPORTED, /* 0b11111011 */
    &Instruction_UNSUPPORTED, /* 0b11111100 */
    &Instruction_UNSUPPORTED, /* 0b11111101 */
    &Instruction_UNSUPPORTED, /* 0b11111110 */
    &Instruction_UNSUPPORTED  /* 0b11111111 */
];

