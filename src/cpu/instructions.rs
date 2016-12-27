use super::cpu::*;
use ::debugger::output_registers::*;


pub trait Instruction {
    fn execute(&self, &mut Cpu);
    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters);
}


pub struct Unsupported;

impl Instruction for Unsupported {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        panic!("Unsupported instruction {:#x} at address {:#06x}", cpu.read_word(curr_pc), curr_pc);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


pub struct Nop;

impl Instruction for Nop {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: NOP", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


pub struct AdcR  { pub r: Reg8 }
struct AdcN      ;
struct AdcMemHl  ;

#[inline(always)]
pub fn update_flags_adc8(cpu: &mut Cpu, op1: u8, op2: u8, c: u8, res: u8) {
    cpu.cond_flag  ( SIGN_FLAG            , res & 0x80 != 0                                          );
    cpu.cond_flag  ( ZERO_FLAG            , res == 0                                                 );
    cpu.cond_flag  ( HALF_CARRY_FLAG      , (op1 & 0x0F) + (op2 & 0x0F) + c > 0x0F                   );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , (op1 & 0x80 == op2 & 0x80) && (op1 & 0x80 != res & 0x80) );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                                               );
    cpu.cond_flag  ( CARRY_FLAG           , op1 as u16 + op2 as u16 + c as u16 > 0xFF                );
    cpu.cond_flag  ( X_FLAG               , res & 0x08 != 0                                          );
    cpu.cond_flag  ( Y_FLAG               , res & 0x20 != 0                                          );
}

impl Instruction for AdcR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(r).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, r, c, res);

        info!("{:#06x}: ADC A, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OA|OF)
    }
}

impl Instruction for AdcN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(n).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, n, c, res);

        info!("{:#06x}: ADC A, {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for AdcMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(memval).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, memval, c, res);

        info!("{:#06x}: ADC A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OH|OL, OA|OF)
    }
}


struct AddMemHl  ;
struct AddN      ;
pub struct AddR  { pub r: Reg8  }
struct AddHlSs   { r: Reg16 }

#[inline(always)]
pub fn update_flags_add8(cpu: &mut Cpu, op1: u8, op2: u8, res: u8) {
    cpu.cond_flag  ( SIGN_FLAG            , res & 0x80 != 0                                          );
    cpu.cond_flag  ( ZERO_FLAG            , res == 0                                                 );
    cpu.cond_flag  ( HALF_CARRY_FLAG      , (op1 & 0x0F) + (op2 & 0x0F) > 0x0F                       );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , (op1 & 0x80 == op2 & 0x80) && (op1 & 0x80 != res & 0x80) );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                                               );
    cpu.cond_flag  ( CARRY_FLAG           , op1 as u16 + op2 as u16 > 0xFF                           );
    cpu.cond_flag  ( X_FLAG               , res & 0x08 != 0                                          );
    cpu.cond_flag  ( Y_FLAG               , res & 0x20 != 0                                          );
}

#[inline(always)]
pub fn update_flags_add16(cpu: &mut Cpu, op1: u16, op2: u16, res: u16) {
    cpu.cond_flag  ( HALF_CARRY_FLAG   , (op1 & 0x0FFF) + (op2 & 0x0FFF) > 0x0FFF );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                            );
    cpu.cond_flag  ( CARRY_FLAG        , op1 as u32 + op2 as u32  > 0xFFFF        );
    cpu.cond_flag  ( X_FLAG            , (res >> 8) & 0x08 != 0                   );
    cpu.cond_flag  ( Y_FLAG            , (res >> 8) & 0x20 != 0                   );
}

impl Instruction for AddMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a.wrapping_add(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, memval, res);

        info!("{:#06x}: ADD A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for AddN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);

        let res = a.wrapping_add(n);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, n, res);

        info!("{:#06x}: ADD A, {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for AddR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a.wrapping_add(r);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, r, res);

        info!("{:#06x}: ADD A, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OA|OF)
    }
}

impl Instruction for AddHlSs {
    fn execute(&self, cpu: &mut Cpu) {
        let hl = cpu.read_reg16(Reg16::HL);
        let ss = cpu.read_reg16(self.r);

        let res = hl.wrapping_add(ss);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(Reg16::HL, res);
        cpu.write_reg16(Reg16::WZ, hl);

        update_flags_add16(cpu, hl, ss, res);

        info!("{:#06x}: ADD HL, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OF|OWZ|OutputRegisters::from(self.r), OH|OL|OF|OWZ)
    }
}


pub struct AndR  { pub r: Reg8 }
struct AndN      ;
struct AndMemHl  ;

#[inline(always)]
pub fn update_flags_logical(cpu: &mut Cpu, res: u8) {
    cpu.cond_flag  ( SIGN_FLAG            , res & 0x80 != 0           );
    cpu.cond_flag  ( ZERO_FLAG            , res == 0                  );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , res.count_ones() % 2 == 0 );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                );
    cpu.clear_flag ( CARRY_FLAG                                       );
    cpu.cond_flag  ( X_FLAG               , res & 0x08 != 0           );
    cpu.cond_flag  ( Y_FLAG               , res & 0x20 != 0           );
}

impl Instruction for AndR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a & r;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OA|OF)
    }
}

impl Instruction for AndN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);

        let res = a & n;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for AndMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a & memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


#[inline(always)]
pub fn update_flags_bit(cpu: &mut Cpu, b: u8, bit_is_set: bool) {
    cpu.cond_flag  ( SIGN_FLAG            , b == 7 && bit_is_set );
    cpu.cond_flag  ( ZERO_FLAG            , !bit_is_set          );
    cpu.set_flag   ( HALF_CARRY_FLAG                             );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , !bit_is_set          );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                           );
}

#[inline(always)]
pub fn update_xyflags_bit(cpu: &mut Cpu) {
    let wz = cpu.read_reg16(Reg16::WZ);

    cpu.cond_flag  ( X_FLAG, wz & 0x0800 != 0 );
    cpu.cond_flag  ( Y_FLAG, wz & 0x2000 != 0 );
}


struct CallNn   ;
struct CallCcNn { cond: FlagCond }

impl Instruction for CallNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let nn      =  (cpu.read_word(curr_pc + 1) as u16) |
                      ((cpu.read_word(curr_pc + 2) as u16) << 8);
        let curr_sp = cpu.read_reg16(Reg16::SP);

        cpu.contend_read_no_mreq(curr_pc + 2);

        cpu.write_word(curr_sp - 1, (((curr_pc + 3) & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  ((curr_pc + 3) & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);
        cpu.write_reg16(Reg16::WZ, nn);

        info!("{:#06x}: CALL {:#06X}", curr_pc, nn);
        cpu.set_pc(nn);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OWZ, OSP|OWZ)
    }
}

