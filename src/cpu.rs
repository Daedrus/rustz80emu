use super::memory;
use num::FromPrimitive;
use std::fmt;

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
enum Reg16 {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
enum Reg8 {
    A = 0b111,
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101
}
}

pub struct Cpu {
    // main register set
    a: u8, f: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,

    // alternate register set
    a_alt: u8, f_alt: u8,
    b_alt: u8, c_alt: u8,
    d_alt: u8, e_alt: u8,
    h_alt: u8, l_alt: u8,

    // interrupt vector
    i: u8,

    // memory refresh
    r: u8,

    // index register X
    ix: u16,

    // index register Y
    iy: u16,

    // stack pointer
    sp: u16,

    // program counter
    pc: u16,

    // interrupt flip-flops
    iff1: bool,
    iff2: bool,

    memory: memory::Memory,

    instr_table: [fn(&mut Cpu); 256]
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "
                         -----------
                     af: | {:02X} | {:02X} |
                     bc: | {:02X} | {:02X} |
                     de: | {:02X} | {:02X} |
                     hl: | {:02X} | {:02X} |
                         -----------
                     ir  | {:02X} | {:02X} |
                         -----------
                     ix  |   {:04X}  |
                     iy  |   {:04X}  |
                     sp  |   {:04X}  |
                     pc  |   {:04X}  |
                         -----------",
                      self.a, self.f,
                      self.b, self.c,
                      self.d, self.e,
                      self.h, self.l,
                      self.i, self.r,
                      self.ix,
                      self.iy,
                      self.sp,
                      self.pc)
    }
}

