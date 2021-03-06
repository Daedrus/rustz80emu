use super::instructions::INSTR_TABLE;
use super::instructions_ddcb::INSTR_TABLE_DDCB;
use super::instructions_fdcb::INSTR_TABLE_FDCB;
use super::instructions_cb::INSTR_TABLE_CB;
use super::instructions_ed::INSTR_TABLE_ED;
use super::instructions_dd::INSTR_TABLE_DD;
use super::instructions_fd::INSTR_TABLE_FD;
use ::interconnect::*;

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
        WZ = 11,

        IR = 12
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
    pub struct StatusIndicatorFlags: u8 {
        const EMPTY_FLAGS          = 0b00000000;

        const CARRY_FLAG           = 0b00000001; // C
        const ADD_SUBTRACT_FLAG    = 0b00000010; // N
        const PARITY_OVERFLOW_FLAG = 0b00000100; // P/V
        const X_FLAG               = 0b00001000; // X
        const HALF_CARRY_FLAG      = 0b00010000; // H
        const Y_FLAG               = 0b00100000; // Y
        const ZERO_FLAG            = 0b01000000; // Z
        const SIGN_FLAG            = 0b10000000; // S

        const ALL_FLAGS            = 0b11111111;
    }
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct Cpu {
    // main register set
    a: u8,
    f: StatusIndicatorFlags,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    // alternate register set
    a_alt: u8,
    f_alt: StatusIndicatorFlags,
    b_alt: u8,
    c_alt: u8,
    d_alt: u8,
    e_alt: u8,
    h_alt: u8,
    l_alt: u8,

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
    pub tcycles: u32,

    // HALT state
    halted: bool,

    interconnect: Interconnect,
}