impl Instruction for CallCcNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let cc      = cpu.check_cond(self.cond);

        if cc {
            let nn      =  (cpu.read_word(curr_pc + 1) as u16) |
                          ((cpu.read_word(curr_pc + 2) as u16) << 8);
            let curr_sp = cpu.read_reg16(Reg16::SP);

            cpu.contend_read_no_mreq(curr_pc + 2);

            cpu.write_word(curr_sp - 1, (((curr_pc + 3) & 0xFF00) >> 8) as u8);
            cpu.write_word(curr_sp - 2,  ((curr_pc + 3) & 0x00FF)       as u8);

            cpu.write_reg16(Reg16::SP, curr_sp - 2);

            info!("{:#06x}: CALL {:?}, {:#06X}", curr_pc, self.cond, nn);
            cpu.set_pc(nn);
            cpu.write_reg16(Reg16::WZ, nn);
        } else {
            cpu.contend_read(curr_pc + 1, 3);
            cpu.contend_read(curr_pc + 2, 3);
            //TODO
            //let nn =  (cpu.read_word(curr_pc + 1) as u16) |
            //         ((cpu.read_word(curr_pc + 2) as u16) << 8);

            //info!("{:#06x}: CALL {:?}, {:#06X}", curr_pc, self.cond, nn);
            cpu.inc_pc(3);
            //cpu.write_reg16(Reg16::WZ, nn);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OF|OWZ, OSP|OWZ)
    }
}


struct Ccf;

impl Instruction for Ccf {
    fn execute(&self, cpu: &mut Cpu) {
        let c = cpu.get_flag(CARRY_FLAG);

        cpu.cond_flag  ( HALF_CARRY_FLAG   , c  );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG      );
        cpu.cond_flag  ( CARRY_FLAG        , !c );

        let a = cpu.read_reg8(Reg8::A);
        cpu.cond_flag ( X_FLAG, a & 0x08 != 0 );
        cpu.cond_flag ( Y_FLAG, a & 0x20 != 0 );

        info!("{:#06x}: CCF", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF, OF)
    }
}


pub struct CpR  { pub r: Reg8 }
struct CpN      ;
struct CpMemHl  ;

#[inline(always)]
pub fn update_flags_cp8(cpu: &mut Cpu, op1: u8, op2: u8, res: u8) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0                                          );
    cpu.cond_flag ( ZERO_FLAG            , res == 0                                                 );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (op1 & 0x0F) < (op2 & 0x0F)                              );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , (op1 & 0x80 != op2 & 0x80) && (op1 & 0x80 != res & 0x80) );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                                               );
    cpu.cond_flag ( CARRY_FLAG           , op1 < op2                                                );
    cpu.cond_flag ( X_FLAG               , op2 & 0x08 != 0                                          );
    cpu.cond_flag ( Y_FLAG               , op2 & 0x20 != 0                                          );
}

impl Instruction for CpR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a.wrapping_sub(r);

        update_flags_cp8(cpu, a, r, res);

        info!("{:#06x}: CP {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OF)
    }
}

impl Instruction for CpN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);

        let res = a.wrapping_sub(n);

        update_flags_cp8(cpu, a, n, res);

        info!("{:#06x}: CP {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OF)
    }
}

impl Instruction for CpMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a.wrapping_sub(memval);

        update_flags_cp8(cpu, a, memval, res);

        info!("{:#06x}: CP (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OH|OL, OF)
    }
}


struct Cpl;

impl Instruction for Cpl {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let res = a ^ 0xFF;

        cpu.set_flag  ( HALF_CARRY_FLAG                        );
        cpu.set_flag  ( ADD_SUBTRACT_FLAG                      );
        cpu.cond_flag ( X_FLAG               , res & 0x08 != 0 );
        cpu.cond_flag ( Y_FLAG               , res & 0x20 != 0 );

        cpu.write_reg8(Reg8::A, res);

        info!("{:#06x}: CPL", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


struct Daa;

impl Instruction for Daa {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let diff = match (cpu.get_flag(CARRY_FLAG),
                          (a & 0xF0) >> 4,
                          cpu.get_flag(HALF_CARRY_FLAG),
                           a & 0x0F) {
            (false, 0x0...0x9, false, 0x0...0x9) => 0x00,
            (false, 0x0...0x9, true , 0x0...0x9) => 0x06,
            (false, 0x0...0x8, _    , 0xA...0xF) => 0x06,
            (false, 0xA...0xF, false, 0x0...0x9) => 0x60,
            (true , _        , false, 0x0...0x9) => 0x60,
            (true , _        , true , 0x0...0x9) => 0x66,
            (true , _        , _    , 0xA...0xF) => 0x66,
            (false, 0x9...0xF, _    , 0xA...0xF) => 0x66,
            (false, 0xA...0xF, true , 0x0...0x9) => 0x66,
            _ => unreachable!()
        };

        let res = if cpu.get_flag(ADD_SUBTRACT_FLAG) {
            a.wrapping_sub(diff)
        } else {
            a.wrapping_add(diff)
        };
        cpu.write_reg8(Reg8::A, res);

        let h = match (cpu.get_flag(ADD_SUBTRACT_FLAG),
                       cpu.get_flag(HALF_CARRY_FLAG),
                       a & 0x0F) {
            (false, _    , 0x0...0x9) => false,
            (false, _    , 0xA...0xF) => true ,
            (true , false, _        ) => false,
            (true , true , 0x6...0xF) => false,
            (true , true , 0x0...0x5) => true ,
            _ => unreachable!()
        };

        let c = match (cpu.get_flag(CARRY_FLAG),
                       (a & 0xF0) >> 4,
                        a & 0x0F) {
            (false, 0x0...0x9, 0x0...0x9) => false,
            (false, 0x0...0x8, 0xA...0xF) => false,
            (false, 0x9...0xF, 0xA...0xF) => true ,
            (false, 0xA...0xF, 0x0...0x9) => true ,
            (true , _        , _        ) => true ,
            _ => unreachable!()
        };

        cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0           );
        cpu.cond_flag ( ZERO_FLAG            , res == 0                  );
        cpu.cond_flag ( HALF_CARRY_FLAG      , h                         );
        cpu.cond_flag ( PARITY_OVERFLOW_FLAG , res.count_ones() % 2 == 0 );
        cpu.cond_flag ( CARRY_FLAG           , c                         );
        cpu.cond_flag ( X_FLAG               , res & 0x08 != 0           );
        cpu.cond_flag ( Y_FLAG               , res & 0x20 != 0           );

        info!("{:#06x}: DAA", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


pub struct DecR  { pub r: Reg8  }
struct DecMemHl  ;
pub struct DecSs { pub r: Reg16 }

pub fn update_flags_dec8(cpu: &mut Cpu, op: u8, res: u8) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0  );
    cpu.cond_flag ( ZERO_FLAG            , res == 0         );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (op & 0x0F) == 0 );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , res == 0x7F      );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                       );
    cpu.cond_flag ( X_FLAG               , res & 0x08 != 0  );
    cpu.cond_flag ( Y_FLAG               , res & 0x20 != 0  );
}