impl Cpu {
    pub fn new(memory: memory::Memory) -> Cpu {
        Cpu {
            a: 0, f: 0,
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            a_alt: 0, f_alt: 0,
            b_alt: 0, c_alt: 0,
            d_alt: 0, e_alt: 0,
            h_alt: 0, l_alt: 0,
            i: 0,
            r: 0,
            ix: 0,
            iy: 0,
            sp: 0,
            pc: 0,
            iff1: false,
            iff2: false,

            memory: memory,

            instr_table: [
                Cpu::instr_UNSUPPORTED, /* 0b00000000 */
                Cpu::instr_LD_BC_NN   , /* 0b00000001 */
                Cpu::instr_UNSUPPORTED, /* 0b00000010 */
                Cpu::instr_UNSUPPORTED, /* 0b00000011 */
                Cpu::instr_UNSUPPORTED, /* 0b00000100 */
                Cpu::instr_UNSUPPORTED, /* 0b00000101 */
                Cpu::instr_UNSUPPORTED, /* 0b00000110 */
                Cpu::instr_UNSUPPORTED, /* 0b00000111 */
                Cpu::instr_UNSUPPORTED, /* 0b00001000 */
                Cpu::instr_UNSUPPORTED, /* 0b00001001 */
                Cpu::instr_UNSUPPORTED, /* 0b00001010 */
                Cpu::instr_DEC_BC     , /* 0b00001011 */
                Cpu::instr_UNSUPPORTED, /* 0b00001100 */
                Cpu::instr_UNSUPPORTED, /* 0b00001101 */
                Cpu::instr_UNSUPPORTED, /* 0b00001110 */
                Cpu::instr_UNSUPPORTED, /* 0b00001111 */
                Cpu::instr_UNSUPPORTED, /* 0b00010000 */
                Cpu::instr_LD_DE_NN,    /* 0b00010001 */
                Cpu::instr_UNSUPPORTED, /* 0b00010010 */
                Cpu::instr_UNSUPPORTED, /* 0b00010011 */
                Cpu::instr_UNSUPPORTED, /* 0b00010100 */
                Cpu::instr_UNSUPPORTED, /* 0b00010101 */
                Cpu::instr_UNSUPPORTED, /* 0b00010110 */
                Cpu::instr_UNSUPPORTED, /* 0b00010111 */
                Cpu::instr_UNSUPPORTED, /* 0b00011000 */
                Cpu::instr_UNSUPPORTED, /* 0b00011001 */
                Cpu::instr_UNSUPPORTED, /* 0b00011010 */
                Cpu::instr_DEC_DE     , /* 0b00011011 */
                Cpu::instr_UNSUPPORTED, /* 0b00011100 */
                Cpu::instr_UNSUPPORTED, /* 0b00011101 */
                Cpu::instr_UNSUPPORTED, /* 0b00011110 */
                Cpu::instr_UNSUPPORTED, /* 0b00011111 */
                Cpu::instr_UNSUPPORTED, /* 0b00100000 */
                Cpu::instr_LD_HL_NN   , /* 0b00100001 */
                Cpu::instr_UNSUPPORTED, /* 0b00100010 */
                Cpu::instr_UNSUPPORTED, /* 0b00100011 */
                Cpu::instr_UNSUPPORTED, /* 0b00100100 */
                Cpu::instr_UNSUPPORTED, /* 0b00100101 */
                Cpu::instr_UNSUPPORTED, /* 0b00100110 */
                Cpu::instr_UNSUPPORTED, /* 0b00100111 */
                Cpu::instr_UNSUPPORTED, /* 0b00101000 */
                Cpu::instr_UNSUPPORTED, /* 0b00101001 */
                Cpu::instr_UNSUPPORTED, /* 0b00101010 */
                Cpu::instr_DEC_HL     , /* 0b00101011 */
                Cpu::instr_UNSUPPORTED, /* 0b00101100 */
                Cpu::instr_UNSUPPORTED, /* 0b00101101 */
                Cpu::instr_UNSUPPORTED, /* 0b00101110 */
                Cpu::instr_UNSUPPORTED, /* 0b00101111 */
                Cpu::instr_UNSUPPORTED, /* 0b00110000 */
                Cpu::instr_LD_SP_NN   , /* 0b00110001 */
                Cpu::instr_UNSUPPORTED, /* 0b00110010 */
                Cpu::instr_UNSUPPORTED, /* 0b00110011 */
                Cpu::instr_UNSUPPORTED, /* 0b00110100 */
                Cpu::instr_UNSUPPORTED, /* 0b00110101 */
                Cpu::instr_UNSUPPORTED, /* 0b00110110 */
                Cpu::instr_UNSUPPORTED, /* 0b00110111 */
                Cpu::instr_UNSUPPORTED, /* 0b00111000 */
                Cpu::instr_UNSUPPORTED, /* 0b00111001 */
                Cpu::instr_UNSUPPORTED, /* 0b00111010 */
                Cpu::instr_DEC_SP     , /* 0b00111011 */
                Cpu::instr_UNSUPPORTED, /* 0b00111100 */
                Cpu::instr_UNSUPPORTED, /* 0b00111101 */
                Cpu::instr_UNSUPPORTED, /* 0b00111110 */
                Cpu::instr_UNSUPPORTED, /* 0b00111111 */
                Cpu::instr_LD_R_R,      /* 0b01000000 */
                Cpu::instr_LD_R_R,      /* 0b01000001 */
                Cpu::instr_LD_R_R,      /* 0b01000010 */
                Cpu::instr_LD_R_R,      /* 0b01000011 */
                Cpu::instr_LD_R_R,      /* 0b01000100 */
                Cpu::instr_LD_R_R,      /* 0b01000101 */
                Cpu::instr_UNSUPPORTED, /* 0b01000110 */
                Cpu::instr_LD_R_R,      /* 0b01000111 */
                Cpu::instr_LD_R_R,      /* 0b01001000 */
                Cpu::instr_LD_R_R,      /* 0b01001001 */
                Cpu::instr_LD_R_R,      /* 0b01001010 */
                Cpu::instr_LD_R_R,      /* 0b01001011 */
                Cpu::instr_LD_R_R,      /* 0b01001100 */
                Cpu::instr_LD_R_R,      /* 0b01001101 */
                Cpu::instr_UNSUPPORTED, /* 0b01001110 */
                Cpu::instr_LD_R_R,      /* 0b01001111 */
                Cpu::instr_LD_R_R,      /* 0b01010000 */
                Cpu::instr_LD_R_R,      /* 0b01010001 */
                Cpu::instr_LD_R_R,      /* 0b01010010 */
                Cpu::instr_LD_R_R,      /* 0b01010011 */
                Cpu::instr_LD_R_R,      /* 0b01010100 */
                Cpu::instr_LD_R_R,      /* 0b01010101 */
                Cpu::instr_UNSUPPORTED, /* 0b01010110 */
                Cpu::instr_LD_R_R,      /* 0b01010111 */
                Cpu::instr_LD_R_R,      /* 0b01011000 */
                Cpu::instr_LD_R_R,      /* 0b01011001 */
                Cpu::instr_LD_R_R,      /* 0b01011010 */
                Cpu::instr_LD_R_R,      /* 0b01011011 */
                Cpu::instr_LD_R_R,      /* 0b01011100 */
                Cpu::instr_LD_R_R,      /* 0b01011101 */
                Cpu::instr_UNSUPPORTED, /* 0b01011110 */
                Cpu::instr_LD_R_R,      /* 0b01011111 */
                Cpu::instr_LD_R_R,      /* 0b01100000 */
                Cpu::instr_LD_R_R,      /* 0b01100001 */
                Cpu::instr_LD_R_R,      /* 0b01100010 */
                Cpu::instr_LD_R_R,      /* 0b01100011 */
                Cpu::instr_LD_R_R,      /* 0b01100100 */
                Cpu::instr_LD_R_R,      /* 0b01100101 */
                Cpu::instr_UNSUPPORTED, /* 0b01100110 */
                Cpu::instr_LD_R_R,      /* 0b01100111 */
                Cpu::instr_LD_R_R,      /* 0b01101000 */
                Cpu::instr_LD_R_R,      /* 0b01101001 */
                Cpu::instr_LD_R_R,      /* 0b01101010 */
                Cpu::instr_LD_R_R,      /* 0b01101011 */
                Cpu::instr_LD_R_R,      /* 0b01101100 */
                Cpu::instr_LD_R_R,      /* 0b01101101 */
                Cpu::instr_UNSUPPORTED, /* 0b01101110 */
                Cpu::instr_LD_R_R,      /* 0b01101111 */
                Cpu::instr_UNSUPPORTED, /* 0b01110000 */
                Cpu::instr_UNSUPPORTED, /* 0b01110001 */
                Cpu::instr_UNSUPPORTED, /* 0b01110010 */
                Cpu::instr_UNSUPPORTED, /* 0b01110011 */
                Cpu::instr_UNSUPPORTED, /* 0b01110100 */
                Cpu::instr_UNSUPPORTED, /* 0b01110101 */
                Cpu::instr_UNSUPPORTED, /* 0b01110110 */
                Cpu::instr_UNSUPPORTED, /* 0b01110111 */
                Cpu::instr_LD_R_R,      /* 0b01111000 */
                Cpu::instr_LD_R_R,      /* 0b01111001 */
                Cpu::instr_LD_R_R,      /* 0b01111010 */
                Cpu::instr_LD_R_R,      /* 0b01111011 */
                Cpu::instr_LD_R_R,      /* 0b01111100 */
                Cpu::instr_LD_R_R,      /* 0b01111101 */
                Cpu::instr_UNSUPPORTED, /* 0b01111110 */
                Cpu::instr_LD_R_R,      /* 0b01111111 */
                Cpu::instr_UNSUPPORTED, /* 0b10000000 */
                Cpu::instr_UNSUPPORTED, /* 0b10000001 */
                Cpu::instr_UNSUPPORTED, /* 0b10000010 */
                Cpu::instr_UNSUPPORTED, /* 0b10000011 */
                Cpu::instr_UNSUPPORTED, /* 0b10000100 */
                Cpu::instr_UNSUPPORTED, /* 0b10000101 */
                Cpu::instr_UNSUPPORTED, /* 0b10000110 */
                Cpu::instr_UNSUPPORTED, /* 0b10000111 */
                Cpu::instr_UNSUPPORTED, /* 0b10001000 */
                Cpu::instr_UNSUPPORTED, /* 0b10001001 */
                Cpu::instr_UNSUPPORTED, /* 0b10001010 */
                Cpu::instr_UNSUPPORTED, /* 0b10001011 */
                Cpu::instr_UNSUPPORTED, /* 0b10001100 */
                Cpu::instr_UNSUPPORTED, /* 0b10001101 */
                Cpu::instr_UNSUPPORTED, /* 0b10001110 */
                Cpu::instr_UNSUPPORTED, /* 0b10001111 */
                Cpu::instr_UNSUPPORTED, /* 0b10010000 */
                Cpu::instr_UNSUPPORTED, /* 0b10010001 */
                Cpu::instr_UNSUPPORTED, /* 0b10010010 */
                Cpu::instr_UNSUPPORTED, /* 0b10010011 */
                Cpu::instr_UNSUPPORTED, /* 0b10010100 */
                Cpu::instr_UNSUPPORTED, /* 0b10010101 */
                Cpu::instr_UNSUPPORTED, /* 0b10010110 */
                Cpu::instr_UNSUPPORTED, /* 0b10010111 */
                Cpu::instr_UNSUPPORTED, /* 0b10011000 */
                Cpu::instr_UNSUPPORTED, /* 0b10011001 */
                Cpu::instr_UNSUPPORTED, /* 0b10011010 */
                Cpu::instr_UNSUPPORTED, /* 0b10011011 */
                Cpu::instr_UNSUPPORTED, /* 0b10011100 */
                Cpu::instr_UNSUPPORTED, /* 0b10011101 */
                Cpu::instr_UNSUPPORTED, /* 0b10011110 */
                Cpu::instr_UNSUPPORTED, /* 0b10011111 */
                Cpu::instr_UNSUPPORTED, /* 0b10100000 */
                Cpu::instr_UNSUPPORTED, /* 0b10100001 */
                Cpu::instr_UNSUPPORTED, /* 0b10100010 */
                Cpu::instr_UNSUPPORTED, /* 0b10100011 */
                Cpu::instr_UNSUPPORTED, /* 0b10100100 */
                Cpu::instr_UNSUPPORTED, /* 0b10100101 */
                Cpu::instr_UNSUPPORTED, /* 0b10100110 */
                Cpu::instr_UNSUPPORTED, /* 0b10100111 */
                Cpu::instr_UNSUPPORTED, /* 0b10101000 */
                Cpu::instr_UNSUPPORTED, /* 0b10101001 */
                Cpu::instr_UNSUPPORTED, /* 0b10101010 */
                Cpu::instr_UNSUPPORTED, /* 0b10101011 */
                Cpu::instr_UNSUPPORTED, /* 0b10101100 */
                Cpu::instr_UNSUPPORTED, /* 0b10101101 */
                Cpu::instr_UNSUPPORTED, /* 0b10101110 */
                Cpu::instr_UNSUPPORTED, /* 0b10101111 */
                Cpu::instr_UNSUPPORTED, /* 0b10110000 */
                Cpu::instr_UNSUPPORTED, /* 0b10110001 */
                Cpu::instr_UNSUPPORTED, /* 0b10110010 */
                Cpu::instr_UNSUPPORTED, /* 0b10110011 */
                Cpu::instr_UNSUPPORTED, /* 0b10110100 */
                Cpu::instr_UNSUPPORTED, /* 0b10110101 */
                Cpu::instr_UNSUPPORTED, /* 0b10110110 */
                Cpu::instr_UNSUPPORTED, /* 0b10110111 */
                Cpu::instr_UNSUPPORTED, /* 0b10111000 */
                Cpu::instr_UNSUPPORTED, /* 0b10111001 */
                Cpu::instr_UNSUPPORTED, /* 0b10111010 */
                Cpu::instr_UNSUPPORTED, /* 0b10111011 */
                Cpu::instr_UNSUPPORTED, /* 0b10111100 */
                Cpu::instr_UNSUPPORTED, /* 0b10111101 */
                Cpu::instr_UNSUPPORTED, /* 0b10111110 */
                Cpu::instr_UNSUPPORTED, /* 0b10111111 */
                Cpu::instr_UNSUPPORTED, /* 0b11000000 */
                Cpu::instr_UNSUPPORTED, /* 0b11000001 */
                Cpu::instr_UNSUPPORTED, /* 0b11000010 */
                Cpu::instr_UNSUPPORTED, /* 0b11000011 */
                Cpu::instr_UNSUPPORTED, /* 0b11000100 */
                Cpu::instr_UNSUPPORTED, /* 0b11000101 */
                Cpu::instr_UNSUPPORTED, /* 0b11000110 */
                Cpu::instr_UNSUPPORTED, /* 0b11000111 */
                Cpu::instr_UNSUPPORTED, /* 0b11001000 */
                Cpu::instr_UNSUPPORTED, /* 0b11001001 */
                Cpu::instr_UNSUPPORTED, /* 0b11001010 */
                Cpu::instr_UNSUPPORTED, /* 0b11001011 */
                Cpu::instr_UNSUPPORTED, /* 0b11001100 */
                Cpu::instr_UNSUPPORTED, /* 0b11001101 */
                Cpu::instr_UNSUPPORTED, /* 0b11001110 */
                Cpu::instr_UNSUPPORTED, /* 0b11001111 */
                Cpu::instr_UNSUPPORTED, /* 0b11010000 */
                Cpu::instr_UNSUPPORTED, /* 0b11010001 */
                Cpu::instr_UNSUPPORTED, /* 0b11010010 */
                Cpu::instr_UNSUPPORTED, /* 0b11010011 */
                Cpu::instr_UNSUPPORTED, /* 0b11010100 */
                Cpu::instr_UNSUPPORTED, /* 0b11010101 */
                Cpu::instr_UNSUPPORTED, /* 0b11010110 */
                Cpu::instr_UNSUPPORTED, /* 0b11010111 */
                Cpu::instr_UNSUPPORTED, /* 0b11011000 */
                Cpu::instr_UNSUPPORTED, /* 0b11011001 */
                Cpu::instr_UNSUPPORTED, /* 0b11011010 */
                Cpu::instr_UNSUPPORTED, /* 0b11011011 */
                Cpu::instr_UNSUPPORTED, /* 0b11011100 */
                Cpu::instr_UNSUPPORTED, /* 0b11011101 */
                Cpu::instr_UNSUPPORTED, /* 0b11011110 */
                Cpu::instr_UNSUPPORTED, /* 0b11011111 */
                Cpu::instr_UNSUPPORTED, /* 0b11100000 */
                Cpu::instr_UNSUPPORTED, /* 0b11100001 */
                Cpu::instr_UNSUPPORTED, /* 0b11100010 */
                Cpu::instr_UNSUPPORTED, /* 0b11100011 */
                Cpu::instr_UNSUPPORTED, /* 0b11100100 */
                Cpu::instr_UNSUPPORTED, /* 0b11100101 */
                Cpu::instr_UNSUPPORTED, /* 0b11100110 */
                Cpu::instr_UNSUPPORTED, /* 0b11100111 */
                Cpu::instr_UNSUPPORTED, /* 0b11101000 */
                Cpu::instr_UNSUPPORTED, /* 0b11101001 */
                Cpu::instr_UNSUPPORTED, /* 0b11101010 */
                Cpu::instr_UNSUPPORTED, /* 0b11101011 */
                Cpu::instr_UNSUPPORTED, /* 0b11101100 */
                Cpu::instr_UNSUPPORTED, /* 0b11101101 */
                Cpu::instr_UNSUPPORTED, /* 0b11101110 */
                Cpu::instr_UNSUPPORTED, /* 0b11101111 */
                Cpu::instr_UNSUPPORTED, /* 0b11110000 */
                Cpu::instr_UNSUPPORTED, /* 0b11110001 */
                Cpu::instr_UNSUPPORTED, /* 0b11110010 */
                Cpu::instr_DI,          /* 0b11110011 */
                Cpu::instr_UNSUPPORTED, /* 0b11110100 */
                Cpu::instr_UNSUPPORTED, /* 0b11110101 */
                Cpu::instr_UNSUPPORTED, /* 0b11110110 */
                Cpu::instr_UNSUPPORTED, /* 0b11110111 */
                Cpu::instr_UNSUPPORTED, /* 0b11111000 */
                Cpu::instr_UNSUPPORTED, /* 0b11111001 */
                Cpu::instr_UNSUPPORTED, /* 0b11111010 */
                Cpu::instr_UNSUPPORTED, /* 0b11111011 */
                Cpu::instr_UNSUPPORTED, /* 0b11111100 */
                Cpu::instr_UNSUPPORTED, /* 0b11111101 */
                Cpu::instr_UNSUPPORTED, /* 0b11111110 */
                Cpu::instr_UNSUPPORTED  /* 0b11111111 */
            ]
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_instruction();
        }
    }

