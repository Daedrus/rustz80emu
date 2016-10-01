use super::memory;
use super::instructions;


enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Reg16 {
    AF = 0,
    BC = 1,
    DE = 2,
    HL = 3,

    AF_ALT = 4,
    BC_ALT = 5,
    DE_ALT = 6,
    HL_ALT = 7,

    SP = 8,
    IX = 9,
    IY = 10,
    WZ = 11
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy, RustcEncodable, RustcDecodable)]
pub enum Reg8 {
    A = 0b111,
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,

    IXL = 0b1000,
    IXH = 0b1001,
    IYL = 0b1010,
    IYH = 0b1011
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy, RustcEncodable, RustcDecodable)]
#[allow(non_camel_case_types)]
pub enum Port {
    MEMORY = 0x7ffd,
    AY38912_REG14 = 0xfffd,
    AY38912_REG14_W = 0xbffd,
    FE = 0xfe
}
}

enum_from_primitive! {
#[derive(Debug, Clone, Copy, RustcEncodable, RustcDecodable)]
pub enum FlagCond {
    NZ = 0b000,
    Z  = 0b001,
    NC = 0b010,
    C  = 0b011,
    PO = 0b100,
    PE = 0b101,
    P  = 0b110,
    M  = 0b111
}
}

bitflags! {
#[derive(RustcEncodable, RustcDecodable)]
    pub flags StatusIndicatorFlags: u8 {
        const EMPTY_FLAGS          = 0b00000000,

        const CARRY_FLAG           = 0b00000001, // C
        const ADD_SUBTRACT_FLAG    = 0b00000010, // N
        const PARITY_OVERFLOW_FLAG = 0b00000100, // P/V
        const X_FLAG               = 0b00001000, // X
        const HALF_CARRY_FLAG      = 0b00010000, // H
        const Y_FLAG               = 0b00100000, // Y
        const ZERO_FLAG            = 0b01000000, // Z
        const SIGN_FLAG            = 0b10000000, // S

        const ALL_FLAGS            = 0b11111111
    }
}

//TODO: Remove pub qualifier after reworking debugger output function
#[derive(RustcEncodable, RustcDecodable)]
pub struct Cpu {
    // main register set
    pub a: u8, pub f: StatusIndicatorFlags,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,

    // alternate register set
    pub a_alt: u8, pub f_alt: StatusIndicatorFlags,
    pub b_alt: u8, pub c_alt: u8,
    pub d_alt: u8, pub e_alt: u8,
    pub h_alt: u8, pub l_alt: u8,

    // interrupt vector
    pub i: u8,

    // memory refresh
    pub r: u8,

    // index register X
    pub ix: u16,

    // index register Y
    pub iy: u16,

    // stack pointer
    pub sp: u16,

    // program counter
    pub pc: u16,

    // temporary register (MEMPTR)
    pub wz: u16,

    // interrupt flip-flops
    pub iff1: bool,
    pub iff2: bool,

    // interrupt mode
    pub im: u8,

    memory: memory::Memory
}


impl Cpu {
    pub fn new(memory: memory::Memory) -> Cpu {
        Cpu {
            a: 0, f: StatusIndicatorFlags::empty(),
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            a_alt: 0, f_alt: StatusIndicatorFlags::empty(),
            b_alt: 0, c_alt: 0,
            d_alt: 0, e_alt: 0,
            h_alt: 0, l_alt: 0,
            i: 0,
            r: 0,
            ix: 0,
            iy: 0,
            sp: 0,
            pc: 0,
            wz: 0,
            iff1: false,
            iff2: false,
            im: 0,

            memory: memory
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_instruction();
        }
    }

    pub fn read_reg8(&self, reg: Reg8) -> u8 {
        let val = match reg {
            Reg8::A   => self.a,
            Reg8::B   => self.b,
            Reg8::C   => self.c,
            Reg8::D   => self.d,
            Reg8::E   => self.e,
            Reg8::H   => self.h,
            Reg8::L   => self.l,
            Reg8::IXL =>  (self.ix & 0x00FF)       as u8,
            Reg8::IXH => ((self.ix & 0xFF00) >> 8) as u8,
            Reg8::IYL =>  (self.iy & 0x00FF)       as u8,
            Reg8::IYH => ((self.iy & 0xFF00) >> 8) as u8
        };

        debug!("                Read value {:#04X} from register {:?}", val, reg);
        val
    }