impl Instruction for DecR {
    fn execute(&self, cpu: &mut Cpu) {
        let r   = cpu.read_reg8(self.r);
        let res = r.wrapping_sub(1);

        cpu.write_reg8(self.r, res);

        update_flags_dec8(cpu, r, res);

        info!("{:#06x}: DEC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for DecMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval.wrapping_sub(1);

        cpu.write_word(hl, res);

        update_flags_dec8(cpu, memval, res);

        info!("{:#06x}: DEC (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for DecSs {
    fn execute(&self, cpu: &mut Cpu) {
        let r   = cpu.read_reg16(self.r);
        let res = r.wrapping_sub(1);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(self.r, res);

        info!("{:#06x}: DEC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}


struct Di;

impl Instruction for Di {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.clear_iff1();
        cpu.clear_iff2();

        info!("{:#06x}: DI", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


struct Djnz;

impl Instruction for Djnz {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        let b = cpu.read_reg8(Reg8::B).wrapping_sub(1);
        cpu.write_reg8(Reg8::B, b);

        if b != 0 {
            let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);

            info!("{:#06x}: DJNZ {:#06X}", cpu.get_pc(), target);
            cpu.set_pc(target);
        } else {
            cpu.contend_read(curr_pc + 1, 3);
            //TODO
            //let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            //let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            //info!("{:#06x}: DJNZ {:#06X}", cpu.get_pc(), target);
            cpu.inc_pc(2);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB, OB)
    }
}


struct Ei;

impl Instruction for Ei {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_iff1();
        cpu.set_iff2();

        info!("{:#06x}: EI", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


struct ExAfAfAlt;
struct ExMemSpHl;
struct ExDeHl;

impl Instruction for ExAfAfAlt {
    fn execute(&self, cpu: &mut Cpu) {
        let af    = cpu.read_reg16(Reg16::AF);
        let afalt = cpu.read_reg16(Reg16::AF_ALT);

        cpu.write_reg16(Reg16::AF, afalt);
        cpu.write_reg16(Reg16::AF_ALT, af);

        info!("{:#06x}: EX AF, AF'", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OA_ALT|OF_ALT, OA|OF|OA_ALT|OF_ALT)
    }
}

impl Instruction for ExMemSpHl {
    fn execute(&self, cpu: &mut Cpu) {
        let sp = cpu.read_reg16(Reg16::SP);
        let hl = cpu.read_reg16(Reg16::HL);

        let (hlhigh, hllow) = (((hl & 0xFF00) >> 8) as u8,
                               ((hl & 0x00FF)       as u8));
        let memval = (cpu.read_word(sp    ) as u16) |
                    ((cpu.read_word(sp + 1) as u16) << 8);

        cpu.contend_read_no_mreq(sp + 1);

        cpu.write_reg16(Reg16::HL, memval);
        cpu.write_reg16(Reg16::WZ, memval);

        cpu.write_word(sp + 1, hlhigh);
        cpu.write_word(sp, hllow);

        cpu.contend_write_no_mreq(sp);
        cpu.contend_write_no_mreq(sp);

        info!("{:#06x}: EX (SP), HL", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OH|OL|OWZ, OH|OL|OWZ)
    }
}

impl Instruction for ExDeHl {
    fn execute(&self, cpu: &mut Cpu) {
        let de = cpu.read_reg16(Reg16::DE);
        let hl = cpu.read_reg16(Reg16::HL);

        cpu.write_reg16(Reg16::DE, hl);
        cpu.write_reg16(Reg16::HL, de);

        info!("{:#06x}: EX DE, HL", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OD|OE|OH|OL, OD|OE|OH|OL)
    }
}


struct Exx;

impl Instruction for Exx {
    fn execute(&self, cpu: &mut Cpu) {
        let bc = cpu.read_reg16(Reg16::BC);
        let de = cpu.read_reg16(Reg16::DE);
        let hl = cpu.read_reg16(Reg16::HL);

        let bcalt = cpu.read_reg16(Reg16::BC_ALT);
        let dealt = cpu.read_reg16(Reg16::DE_ALT);
        let hlalt = cpu.read_reg16(Reg16::HL_ALT);

        cpu.write_reg16(Reg16::BC, bcalt);
        cpu.write_reg16(Reg16::DE, dealt);
        cpu.write_reg16(Reg16::HL, hlalt);

        cpu.write_reg16(Reg16::BC_ALT, bc);
        cpu.write_reg16(Reg16::DE_ALT, de);
        cpu.write_reg16(Reg16::HL_ALT, hl);

        info!("{:#06x}: EXX", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB|OC|OD|OE|OH|OL|OB_ALT|OC_ALT|OD_ALT|OE_ALT|OH_ALT|OL_ALT,
         OB|OC|OD|OE|OH|OL|OB_ALT|OC_ALT|OD_ALT|OE_ALT|OH_ALT|OL_ALT)
    }
}


struct Halt;

impl Instruction for Halt {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: HALT", cpu.get_pc());
        cpu.halt();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


struct InAPortN ;

impl Instruction for InAPortN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let a = cpu.read_reg8(Reg8::A);

        let port = (cpu.read_word(curr_pc + 1) as u16) | ((a as u16) << 8);

        let portval = cpu.read_port(port);

        cpu.write_reg8(Reg8::A, portval);

        info!("{:#06x}: IN A, ({:#04X})", cpu.get_pc(), port);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA, OA)
    }
}


pub struct IncR  { pub r: Reg8  }
struct IncMemHl  ;
pub struct IncSs { pub r: Reg16 }

#[inline(always)]
pub fn update_flags_inc8(cpu: &mut Cpu, op: u8, res: u8) {
    cpu.cond_flag  ( SIGN_FLAG            , res & 0x80 != 0        );
    cpu.cond_flag  ( ZERO_FLAG            , res == 0               );
    cpu.cond_flag  ( HALF_CARRY_FLAG      , (op & 0x0F) + 1 > 0x0F );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , res == 0x80            );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                             );
    cpu.cond_flag  ( X_FLAG               , res & 0x08 != 0        );
    cpu.cond_flag  ( Y_FLAG               , res & 0x20 != 0        );
}

impl Instruction for IncR {
    fn execute(&self, cpu: &mut Cpu) {
        let r   = cpu.read_reg8(self.r);
        let res = r.wrapping_add(1);

        cpu.write_reg8(self.r, res);

        update_flags_inc8(cpu, r, res);

        info!("{:#06x}: INC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for IncMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl  = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval.wrapping_add(1);

        cpu.write_word(hl, res);

        update_flags_inc8(cpu, memval, res);

        info!("{:#06x}: INC (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF, OF)
    }
}

impl Instruction for IncSs {
    fn execute(&self, cpu: &mut Cpu) {
        let r   = cpu.read_reg16(self.r);
        let res = r.wrapping_add(1);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(self.r, res);

        info!("{:#06x}: INC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}


struct JpMemHl;
struct JpNn   ;
struct JpCcNn { cond: FlagCond }

impl Instruction for JpMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl = cpu.read_reg16(Reg16::HL);

        info!("{:#06x}: JP (HL)", cpu.get_pc());
        cpu.set_pc(hl);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, ONONE)
    }
}

impl Instruction for JpNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);

        info!("{:#06x}: JP {:#06X}", cpu.get_pc(), nn);
        cpu.set_pc(nn);
        cpu.write_reg16(Reg16::WZ, nn);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OWZ, OWZ)
    }
}

impl Instruction for JpCcNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let cc = cpu.check_cond(self.cond);

        if cc {
            let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                     ((cpu.read_word(curr_pc + 2) as u16) << 8);

            info!("{:#06x}: JP {:?}, {:#06X}", cpu.get_pc(), self.cond, nn);
            cpu.set_pc(nn);

            cpu.write_reg16(Reg16::WZ, nn);
        } else {
            cpu.contend_read(curr_pc + 1, 3);
            cpu.contend_read(curr_pc + 2, 3);
            //TODO
            //let nn =  (cpu.read_word(curr_pc + 1) as u16) |
            //         ((cpu.read_word(curr_pc + 2) as u16) << 8);

            //info!("{:#06x}: JP {:?}, {:#06X}", cpu.get_pc(), self.cond, nn);
            cpu.inc_pc(3);

            //cpu.write_reg16(Reg16::WZ, nn);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OWZ, OWZ)
    }
}


struct JrZ ;
struct JrNz;
struct JrNcE;
struct JrCE;
struct JrE ;

impl Instruction for JrZ {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        if cpu.get_flag(ZERO_FLAG) {
            let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);

