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
#[allow(non_camel_case_types)]
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
    IYH = 0b1011,

    I = 0b1100,
    R = 0b1101,

    A_ALT = 0b10111,
    B_ALT = 0b10000,
    C_ALT = 0b10001,
    D_ALT = 0b10010,
    E_ALT = 0b10011,
    H_ALT = 0b10100,
    L_ALT = 0b10101,
    F_ALT = 0b11000,
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

#[derive(RustcEncodable, RustcDecodable)]
pub struct Cpu {
    // main register set
    a: u8, f: StatusIndicatorFlags,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,

    // alternate register set
    a_alt: u8, f_alt: StatusIndicatorFlags,
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

    // temporary register (MEMPTR)
    wz: u16,

    // interrupt flip-flops
    iff1: bool,
    iff2: bool,

    // interrupt mode
    im: u8,

    // T Cycle counter
    pub tcycles: u64,

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

            tcycles: 0,

            memory: memory
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_instruction();
        }
    }

    pub fn reset(&mut self) {
        self.a = 0; self.f = StatusIndicatorFlags::empty();
        self.b = 0; self.c = 0;
        self.d = 0; self.e = 0;
        self.h = 0; self.l = 0;
        self.a_alt = 0; self.f_alt = StatusIndicatorFlags::empty();
        self.b_alt = 0; self.c_alt = 0;
        self.d_alt = 0; self.e_alt = 0;
        self.h_alt = 0; self.l_alt = 0;
        self.i = 0;
        self.r = 0;
        self.ix = 0;
        self.iy = 0;
        self.sp = 0;
        self.pc = 0;
        self.wz = 0;
        self.iff1 = false;
        self.iff2 = false;
        self.im = 0;

        self.tcycles = 0;

        self.memory.clear();
    }

    pub fn read_reg8(&self, reg: Reg8) -> u8 {
        let val = match reg {
            Reg8::A     => self.a,
            Reg8::B     => self.b,
            Reg8::C     => self.c,
            Reg8::D     => self.d,
            Reg8::E     => self.e,
            Reg8::H     => self.h,
            Reg8::L     => self.l,
            Reg8::I     => self.i,
            Reg8::R     => self.r,
            Reg8::A_ALT => self.a_alt,
            Reg8::B_ALT => self.b_alt,
            Reg8::C_ALT => self.c_alt,
            Reg8::D_ALT => self.d_alt,
            Reg8::E_ALT => self.e_alt,
            Reg8::H_ALT => self.h_alt,
            Reg8::L_ALT => self.l_alt,
            Reg8::F_ALT => self.f_alt.bits() as u8,
            Reg8::IXL   =>  (self.ix & 0x00FF)       as u8,
            Reg8::IXH   => ((self.ix & 0xFF00) >> 8) as u8,
            Reg8::IYL   =>  (self.iy & 0x00FF)       as u8,
            Reg8::IYH   => ((self.iy & 0xFF00) >> 8) as u8
        };

        debug!("                Read value {:#04X} from register {:?}", val, reg);
        val
    }

    pub fn write_reg8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A     => self.a = val,
            Reg8::B     => self.b = val,
            Reg8::C     => self.c = val,
            Reg8::D     => self.d = val,
            Reg8::E     => self.e = val,
            Reg8::H     => self.h = val,
            Reg8::L     => self.l = val,
            Reg8::I     => self.i = val,
            Reg8::R     => self.r = val,
            Reg8::A_ALT => self.a_alt = val,
            Reg8::B_ALT => self.b_alt = val,
            Reg8::C_ALT => self.c_alt = val,
            Reg8::D_ALT => self.d_alt = val,
            Reg8::E_ALT => self.e_alt = val,
            Reg8::H_ALT => self.h_alt = val,
            Reg8::L_ALT => self.l_alt = val,
            Reg8::F_ALT => self.f_alt = StatusIndicatorFlags::from_bits_truncate(val),
            Reg8::IXL   => self.ix = (self.ix & 0xFF00) | val as u16,
            Reg8::IXH   => self.ix = (self.ix & 0x00FF) | ((val as u16) << 8),
            Reg8::IYL   => self.iy = (self.iy & 0xFF00) | val as u16,
            Reg8::IYH   => self.iy = (self.iy & 0x00FF) | ((val as u16) << 8),
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

    pub fn inc_r(&mut self, val: u8) { self.r = (self.r.wrapping_add(val)) & 0b0111111; }

    pub fn set_iff1(&mut self)   { self.iff1 = true;  }
    pub fn clear_iff1(&mut self) { self.iff1 = false; }
    pub fn get_iff1(&self) -> bool { self.iff1 }
    pub fn set_iff2(&mut self)   { self.iff2 = true;  }
    pub fn clear_iff2(&mut self) { self.iff2 = false; }
    pub fn get_iff2(&self) -> bool { self.iff2 }

    // TODO: Properly model interrupt modes
    pub fn set_im(&mut self, val: u8) { self.im = val; }
    pub fn get_im(&self) -> u8 { self.im }

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
        let i0 = self.memory.read_word(self.pc);
        let i1 = self.memory.read_word(self.pc + 1);
        let i3 = self.memory.read_word(self.pc + 3);

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
        let mut curr_pc = self.pc;
        let i0 = self.fetch_op(curr_pc);

        // TODO: Handle sequences of DD and FD
        match i0 {
            0xCB => {
                self.inc_pc(1); curr_pc += 1;
                let i1 = self.read_word(curr_pc);
                self.inc_r(2);
                &instructions::INSTR_TABLE_CB[i1 as usize].execute(self);
            },
            0xDD => {
                self.inc_pc(1); curr_pc += 1;
                let i1 = self.read_word(curr_pc);
                self.inc_r(2);
                match i1 {
                    0xCB => {
                        self.inc_pc(1); curr_pc += 1;
                        let i3 = self.read_word(curr_pc + 1);
                        &instructions::INSTR_TABLE_DDCB[i3 as usize].execute(self);
                    },
                    _    => {
                        &instructions::INSTR_TABLE_DD[i1 as usize].execute(self);
                    }
                };
            },
            0xED => {
                self.inc_pc(1); curr_pc += 1;
                let i1 = self.read_word(curr_pc);
                self.inc_r(2);
                &instructions::INSTR_TABLE_ED[i1 as usize].execute(self);
            },
            0xFD => {
                self.inc_pc(1); curr_pc += 1;
                let i1 = self.read_word(curr_pc);
                self.inc_r(2);
                match i1 {
                    0xCB => {
                        self.inc_pc(1); curr_pc += 1;
                        let i3 = self.read_word(curr_pc + 1);
                        &instructions::INSTR_TABLE_FDCB[i3 as usize].execute(self);
                    },
                    _    => {
                        &instructions::INSTR_TABLE_FD[i1 as usize].execute(self);
                    }
                };
            },
            _    => {
                self.inc_r(1);
                &instructions::INSTR_TABLE[i0 as usize].execute(self);
            }
        }
    }

    #[inline(always)]
    pub fn contend_read(&mut self, addr: u16, tcycles: u64) {
        println!("{} MC {:04x}", self.tcycles, addr);
        self.tcycles += tcycles;
    }

    #[inline(always)]
    pub fn contend_read_no_mreq(&mut self, addr: u16) {
        println!("{} MC {:04x}", self.tcycles, addr);
        self.tcycles += 1;
    }

    pub fn fetch_op(&mut self, addr: u16) -> u8 {
        self.contend_read(addr, 4);
        let val = self.memory.read_word(addr);
        println!("{} MR {:04x} {:02x}", self.tcycles, addr, val);
        val
    }

    pub fn read_word(&mut self, addr: u16) -> u8 {
        self.contend_read(addr, 3);
        let val = self.memory.read_word(addr);
        println!("{} MR {:04x} {:02x}", self.tcycles, addr, val);
        val
    }

    pub fn write_word(&mut self, addr: u16, val: u8) {
        self.contend_read(addr, 3);
        self.memory.write_word(addr, val);
        println!("{} MW {:04x} {:02x}", self.tcycles, addr, val);
    }

    // Helper function to be able to write to memory without increasing the tcycles
    // Used for setting up the memory in the fuse tests
    pub fn zero_cycle_write_word(&mut self, addr: u16, val: u8) {
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