impl Cpu {
    pub fn new(interconnect: Interconnect) -> Self {
        Cpu {
            a: 0xFF,
            f: StatusIndicatorFlags::all(),
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            a_alt: 0xFF,
            f_alt: StatusIndicatorFlags::all(),
            b_alt: 0x00,
            c_alt: 0x00,
            d_alt: 0x00,
            e_alt: 0x00,
            h_alt: 0x00,
            l_alt: 0x00,
            i: 0x00,
            r: 0x00,
            ix: 0x00,
            iy: 0x00,
            sp: 0xFFFF,
            pc: 0x0000,
            wz: 0x0000,
            iff1: false,
            iff2: false,
            im: 0,
            halted: false,

            tcycles: 0,

            interconnect: interconnect,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0xFF;
        self.f = StatusIndicatorFlags::all();
        self.b = 0x00;
        self.c = 0x00;
        self.d = 0x00;
        self.e = 0x00;
        self.h = 0x00;
        self.l = 0x00;
        self.a_alt = 0xFF;
        self.f_alt = StatusIndicatorFlags::all();
        self.b_alt = 0x00;
        self.c_alt = 0x00;
        self.d_alt = 0x00;
        self.e_alt = 0x00;
        self.h_alt = 0x00;
        self.l_alt = 0x00;
        self.i = 0x00;
        self.r = 0x00;
        self.ix = 0x00;
        self.iy = 0x00;
        self.sp = 0xFFFF;
        self.pc = 0x0000;
        self.wz = 0x0000;
        self.iff1 = false;
        self.iff2 = false;
        self.im = 0;
        self.halted = false;

        self.tcycles = 0;

        self.interconnect.reset();
    }

    pub fn read_reg8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
            Reg8::I => self.i,
            Reg8::R => self.r,
            Reg8::A_ALT => self.a_alt,
            Reg8::B_ALT => self.b_alt,
            Reg8::C_ALT => self.c_alt,
            Reg8::D_ALT => self.d_alt,
            Reg8::E_ALT => self.e_alt,
            Reg8::H_ALT => self.h_alt,
            Reg8::L_ALT => self.l_alt,
            Reg8::F_ALT => self.f_alt.bits() as u8,
            Reg8::IXL => (self.ix & 0x00FF) as u8,
            Reg8::IXH => ((self.ix & 0xFF00) >> 8) as u8,
            Reg8::IYL => (self.iy & 0x00FF) as u8,
            Reg8::IYH => ((self.iy & 0xFF00) >> 8) as u8,
        }
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
            Reg8::I => self.i = val,
            Reg8::R => self.r = val,
            Reg8::A_ALT => self.a_alt = val,
            Reg8::B_ALT => self.b_alt = val,
            Reg8::C_ALT => self.c_alt = val,
            Reg8::D_ALT => self.d_alt = val,
            Reg8::E_ALT => self.e_alt = val,
            Reg8::H_ALT => self.h_alt = val,
            Reg8::L_ALT => self.l_alt = val,
            Reg8::F_ALT => self.f_alt = StatusIndicatorFlags::from_bits_truncate(val),
            Reg8::IXL => self.ix = (self.ix & 0xFF00) | val as u16,
            Reg8::IXH => self.ix = (self.ix & 0x00FF) | ((val as u16) << 8),
            Reg8::IYL => self.iy = (self.iy & 0xFF00) | val as u16,
            Reg8::IYH => self.iy = (self.iy & 0x00FF) | ((val as u16) << 8),
        }
    }

    pub fn read_reg16(&self, reg: Reg16) -> u16 {
        match reg {
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
                    Reg16::IR => (self.i, self.r),
                    Reg16::AF_ALT => (self.a_alt, self.f_alt.bits() as u8),
                    Reg16::BC_ALT => (self.b_alt, self.c_alt),
                    Reg16::DE_ALT => (self.d_alt, self.e_alt),
                    Reg16::HL_ALT => (self.h_alt, self.l_alt),
                    _ => unreachable!(),
                };
                (((high as u16) << 8) | low as u16)
            }
        }
    }

    pub fn write_reg16(&mut self, reg: Reg16, val: u16) {
        let (high, low) = (((val & 0xFF00) >> 8) as u8, (val & 0x00FF) as u8);
        match reg {
            Reg16::AF => { self.a = high; self.f = StatusIndicatorFlags::from_bits_truncate(low); }
            Reg16::BC => { self.b = high; self.c = low; }
            Reg16::DE => { self.d = high; self.e = low; }
            Reg16::HL => { self.h = high; self.l = low; }
            Reg16::IR => { self.i = high; self.r = low; }
            Reg16::AF_ALT => { self.a_alt = high; self.f_alt = StatusIndicatorFlags::from_bits_truncate(low); }
            Reg16::BC_ALT => { self.b_alt = high; self.c_alt = low; }
            Reg16::DE_ALT => { self.d_alt = high; self.e_alt = low; }
            Reg16::HL_ALT => { self.h_alt = high; self.l_alt = low; }
            Reg16::SP => self.sp = val,
            Reg16::IX => self.ix = val,
            Reg16::IY => self.iy = val,
            Reg16::WZ => self.wz = val,
        }
    }

    pub fn inc_pc(&mut self, val: u16) {
        self.pc += val;
    }
    pub fn dec_pc(&mut self, val: u16) {
        self.pc -= val;
    }
    pub fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn inc_r(&mut self, val: u8) {
        self.r = ((self.r.wrapping_add(val)) & 0b01111111) | (self.r & 0b10000000);
    }

    pub fn set_iff1(&mut self) {
        self.iff1 = true;
    }
    pub fn clear_iff1(&mut self) {
        self.iff1 = false;
    }
    pub fn get_iff1(&self) -> bool {
        self.iff1
    }
    pub fn set_iff2(&mut self) {
        self.iff2 = true;
    }
    pub fn clear_iff2(&mut self) {
        self.iff2 = false;
    }
    pub fn get_iff2(&self) -> bool {
        self.iff2
    }

    pub fn set_im(&mut self, val: u8) {
        self.im = val;
    }
    pub fn get_im(&self) -> u8 {
        self.im
    }

    pub fn halt(&mut self) {
        self.halted = true;
    }
    pub fn resume(&mut self) {
        self.halted = false;
    }
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn set_flag(&mut self, flag: StatusIndicatorFlags) {
        self.f.insert(flag);
    }
    pub fn clear_flag(&mut self, flag: StatusIndicatorFlags) {
        self.f.remove(flag);
    }
    pub fn get_flag(&self, flag: StatusIndicatorFlags) -> bool {
        self.f.contains(flag)
    }
    pub fn get_flags(&self) -> StatusIndicatorFlags {
        self.f
    }
    pub fn cond_flag(&mut self, flag: StatusIndicatorFlags, cond: bool) {
        if cond {
            self.f.insert(flag);
        } else {
            self.f.remove(flag);
        }
    }
    pub fn check_flags(&self, flags: StatusIndicatorFlags) -> bool {
        self.f == flags
    }

    pub fn check_cond(&self, cond: FlagCond) -> bool {
        match cond {
            FlagCond::NZ => !self.f.contains(ZERO_FLAG),
            FlagCond::Z  =>  self.f.contains(ZERO_FLAG),
            FlagCond::NC => !self.f.contains(CARRY_FLAG),
            FlagCond::C  =>  self.f.contains(CARRY_FLAG),
            FlagCond::PO => !self.f.contains(PARITY_OVERFLOW_FLAG),
            FlagCond::PE =>  self.f.contains(PARITY_OVERFLOW_FLAG),
            FlagCond::P  => !self.f.contains(SIGN_FLAG),
            FlagCond::M  =>  self.f.contains(SIGN_FLAG),
        }
    }

    pub fn handle_interrupts(&mut self) {
        if self.tcycles >= 70908 {
            self.tcycles -= 70908;

            if self.iff1 {
                if self.is_halted() {
                    self.inc_pc(1);
                    self.resume();
                }

                self.clear_iff1();
                self.clear_iff2();
                self.inc_r(1);
                self.tcycles += 7;

                let curr_pc = self.pc;
                let curr_sp = self.sp;
                self.write_word(curr_sp - 1, ((curr_pc & 0xFF00) >> 8) as u8);
                self.write_word(curr_sp - 2,  (curr_pc & 0x00FF)       as u8);
                self.sp -= 2;

                match self.im {
                    0 => {
                        self.pc = 0x0038;
                    }
                    1 => {
                        self.pc = 0x0038;
                    }
                    2 => {
                        let addr = 256u16 * (self.i as u16) + 256u16;
                        let low  = self.read_word(addr);
                        let high = self.read_word(addr + 1);
                        self.pc = ((high as u16) << 8 ) | low as u16;
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
        }
    }

    pub fn run_instruction(&mut self) {
        let i0 = self.fetch_op();

        match i0 {
            0xCB => {
                self.inc_pc(1);
                let i1 = self.fetch_op();
                self.inc_r(2);
                &INSTR_TABLE_CB[i1 as usize].execute(self);
            }
            0xDD => {
                self.inc_pc(1);
                let i1 = self.fetch_op();
                self.inc_r(2);
                match i1 {
                    0xCB => {
                        self.inc_pc(1);
                        let curr_pc = self.pc;
                        let i2 = self.read_word(curr_pc);
                        let i3 = self.read_word(curr_pc + 1);
                        self.contend_read_no_mreq(curr_pc + 1);
                        self.contend_read_no_mreq(curr_pc + 1);
                        let addr = ((self.read_reg16(Reg16::IX) as i16) + i2 as i16) as u16;
                        self.write_reg16(Reg16::WZ, addr);
                        &INSTR_TABLE_DDCB[i3 as usize].execute(self);
                    }
                    0xFD => {
                        self.inc_pc(1);
                    }
                    _ => {
                        &INSTR_TABLE_DD[i1 as usize].execute(self);
                    }
                };
            }
            0xED => {
                self.inc_pc(1);
                let i1 = self.fetch_op();
                self.inc_r(2);
                &INSTR_TABLE_ED[i1 as usize].execute(self);
            }
            0xFD => {
                self.inc_pc(1);
                let i1 = self.fetch_op();
                self.inc_r(2);
                match i1 {
                    0xCB => {
                        self.inc_pc(1);
                        let curr_pc = self.pc;
                        let i2 = self.read_word(curr_pc);
                        let i3 = self.read_word(curr_pc + 1);
                        self.contend_read_no_mreq(curr_pc + 1);
                        self.contend_read_no_mreq(curr_pc + 1);
                        let addr = ((self.read_reg16(Reg16::IY) as i16) + i2 as i16) as u16;
                        self.write_reg16(Reg16::WZ, addr);
                        &INSTR_TABLE_FDCB[i3 as usize].execute(self);
                    }
                    0xDD => {
                        self.inc_pc(1);
                    }
                    _ => {
                        &INSTR_TABLE_FD[i1 as usize].execute(self);
                    }
                };
            }
            _ => {
                self.inc_r(1);
                &INSTR_TABLE[i0 as usize].execute(self);
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.handle_interrupts();
            self.run_instruction();
        }
    }

    #[inline(always)]
    pub fn contend_read(&mut self, addr: u16, tcycles: u32) {
        self.tcycles += self.interconnect.contend_read(addr, self.tcycles, tcycles);
    }

    #[inline(always)]
    pub fn contend_read_no_mreq(&mut self, addr: u16) {
        self.tcycles += self.interconnect.contend_read_no_mreq(addr, self.tcycles);
    }

    #[inline(always)]
    pub fn contend_write_no_mreq(&mut self, addr: u16) {
        self.tcycles += self.interconnect.contend_write_no_mreq(addr, self.tcycles);
    }

    fn fetch_op(&mut self) -> u8 {
        let curr_pc = self.pc;
        self.contend_read(curr_pc, 4);
        self.interconnect.read_word(curr_pc, self.tcycles)
    }

    pub fn read_word(&mut self, addr: u16) -> u8 {
        self.contend_read(addr, 3);
        self.interconnect.read_word(addr, self.tcycles)
    }

    pub fn write_word(&mut self, addr: u16, val: u8) {
        self.contend_read(addr, 3);
        self.interconnect.write_word(addr, val, self.tcycles);
    }

    #[inline(always)]
    fn contend_port_early(&mut self, port: u16) {
        self.tcycles += self.interconnect.contend_port_early(port, self.tcycles);
    }

    #[inline(always)]
    fn contend_port_late(&mut self, port: u16) {
        self.tcycles += self.interconnect.contend_port_late(port, self.tcycles);
    }

    pub fn read_port(&mut self, port: u16) -> u8 {
        self.contend_port_early(port);

        let val = self.interconnect.read_port(port, self.tcycles);

        self.contend_port_late(port);

        val
    }

    pub fn write_port(&mut self, port: u16, val: u8) {
        self.contend_port_early(port);

        self.interconnect.write_port(port, val, self.tcycles);

        self.contend_port_late(port);
    }
}