            info!("{:#06x}: JR Z, {:#06X}", cpu.get_pc(), target);
            cpu.set_pc(target);
            cpu.write_reg16(Reg16::WZ, target);
        } else {
            cpu.contend_read(curr_pc + 1, 3);
            //TODO
            //let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            //let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            //info!("{:#06x}: JR Z, {:#06X}", cpu.get_pc(), target);
            cpu.inc_pc(2);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OWZ, OWZ)
    }
}

impl Instruction for JrNz {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        if cpu.get_flag(ZERO_FLAG) {
            cpu.contend_read(curr_pc + 1, 3);
            //TODO
            //let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            //let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            //info!("{:#06x}: JR NZ, {:#06X}", cpu.get_pc(), target);
            cpu.inc_pc(2);
        } else {
            let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);

            info!("{:#06x}: JR NZ, {:#06X}", cpu.get_pc(), target);
            cpu.set_pc(target);
            cpu.write_reg16(Reg16::WZ, target);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OWZ, OWZ)
    }
}

impl Instruction for JrNcE {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        if cpu.get_flag(CARRY_FLAG) {
            cpu.contend_read(curr_pc + 1, 3);
            //TODO
            //let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            //let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            //info!("{:#06x}: JR NC, {:#06X}", cpu.get_pc(), target);
            cpu.inc_pc(2);
        } else {
            let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);

            info!("{:#06x}: JR NC, {:#06X}", cpu.get_pc(), target);
            cpu.set_pc(target);
            cpu.write_reg16(Reg16::WZ, target);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OWZ, OWZ)
    }
}

impl Instruction for JrCE {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        if cpu.get_flag(CARRY_FLAG) {
            let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);
            cpu.contend_read_no_mreq(curr_pc + 1);

            info!("{:#06x}: JR C, {:#06X}", cpu.get_pc(), target);
            cpu.set_pc(target);
            cpu.write_reg16(Reg16::WZ, target);
        } else {
            cpu.contend_read(curr_pc + 1, 3);
            //TODO
            //let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
            //let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            //info!("{:#06x}: JR C, {:#06X}", cpu.get_pc(), target);
            cpu.inc_pc(2);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OWZ, OWZ)
    }
}

impl Instruction for JrE {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (cpu.get_pc() as i16 + offset as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        info!("{:#06x}: JR {:#06X}", cpu.get_pc(), target);
        cpu.set_pc(target);
        cpu.write_reg16(Reg16::WZ, target);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OWZ, OWZ)
    }
}


struct LdMemBcA   ;
struct LdMemDeA   ;
struct LdMemHlN   ;
struct LdMemHlR   { r: Reg8  }
struct LdMemNnA   ;
struct LdMemNnHl  ;
struct LdAMemBc   ;
struct LdAMemDe   ;
struct LdAMemNn   ;
pub struct LdDdNn { pub r: Reg16 }
pub struct LdRN   { pub r: Reg8  }
struct LdHlMemNn  ;
struct LdSpHl     ;
pub struct LdRR   { pub rt: Reg8, pub rs: Reg8 }
struct LdRMemHl   { r: Reg8  }

impl Instruction for LdMemBcA {
    fn execute(&self, cpu: &mut Cpu) {
        let bc = cpu.read_reg16(Reg16::BC);
        let a  = cpu.read_reg8(Reg8::A);

        cpu.write_word(bc, a);
        cpu.write_reg16(Reg16::WZ, ((a as u16) << 8) | ((bc + 1) & 0x00FF));

        info!("{:#06x}: LD (BC), A", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OB|OC|OWZ, OWZ)
    }
}

impl Instruction for LdMemDeA {
    fn execute(&self, cpu: &mut Cpu) {
        let de = cpu.read_reg16(Reg16::DE);
        let a  = cpu.read_reg8(Reg8::A);

        cpu.write_word(de, a);
        cpu.write_reg16(Reg16::WZ, ((a as u16) << 8) | ((de + 1) & 0x00FF));

        info!("{:#06x}: LD (DE), A", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OD|OE|OWZ, OWZ)
    }
}

impl Instruction for LdMemHlN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let hl = cpu.read_reg16(Reg16::HL);
        let n  = cpu.read_word(curr_pc + 1);

        cpu.write_word(hl, n);

        info!("{:#06x}: LD (HL), {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, ONONE)
    }
}

impl Instruction for LdMemHlR {
    fn execute(&self, cpu: &mut Cpu) {
        let hl = cpu.read_reg16(Reg16::HL);
        let r  = cpu.read_reg8(self.r);

        cpu.write_word(hl, r);

        info!("{:#06x}: LD (HL), {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OutputRegisters::from(self.r), ONONE)
    }
}

impl Instruction for LdMemNnA {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a  = cpu.read_reg8(Reg8::A);
        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);

        cpu.write_word(nn, a);
        cpu.write_reg16(Reg16::WZ, ((a as u16) << 8) | ((nn + 1) & 0x00FF));

        info!("{:#06x}: LD ({:#06X}), A", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OWZ, OWZ)
    }
}

impl Instruction for LdMemNnHl {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let hl = cpu.read_reg16(Reg16::HL);
        let (hlhigh, hllow) = (((hl & 0xFF00) >> 8) as u8,
                               ((hl & 0x00FF)       as u8));
        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);

        cpu.write_word(nn, hllow);
        cpu.write_word(nn + 1, hlhigh);

        info!("{:#06x}: LD ({:#06X}), HL", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, ONONE)
    }
}