    fn read_reg8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l
        }
    }

    fn write_reg8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val
        }
    }

    fn read_reg16(&self, reg: Reg16) -> u16 {
        let value = match reg {
            Reg16::SP => self.sp,
            _ => {
                let (high, low) = match reg {
                    Reg16::BC => (self.b, self.c),
                    Reg16::DE => (self.d, self.e),
                    Reg16::HL => (self.h, self.l),
                    _ => unreachable!()
                };
                (((high as u16) << 8 ) + low as u16)
            }
        };
        value
    }

    fn write_reg16(&mut self, reg: Reg16, val: u16) {
        let (high, low) = (((val & 0xFF00) >> 8) as u8, (val & 0x00FF) as u8);
        match reg {
            Reg16::BC => { self.b = high; self.c = low; }
            Reg16::DE => { self.d = high; self.e = low; }
            Reg16::HL => { self.h = high; self.l = low; }
            Reg16::SP => { self.sp = val }
        }
    }

    fn instr_UNSUPPORTED(_: &mut Cpu) {
        panic!("Unsupported instruction");
    }

    fn instr_DI(cpu: &mut Cpu) {
        cpu.iff1 = false;
        cpu.iff2 = false;
        println!("{:#x}: DI", cpu.pc);
        cpu.pc += 1;
    }

    fn instr_DEC_BC(cpu: &mut Cpu) { Cpu::instr_DEC_SS(cpu, Reg16::BC); }
    fn instr_DEC_DE(cpu: &mut Cpu) { Cpu::instr_DEC_SS(cpu, Reg16::DE); }
    fn instr_DEC_HL(cpu: &mut Cpu) { Cpu::instr_DEC_SS(cpu, Reg16::HL); }
    fn instr_DEC_SP(cpu: &mut Cpu) { Cpu::instr_DEC_SS(cpu, Reg16::SP); }
    fn instr_DEC_SS(cpu: &mut Cpu, regpair: Reg16) {
        let oldregval = cpu.read_reg16(regpair);
        cpu.write_reg16(regpair, oldregval - 1);

        println!("{:#x}: DEC {:?}", cpu.pc, regpair);
        cpu.pc += 1;
    }

    fn instr_LD_BC_NN(cpu: &mut Cpu) { Cpu::instr_LD_DD_NN(cpu, Reg16::BC); }
    fn instr_LD_DE_NN(cpu: &mut Cpu) { Cpu::instr_LD_DD_NN(cpu, Reg16::DE); }
    fn instr_LD_HL_NN(cpu: &mut Cpu) { Cpu::instr_LD_DD_NN(cpu, Reg16::HL); }
    fn instr_LD_SP_NN(cpu: &mut Cpu) { Cpu::instr_LD_DD_NN(cpu, Reg16::SP); }
    fn instr_LD_DD_NN(cpu: &mut Cpu, regpair: Reg16) {
        let nn =  (cpu.read_word(cpu.pc + 1) as u16) +
                 ((cpu.read_word(cpu.pc + 2) as u16) << 8);
        cpu.write_reg16(regpair, nn);

        println!("{:#x}: LD {:?}, ${:x}", cpu.pc, regpair, nn);
        cpu.pc += 3;
    }

    fn instr_LD_R_R(cpu: &mut Cpu) {
        /*let rt = Reg8::from_u8((instr >> 3) & 0b111).unwrap();
        let rs = Reg8::from_u8( instr       & 0b111).unwrap();
        let rsval = cpu.read_reg8(rs);
        cpu.write_reg8(rt, rsval);
        println!("{:#x}: LD {:?}, {:?}", cpu.pc, rt, rs);*/
        cpu.pc += 1;
    }

    fn run_instruction(&mut self) {
        let instruction = self.read_word(self.pc);

        self.instr_table[instruction as usize](self);

        println!("{:?}", self);
    }

    fn read_word(&self, addr: u16) -> u8 {
        self.memory.read_word(addr)
    }
}