    pub fn write_reg8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val,
            Reg8::IXL => self.ix = (self.ix & 0xFF00) | val as u16,
            Reg8::IXH => self.ix = (self.ix & 0x00FF) | ((val as u16) << 8),
            Reg8::IYL => self.iy = (self.iy & 0xFF00) | val as u16,
            Reg8::IYH => self.iy = (self.iy & 0x00FF) | ((val as u16) << 8),
        }

        debug!("                Write value {:#04X} to register {:?}", val, reg);
    }

    pub fn read_reg16(&self, reg: Reg16) -> u16 {
        let val = match reg {
            Reg16::IX => self.ix,
            Reg16::IY => self.iy,
            Reg16::SP => self.sp,
            Reg16::WZ => self.wz,
            _ => {
                let (high, low) = match reg {
                    Reg16::AF => (self.a, self.f.bits() as u8),
                    Reg16::BC => (self.b, self.c),
                    Reg16::DE => (self.d, self.e),
                    Reg16::HL => (self.h, self.l),
                    Reg16::AF_ALT => (self.a_alt, self.f_alt.bits() as u8),
                    Reg16::BC_ALT => (self.b_alt, self.c_alt),
                    Reg16::DE_ALT => (self.d_alt, self.e_alt),
                    Reg16::HL_ALT => (self.h_alt, self.l_alt),
                    _ => unreachable!()
                };
                (((high as u16) << 8 ) | low as u16)
            }
        };

        debug!("                Read value {:#04X} from register {:?}", val, reg);
        val
    }

    pub fn write_reg16(&mut self, reg: Reg16, val: u16) {
        let (high, low) = (((val & 0xFF00) >> 8) as u8, (val & 0x00FF) as u8);
        match reg {
            Reg16::AF => { self.a = high; self.f = StatusIndicatorFlags::from_bits_truncate(low); }
            Reg16::BC => { self.b = high; self.c = low; }
            Reg16::DE => { self.d = high; self.e = low; }
            Reg16::HL => { self.h = high; self.l = low; }
            Reg16::AF_ALT => { self.a_alt = high; self.f_alt = StatusIndicatorFlags::from_bits_truncate(low); }
            Reg16::BC_ALT => { self.b_alt = high; self.c_alt = low; }
            Reg16::DE_ALT => { self.d_alt = high; self.e_alt = low; }
            Reg16::HL_ALT => { self.h_alt = high; self.l_alt = low; }
            Reg16::SP => { self.sp = val }
            Reg16::IX => { self.ix = val }
            Reg16::IY => { self.iy = val }
            Reg16::WZ => { self.wz = val }
        }

        debug!("                Write value {:#06X} to register {:?}", val, reg);
    }

    pub fn inc_pc(&mut self, val: u16) { self.pc += val; }
    pub fn dec_pc(&mut self, val: u16) { self.pc -= val; }
    pub fn set_pc(&mut self, val: u16) { self.pc = val; }
    pub fn get_pc(&self) -> u16 { self.pc }

    pub fn set_iff1(&mut self)   { self.iff1 = true;  }
    pub fn clear_iff1(&mut self) { self.iff1 = false; }
    pub fn set_iff2(&mut self)   { self.iff2 = true;  }
    pub fn clear_iff2(&mut self) { self.iff2 = false; }

    // TODO: Properly model interrupt modes
    pub fn set_im(&mut self, val: u8) { self.im = val; }

    pub fn set_flag(&mut self, flag: StatusIndicatorFlags) { self.f.insert(flag); }
    pub fn clear_flag(&mut self, flag: StatusIndicatorFlags) { self.f.remove(flag); }
    pub fn get_flag(&self, flag: StatusIndicatorFlags) -> bool { self.f.contains(flag) }
    pub fn get_flags(&self) -> StatusIndicatorFlags { self.f }
    pub fn cond_flag(&mut self, flag: StatusIndicatorFlags, cond: bool) {
        if cond { self.f.insert(flag); } else { self.f.remove(flag); }
    }
    pub fn check_flags(&self, flags: StatusIndicatorFlags) -> bool { self.f == flags }

    pub fn check_cond(&self, cond: FlagCond) -> bool {
        match cond {
            FlagCond::NZ => !self.f.contains(ZERO_FLAG),
            FlagCond::Z  =>  self.f.contains(ZERO_FLAG),
            FlagCond::NC => !self.f.contains(CARRY_FLAG),
            FlagCond::C  =>  self.f.contains(CARRY_FLAG),
            FlagCond::PO => !self.f.contains(PARITY_OVERFLOW_FLAG),
            FlagCond::PE =>  self.f.contains(PARITY_OVERFLOW_FLAG),
            FlagCond::P  => !self.f.contains(SIGN_FLAG),
            FlagCond::M  =>  self.f.contains(SIGN_FLAG)
        }
    }

    pub fn decode_instruction(&self) -> &instructions::Instruction {
        let i0 = self.read_word(self.pc);
        let i1 = self.read_word(self.pc + 1);
        let i3 = self.read_word(self.pc + 3);

        match (i0, i1) {
            (0xDD, 0xCB) => instructions::INSTR_TABLE_DDCB [i3 as usize],
            (0xDD, _   ) => instructions::INSTR_TABLE_DD   [i1 as usize],
            (0xFD, 0xCB) => instructions::INSTR_TABLE_FDCB [i3 as usize],
            (0xFD, _   ) => instructions::INSTR_TABLE_FD   [i1 as usize],
            (0xCB, _   ) => instructions::INSTR_TABLE_CB   [i1 as usize],
            (0xED, _   ) => instructions::INSTR_TABLE_ED   [i1 as usize],
            (_   , _   ) => instructions::INSTR_TABLE      [i0 as usize]
        }
    }

    pub fn run_instruction(&mut self) {
        let i0 = self.read_word(self.pc);
        let i1 = self.read_word(self.pc + 1);
        let i3 = self.read_word(self.pc + 3);

        match (i0, i1) {
            (0xDD, 0xCB) => {
                self.pc += 2;
                &instructions::INSTR_TABLE_DDCB [i3 as usize].execute(self);
            },
            (0xDD, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_DD   [i1 as usize].execute(self);
            },
            (0xFD, 0xCB) => {
                self.pc += 2;
                &instructions::INSTR_TABLE_FDCB [i3 as usize].execute(self);
            },
            (0xFD, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_FD   [i1 as usize].execute(self);
            },
            (0xCB, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_CB   [i1 as usize].execute(self);
            },
            (0xED, _   ) => {
                self.pc += 1;
                &instructions::INSTR_TABLE_ED   [i1 as usize].execute(self);
            },
            (_   , _   ) => {
                &instructions::INSTR_TABLE      [i0 as usize].execute(self);
            }
        }
    }

    pub fn read_word(&self, addr: u16) -> u8 {
        self.memory.read_word(addr)
    }

    pub fn write_word(&mut self, addr: u16, val: u8) {
        self.memory.write_word(addr, val);
    }

    pub fn read_port(&self, port: Port) -> u8 {
        // TODO
        match port {
            Port::MEMORY => 0x0,
            Port::AY38912_REG14 => 0x0,
            Port::AY38912_REG14_W => unreachable!(),
            Port::FE => 0x0
        }
    }

    pub fn write_port(&mut self, port: Port, val: u8) {
        // TODO
        match port {
            Port::MEMORY => {
                let bank = val & 0b00000111;
                self.memory.change_bank(bank);

                let rombank = (val & 0b00010000) >> 4;
                self.memory.change_rom_bank(rombank);

                let screen = (val & 0b00001000) >> 3;
                if screen == 1 { panic!("Unhandled screen mode"); }

                let disable = (val & 0b00100000) >> 5;
                if disable == 1 { panic!("Unhandled disabled mode"); }
            }
            Port::AY38912_REG14 => (),
            Port::AY38912_REG14_W => (),
            Port::FE => ()
        }
    }
}