impl Instruction for LdAMemBc {
    fn execute(&self, cpu: &mut Cpu) {
        let bc     = cpu.read_reg16(Reg16::BC);
        let memval = cpu.read_word(bc);

        cpu.write_reg8(Reg8::A, memval);
        cpu.write_reg16(Reg16::WZ, bc + 1);

        info!("{:#06x}: LD A, (BC)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OB|OC|OWZ, OA|OWZ)
    }
}

impl Instruction for LdAMemDe {
    fn execute(&self, cpu: &mut Cpu) {
        let de     = cpu.read_reg16(Reg16::DE);
        let memval = cpu.read_word(de);

        cpu.write_reg8(Reg8::A, memval);
        cpu.write_reg16(Reg16::WZ, de + 1);

        info!("{:#06x}: LD A, (DE)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OD|OE|OWZ, OA|OWZ)
    }
}

impl Instruction for LdAMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);
        let memval = cpu.read_word(nn);

        cpu.write_reg8(Reg8::A, memval);
        cpu.write_reg16(Reg16::WZ, nn + 1);

        info!("{:#06x}: LD A, ({:#06X})", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OWZ, OA|OWZ)
    }
}

impl Instruction for LdDdNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);

        cpu.write_reg16(self.r, nn);

        info!("{:#06x}: LD {:?}, {:#06X}", cpu.get_pc(), self.r, nn);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}

impl Instruction for LdRN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let n = cpu.read_word(curr_pc + 1);

        cpu.write_reg8(self.r, n);

        info!("{:#06x}: LD {:?}, {:#04X}", cpu.get_pc(), self.r, n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}

impl Instruction for LdHlMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);
        let nnmemval = (cpu.read_word(nn    ) as u16) |
                      ((cpu.read_word(nn + 1) as u16) << 8);

        cpu.write_reg16(Reg16::HL, nnmemval);

        info!("{:#06x}: LD HL, ({:#06X})", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, OH|OL)
    }
}

impl Instruction for LdSpHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl = cpu.read_reg16(Reg16::HL);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(Reg16::SP, hl);

        info!("{:#06x}: LD SP, HL", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OH|OL, OSP)
    }
}

impl Instruction for LdRR {
    fn execute(&self, cpu: &mut Cpu) {
        let rs = cpu.read_reg8(self.rs);

        cpu.write_reg8(self.rt, rs);

        info!("{:#06x}: LD {:?}, {:?}", cpu.get_pc(), self.rt, self.rs);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.rt) | OutputRegisters::from(self.rs),
         OutputRegisters::from(self.rt) | OutputRegisters::from(self.rt))
    }
}

impl Instruction for LdRMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.write_reg8(self.r, memval);

        info!("{:#06x}: LD {:?}, (HL)", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}


pub struct OrR  { pub r: Reg8 }
struct OrN      ;
struct OrMemHl  ;

impl Instruction for OrR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a | r;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OA|OF)
    }
}

impl Instruction for OrN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);

        let res = a | n;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for OrMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a | memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OH|OL, OA|OF)
    }
}


struct OutPortNA ;

impl Instruction for OutPortNA {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let a    = cpu.read_reg8(Reg8::A);

        let port = (cpu.read_word(curr_pc + 1) as u16) | ((a as u16) << 8);

        cpu.write_port(port as u16, a);

        info!("{:#06x}: OUT ({:#04X}), A", cpu.get_pc(), port);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA, ONONE)
    }
}


pub struct PopQq { pub r: Reg16 }

impl Instruction for PopQq {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low  = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16(self.r, ((high as u16) << 8 ) | low as u16);
        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        info!("{:#06x}: POP {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OutputRegisters::from(self.r), OSP|OutputRegisters::from(self.r))
    }
}


pub struct PushQq { pub r: Reg16 }

impl Instruction for PushQq {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);
        let r = cpu.read_reg16(self.r);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        cpu.write_word(curr_sp - 1, ((r & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (r & 0x00FF)       as u8);
        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        info!("{:#06x}: PUSH {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OutputRegisters::from(self.r), OSP)
    }
}


struct Ret   ;
struct RetCc { cond: FlagCond }

impl Instruction for Ret {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low  = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        info!("{:#06x}: RET", cpu.get_pc());
        cpu.set_pc(((high as u16) << 8 ) | low as u16);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP, OSP)
    }
}

impl Instruction for RetCc {
    fn execute(&self, cpu: &mut Cpu) {
        let cc = cpu.check_cond(self.cond);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        info!("{:#06x}: RET {:?}", cpu.get_pc(), self.cond);
        if cc {
            let curr_sp = cpu.read_reg16(Reg16::SP);

            let low  = cpu.read_word(curr_sp);
            let high = cpu.read_word(curr_sp + 1);

            cpu.write_reg16(Reg16::SP, curr_sp + 2);

            cpu.set_pc(((high as u16) << 8 ) | low as u16);
        } else {
            cpu.inc_pc(1);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OSP, OSP)
    }
}


struct RlA        ;
struct RlcA       ;

impl Instruction for RlA {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let mut res = a.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_reg8(Reg8::A, res);

        cpu.clear_flag ( HALF_CARRY_FLAG                     );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                   );
        cpu.cond_flag  ( CARRY_FLAG        , a & 0x80 != 0   );
        cpu.cond_flag  ( X_FLAG            , res & 0x08 != 0 );
        cpu.cond_flag  ( Y_FLAG            , res & 0x20 != 0 );

        info!("{:#06x}: RLA", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for RlcA {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let res = a.rotate_left(1);

        cpu.write_reg8(Reg8::A, res);

        cpu.clear_flag ( HALF_CARRY_FLAG                     );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                   );
        cpu.cond_flag  ( CARRY_FLAG        , a & 0x80 != 0   );
        cpu.cond_flag  ( X_FLAG            , res & 0x08 != 0 );
        cpu.cond_flag  ( Y_FLAG            , res & 0x20 != 0 );

        info!("{:#06x}: RLCA", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


struct RrA        ;
struct RrcA       ;

impl Instruction for RrA {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let mut res = a.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_reg8(Reg8::A, res);

        cpu.clear_flag ( HALF_CARRY_FLAG                     );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                   );
        cpu.cond_flag  ( CARRY_FLAG        , a & 0x01 != 0   );
        cpu.cond_flag  ( X_FLAG            , res & 0x08 != 0 );
        cpu.cond_flag  ( Y_FLAG            , res & 0x20 != 0 );

        info!("{:#06x}: RRA", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for RrcA {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let res = a.rotate_right(1);

        cpu.write_reg8(Reg8::A, res);

        cpu.clear_flag ( HALF_CARRY_FLAG                     );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                   );
        cpu.cond_flag  ( CARRY_FLAG        , a & 0x01 != 0   );
        cpu.cond_flag  ( X_FLAG            , res & 0x08 != 0 );
        cpu.cond_flag  ( Y_FLAG            , res & 0x20 != 0 );

        info!("{:#06x}: RRCA", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


struct Rst { addr: u8 }

impl Instruction for Rst {
    fn execute(&self, cpu: &mut Cpu) {
        let next_pc = cpu.get_pc() + 1;
        let curr_sp = cpu.read_reg16(Reg16::SP);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        cpu.write_word(curr_sp - 1, ((next_pc & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (next_pc & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        info!("{:#06x}: RST {:#04X}", cpu.get_pc(), self.addr);
        cpu.set_pc(self.addr as u16);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP, OSP)
    }
}


struct Scf;

impl Instruction for Scf {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_flag   ( CARRY_FLAG        );
        cpu.clear_flag ( HALF_CARRY_FLAG   );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG );

        let a = cpu.read_reg8(Reg8::A);
        cpu.cond_flag ( X_FLAG , a & 0x08 != 0 );
        cpu.cond_flag ( Y_FLAG , a & 0x20 != 0 );

        info!("{:#06x}: SCF", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF, OF)
    }
}


pub struct SbcR  { pub r: Reg8 }
struct SbcN      ;
struct SbcMemHl  ;

#[inline(always)]
pub fn update_flags_sbc8(cpu: &mut Cpu, op1: u8, op2: u8, c: u8, res: u8) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0                                          );
    cpu.cond_flag ( ZERO_FLAG            , res == 0                                                 );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (op1 & 0x0F) < (op2 & 0x0F) + c                          );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , (op1 & 0x80 != op2 & 0x80) && (op1 & 0x80 != res & 0x80) );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                                               );
    cpu.cond_flag ( CARRY_FLAG           , (op1 as u16) < ((op2 as u16) + (c as u16))               );
    cpu.cond_flag ( X_FLAG               , res & 0x08 != 0                                          );
    cpu.cond_flag ( Y_FLAG               , res & 0x20 != 0                                          );
}

impl Instruction for SbcR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(r).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, r, c, res);

        info!("{:#06x}: SBC A, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OA|OF)
    }
}

impl Instruction for SbcN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(n).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, n, c, res);

        info!("{:#06x}: SBC A, {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for SbcMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(memval).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, memval, c, res);

        info!("{:#06x}: SBC A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OH|OL, OA|OF)
    }
}


pub struct SubR  { pub r: Reg8 }
struct SubN      ;
struct SubMemHl  ;

#[inline(always)]
pub fn update_flags_sub8(cpu: &mut Cpu, op1: u8, op2: u8, res: u8) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0                                          );
    cpu.cond_flag ( ZERO_FLAG            , res == 0                                                 );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (op1 & 0x0F) < (op2 & 0x0F)                              );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , (op1 & 0x80 != op2 & 0x80) && (op1 & 0x80 != res & 0x80) );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                                               );
    cpu.cond_flag ( CARRY_FLAG           , (op1 as u16) < (op2 as u16)                              );
    cpu.cond_flag ( X_FLAG               , res & 0x08 != 0                                          );
    cpu.cond_flag ( Y_FLAG               , res & 0x20 != 0                                          );
}

impl Instruction for SubR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a.wrapping_sub(r);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, r, res);

        info!("{:#06x}: SUB {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OA|OF)
    }
}

impl Instruction for SubN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);

        let res = a.wrapping_sub(n);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, n, res);

        info!("{:#06x}: SUB {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for SubMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a.wrapping_sub(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, memval, res);

        info!("{:#06x}: SUB A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


pub struct XorR  { pub r: Reg8 }
struct XorN      ;
struct XorMemHl  ;

impl Instruction for XorR {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a ^ r;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OutputRegisters::from(self.r), OA|OF)
    }
}

impl Instruction for XorN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(curr_pc + 1);

        let res = a ^ n;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}

impl Instruction for XorMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a ^ memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


pub const INSTR_TABLE: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */             /* 0x02 */    /* 0x03 */           /* 0x04 */        /* 0x05 */        /* 0x06 */        /* 0x07 */
    &Nop        , &LdDdNn{r:Reg16::BC} , &LdMemBcA   , &IncSs{r:Reg16::BC}, &IncR{r:Reg8::B}, &DecR{r:Reg8::B}, &LdRN{r:Reg8::B}, &RlcA       ,

    /* 0x08 */    /* 0x09 */             /* 0x0A */    /* 0x0B */           /* 0x0C */        /* 0x0D */        /* 0x0E */        /* 0x0F */
    &ExAfAfAlt  , &AddHlSs{r:Reg16::BC}, &LdAMemBc   , &DecSs{r:Reg16::BC}, &IncR{r:Reg8::C}, &DecR{r:Reg8::C}, &LdRN{r:Reg8::C}, &RrcA       ,

    /* 0x10 */    /* 0x11 */             /* 0x12 */    /* 0x13 */           /* 0x14 */        /* 0x15 */        /* 0x16 */        /* 0x17 */
    &Djnz       , &LdDdNn{r:Reg16::DE} , &LdMemDeA   , &IncSs{r:Reg16::DE}, &IncR{r:Reg8::D}, &DecR{r:Reg8::D}, &LdRN{r:Reg8::D}, &RlA        ,

    /* 0x18 */    /* 0x19 */             /* 0x1A */    /* 0x1B */           /* 0x1C */        /* 0x1D */        /* 0x1E */        /* 0x1F */
    &JrE        , &AddHlSs{r:Reg16::DE}, &LdAMemDe   , &DecSs{r:Reg16::DE}, &IncR{r:Reg8::E}, &DecR{r:Reg8::E}, &LdRN{r:Reg8::E}, &RrA        ,

    /* 0x20 */    /* 0x21 */             /* 0x22 */    /* 0x23 */           /* 0x24 */        /* 0x25 */        /* 0x26 */        /* 0x27 */
    &JrNz       , &LdDdNn{r:Reg16::HL} , &LdMemNnHl  , &IncSs{r:Reg16::HL}, &IncR{r:Reg8::H}, &DecR{r:Reg8::H}, &LdRN{r:Reg8::H}, &Daa        ,

    /* 0x28 */    /* 0x29 */             /* 0x2A */    /* 0x2B */           /* 0x2C */        /* 0x2D */        /* 0x2E */        /* 0x2F */
    &JrZ        , &AddHlSs{r:Reg16::HL}, &LdHlMemNn  , &DecSs{r:Reg16::HL}, &IncR{r:Reg8::L}, &DecR{r:Reg8::L}, &LdRN{r:Reg8::L}, &Cpl        ,

    /* 0x30 */    /* 0x31 */             /* 0x32 */    /* 0x33 */           /* 0x34 */        /* 0x35 */        /* 0x36 */        /* 0x37 */
    &JrNcE      , &LdDdNn{r:Reg16::SP} , &LdMemNnA   , &IncSs{r:Reg16::SP}, &IncMemHl       , &DecMemHl       , &LdMemHlN       , &Scf        ,

    /* 0x38 */    /* 0x39 */             /* 0x3A */    /* 0x3B */           /* 0x3C */        /* 0x3D */        /* 0x3E */        /* 0x3F */
    &JrCE       , &AddHlSs{r:Reg16::SP}, &LdAMemNn   , &DecSs{r:Reg16::SP}, &IncR{r:Reg8::A}, &DecR{r:Reg8::A}, &LdRN{r:Reg8::A}, &Ccf        ,

    /* 0x40 */                                 /* 0x41 */                               /* 0x42 */                          /* 0x43 */
    &LdRR{rt:Reg8::B,rs:Reg8::B}             , &LdRR{rt:Reg8::B,rs:Reg8::C}           , &LdRR{rt:Reg8::B,rs:Reg8::D}      , &LdRR{rt:Reg8::B,rs:Reg8::E},

    /* 0x44 */                                 /* 0x45 */                               /* 0x46 */                          /* 0x47 */
    &LdRR{rt:Reg8::B,rs:Reg8::H}             , &LdRR{rt:Reg8::B,rs:Reg8::L}           , &LdRMemHl{r:Reg8::B}              , &LdRR{rt:Reg8::B,rs:Reg8::A},

    /* 0x48 */                                 /* 0x49 */                               /* 0x4A */                          /* 0x4B */
    &LdRR{rt:Reg8::C,rs:Reg8::B}             , &LdRR{rt:Reg8::C,rs:Reg8::C}           , &LdRR{rt:Reg8::C,rs:Reg8::D}      , &LdRR{rt:Reg8::C,rs:Reg8::E},

    /* 0x4C */                                 /* 0x4D */                               /* 0x4E */                          /* 0x4F */
    &LdRR{rt:Reg8::C,rs:Reg8::H}             , &LdRR{rt:Reg8::C,rs:Reg8::L}           , &LdRMemHl{r:Reg8::C}              , &LdRR{rt:Reg8::C,rs:Reg8::A},

    /* 0x50 */                                 /* 0x51 */                               /* 0x52 */                          /* 0x53 */
    &LdRR{rt:Reg8::D,rs:Reg8::B}             , &LdRR{rt:Reg8::D,rs:Reg8::C}           , &LdRR{rt:Reg8::D,rs:Reg8::D}      , &LdRR{rt:Reg8::D,rs:Reg8::E},

    /* 0x54 */                                 /* 0x55 */                               /* 0x56 */                          /* 0x57 */
    &LdRR{rt:Reg8::D,rs:Reg8::H}             , &LdRR{rt:Reg8::D,rs:Reg8::L}           , &LdRMemHl{r:Reg8::D}              , &LdRR{rt:Reg8::D,rs:Reg8::A},

    /* 0x58 */                                 /* 0x59 */                               /* 0x5A */                          /* 0x5B */
    &LdRR{rt:Reg8::E,rs:Reg8::B}             , &LdRR{rt:Reg8::E,rs:Reg8::C}           , &LdRR{rt:Reg8::E,rs:Reg8::D}      , &LdRR{rt:Reg8::E,rs:Reg8::E},

    /* 0x5C */                                 /* 0x5D */                               /* 0x5E */                          /* 0x5F */
    &LdRR{rt:Reg8::E,rs:Reg8::H}             , &LdRR{rt:Reg8::E,rs:Reg8::L}           , &LdRMemHl{r:Reg8::E}              , &LdRR{rt:Reg8::E,rs:Reg8::A},

    /* 0x60 */                                 /* 0x61 */                               /* 0x62 */                          /* 0x63 */
    &LdRR{rt:Reg8::H,rs:Reg8::B}             , &LdRR{rt:Reg8::H,rs:Reg8::C}           , &LdRR{rt:Reg8::H,rs:Reg8::D}      , &LdRR{rt:Reg8::H,rs:Reg8::E},

    /* 0x64 */                                 /* 0x65 */                               /* 0x66 */                          /* 0x67 */
    &LdRR{rt:Reg8::H,rs:Reg8::H}             , &LdRR{rt:Reg8::H,rs:Reg8::L}           , &LdRMemHl{r:Reg8::H}              , &LdRR{rt:Reg8::H,rs:Reg8::A},

    /* 0x68 */                                 /* 0x69 */                               /* 0x6A */                          /* 0x6B */
    &LdRR{rt:Reg8::L,rs:Reg8::B}             , &LdRR{rt:Reg8::L,rs:Reg8::C}           , &LdRR{rt:Reg8::L,rs:Reg8::D}      , &LdRR{rt:Reg8::L,rs:Reg8::E},

    /* 0x6C */                                 /* 0x6D */                               /* 0x6E */                          /* 0x6F */
    &LdRR{rt:Reg8::L,rs:Reg8::H}             , &LdRR{rt:Reg8::L,rs:Reg8::L}           , &LdRMemHl{r:Reg8::L}              , &LdRR{rt:Reg8::L,rs:Reg8::A},

    /* 0x70 */                                 /* 0x71 */                               /* 0x72 */                          /* 0x73 */
    &LdMemHlR{r:Reg8::B}                     , &LdMemHlR{r:Reg8::C}                   , &LdMemHlR{r:Reg8::D}              , &LdMemHlR{r:Reg8::E},

    /* 0x74 */                                 /* 0x75 */                               /* 0x76 */                          /* 0x77 */
    &LdMemHlR{r:Reg8::H}                     , &LdMemHlR{r:Reg8::L}                   , &Halt                             , &LdMemHlR{r:Reg8::A},

    /* 0x78 */                                 /* 0x79 */                               /* 0x7A */                           /* 0x7B */
    &LdRR{rt:Reg8::A,rs:Reg8::B}             , &LdRR{rt:Reg8::A,rs:Reg8::C}           , &LdRR{rt:Reg8::A,rs:Reg8::D}      , &LdRR{rt:Reg8::A,rs:Reg8::E},

    /* 0x7C */                                 /* 0x7D */                               /* 0x7E */                          /* 0x7F */
    &LdRR{rt:Reg8::A,rs:Reg8::H}             , &LdRR{rt:Reg8::A,rs:Reg8::L}           , &LdRMemHl{r:Reg8::A}              , &LdRR{rt:Reg8::A,rs:Reg8::A},

    /* 0x80 */         /* 0x81 */         /* 0x82 */         /* 0x83 */         /* 0x84 */         /* 0x85 */         /* 0x86 */    /* 0x87 */
    &AddR{r:Reg8::B} , &AddR{r:Reg8::C} , &AddR{r:Reg8::D} , &AddR{r:Reg8::E} , &AddR{r:Reg8::H} , &AddR{r:Reg8::L} , &AddMemHl   , &AddR{r:Reg8::A},

    /* 0x88 */         /* 0x89 */         /* 0x8A */         /* 0x8B */         /* 0x8C */         /* 0x8D */         /* 0x8E */    /* 0x8F */
    &AdcR{r:Reg8::B} , &AdcR{r:Reg8::C} , &AdcR{r:Reg8::D} , &AdcR{r:Reg8::E} , &AdcR{r:Reg8::H} , &AdcR{r:Reg8::L} , &AdcMemHl   , &AdcR{r:Reg8::A} ,

    /* 0x90 */         /* 0x91 */         /* 0x92 */         /* 0x93 */         /* 0x94 */         /* 0x95 */         /* 0x96 */    /* 0x97 */
    &SubR{r:Reg8::B} , &SubR{r:Reg8::C} , &SubR{r:Reg8::D} , &SubR{r:Reg8::E} , &SubR{r:Reg8::H} , &SubR{r:Reg8::L} , &SubMemHl   , &SubR{r:Reg8::A} ,

    /* 0x98 */         /* 0x99 */         /* 0x9A */         /* 0x9B */         /* 0x9C */         /* 0x9D */         /* 0x9E */    /* 0x9F */
    &SbcR{r:Reg8::B} , &SbcR{r:Reg8::C} , &SbcR{r:Reg8::D} , &SbcR{r:Reg8::E} , &SbcR{r:Reg8::H} , &SbcR{r:Reg8::L} , &SbcMemHl   , &SbcR{r:Reg8::A} ,

    /* 0xA0 */         /* 0xA1 */         /* 0xA2 */         /* 0xA3 */         /* 0xA4 */         /* 0xA5 */         /* 0xA6 */    /* 0xA7 */
    &AndR{r:Reg8::B} , &AndR{r:Reg8::C} , &AndR{r:Reg8::D} , &AndR{r:Reg8::E} , &AndR{r:Reg8::H} , &AndR{r:Reg8::L} , &AndMemHl   , &AndR{r:Reg8::A} ,

    /* 0xA8 */         /* 0xA9 */         /* 0xAA */         /* 0xAB */         /* 0xAC */         /* 0xAD */         /* 0xAE */    /* 0xAF */
    &XorR{r:Reg8::B} , &XorR{r:Reg8::C} , &XorR{r:Reg8::D} , &XorR{r:Reg8::E} , &XorR{r:Reg8::H} , &XorR{r:Reg8::L} , &XorMemHl   , &XorR{r:Reg8::A} ,

    /* 0xB0 */         /* 0xB1 */         /* 0xB2 */         /* 0xB3 */         /* 0xB4 */         /* 0xB5 */         /* 0xB6 */    /* 0xB7 */
    &OrR{r:Reg8::B}  , &OrR{r:Reg8::C}  , &OrR{r:Reg8::D}  , &OrR{r:Reg8::E}  , &OrR{r:Reg8::H}  , &OrR{r:Reg8::L}  , &OrMemHl    , &OrR{r:Reg8::A}  ,

    /* 0xB8 */         /* 0xB9 */         /* 0xBA */         /* 0xBB */         /* 0xBC */         /* 0xBD */         /* 0xBE */    /* 0xBF */
    &CpR{r:Reg8::B}  , &CpR{r:Reg8::C}  , &CpR{r:Reg8::D}  , &CpR{r:Reg8::E}  , &CpR{r:Reg8::H}  , &CpR{r:Reg8::L}  , &CpMemHl    , &CpR{r:Reg8::A}  ,

    /* 0xC0 */                 /* 0xC1 */           /* 0xC2 */                  /* 0xC3 */    /* 0xC4 */                    /* 0xC5 */            /* 0xC6 */    /* 0xC7 */
    &RetCc{cond:FlagCond::NZ}, &PopQq{r:Reg16::BC}, &JpCcNn{cond:FlagCond::NZ}, &JpNn       , &CallCcNn{cond:FlagCond::NZ}, &PushQq{r:Reg16::BC}, &AddN       , &Rst{addr:0x00},

    /* 0xC8 */                 /* 0xC9 */           /* 0xCA */                  /* 0xCB */    /* 0xCC */                    /* 0xCD */            /* 0xCE */    /* 0xCF */
    &RetCc{cond:FlagCond::Z} , &Ret               , &JpCcNn{cond:FlagCond::Z} , &Unsupported, &CallCcNn{cond:FlagCond::Z} , &CallNn             , &AdcN       , &Rst{addr:0x08},

    /* 0xD0 */                 /* 0xD1 */           /* 0xD2 */                  /* 0xD3 */    /* 0xD4 */                    /* 0xD5 */            /* 0xD6 */    /* 0xD7 */
    &RetCc{cond:FlagCond::NC}, &PopQq{r:Reg16::DE}, &JpCcNn{cond:FlagCond::NC}, &OutPortNA  , &CallCcNn{cond:FlagCond::NC}, &PushQq{r:Reg16::DE}, &SubN       , &Rst{addr:0x10},

    /* 0xD8 */                 /* 0xD9 */           /* 0xDA */                  /* 0xDB */    /* 0xDC */                    /* 0xDD */            /* 0xDE */    /* 0xDF */
    &RetCc{cond:FlagCond::C} , &Exx               , &JpCcNn{cond:FlagCond::C} , &InAPortN   , &CallCcNn{cond:FlagCond::C} , &Unsupported        , &SbcN       , &Rst{addr:0x18},

    /* 0xE0 */                 /* 0xE1 */           /* 0xE2 */                  /* 0xE3 */    /* 0xE4 */                    /* 0xE5 */            /* 0xE6 */    /* 0xE7 */
    &RetCc{cond:FlagCond::PO}, &PopQq{r:Reg16::HL}, &JpCcNn{cond:FlagCond::PO}, &ExMemSpHl  , &CallCcNn{cond:FlagCond::PO}, &PushQq{r:Reg16::HL}, &AndN       , &Rst{addr:0x20},

    /* 0xE8 */                 /* 0xE9 */           /* 0xEA */                  /* 0xEB */    /* 0xEC */                    /* 0xED */            /* 0xEE */    /* 0xEF */
    &RetCc{cond:FlagCond::PE}, &JpMemHl           , &JpCcNn{cond:FlagCond::PE}, &ExDeHl     , &CallCcNn{cond:FlagCond::PE}, &Unsupported        , &XorN       , &Rst{addr:0x28},

    /* 0xF0 */                 /* 0xF1 */           /* 0xF2 */                  /* 0xF3 */    /* 0xF4 */                    /* 0xF5 */            /* 0xF6 */    /* 0xF7 */
    &RetCc{cond:FlagCond::P} , &PopQq{r:Reg16::AF}, &JpCcNn{cond:FlagCond::P} , &Di         , &CallCcNn{cond:FlagCond::P} , &PushQq{r:Reg16::AF}, &OrN        , &Rst{addr:0x30},

    /* 0xF8 */                 /* 0xF9 */           /* 0xFA */                  /* 0xFB */    /* 0xFC */                    /* 0xFD */            /* 0xFE */    /* 0xFF */
    &RetCc{cond:FlagCond::M} , &LdSpHl            , &JpCcNn{cond:FlagCond::M} , &Ei         , &CallCcNn{cond:FlagCond::M} , &Unsupported        , &CpN        , &Rst{addr:0x38}
];

