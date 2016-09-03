use super::cpu::*;
use num::FromPrimitive;


pub trait Instruction {
    fn execute(&self, &mut Cpu);
}


struct Unsupported;

impl Instruction for Unsupported {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:?}", cpu);

        panic!("Unsupported instruction {:#x} at address {:#06x}", cpu.read_word(cpu.get_pc()), cpu.get_pc());
    }
}


struct Nop;

impl Instruction for Nop {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(ONONE));

        info!("{:#06x}: NOP", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}


struct AdcR      { r: Reg8 }
struct AdcN      ;
struct AdcMemHl  ;
struct AdcMemIxD ;
struct AdcMemIyD ;
struct AdcHlSs   { r: Reg16 }

#[inline(always)]
fn update_flags_adc8(cpu: &mut Cpu, op1: u8, op2: u8, c: u8, res: u8) {
    cpu.cond_flag  ( SIGN_FLAG            , res & 0x80 != 0                                          );
    cpu.cond_flag  ( ZERO_FLAG            , res == 0                                                 );
    cpu.cond_flag  ( HALF_CARRY_FLAG      , (op1 & 0x0F) + (op2 & 0x0F) + c > 0x0F                   );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , (op1 & 0x80 == op2 & 0x80) && (op1 & 0x80 != res & 0x80) );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                                               );
    cpu.cond_flag  ( CARRY_FLAG           , op1 as u16 + op2 as u16 + c as u16 > 0xFF                );
    cpu.cond_flag  ( X_FLAG               , res & 0x08 != 0                                          );
    cpu.cond_flag  ( Y_FLAG               , res & 0x20 != 0                                          );
}

#[inline(always)]
fn update_flags_adc16(cpu: &mut Cpu, op1: u16, op2: u16, c: u16, res: u16) {
    cpu.cond_flag  ( SIGN_FLAG            , res & 0x8000 != 0                                                );
    cpu.cond_flag  ( ZERO_FLAG            , res == 0                                                         );
    cpu.cond_flag  ( HALF_CARRY_FLAG      , (op1 & 0x0FFF) + (op2 & 0x0FFF) + c > 0x0FFF                     );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , (op1 & 0x8000 == op2 & 0x8000) && (op1 & 0x8000 != res & 0x8000) );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                                                       );
    cpu.cond_flag  ( CARRY_FLAG           , op1 as u32 + op2 as u32 + c as u32 > 0xFFFF                      );
    cpu.cond_flag  ( X_FLAG               , (res >> 8) & 0x08 != 0                                           );
    cpu.cond_flag  ( Y_FLAG               , (res >> 8) & 0x20 != 0                                           );
}

impl Instruction for AdcR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(r).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, r, c, res);

        info!("{:#06x}: ADC A, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AdcN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(n).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, n, c, res);

        info!("{:#06x}: ADC A, {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AdcMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIX));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(memval).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, memval, c, res);

        if d < 0 {
            info!("{:#06x}: ADC A, (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: ADC A, (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AdcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(memval).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, memval, c, res);

        if d < 0 {
            info!("{:#06x}: ADC A, (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: ADC A, (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AdcMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OH|OL));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(memval).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_adc8(cpu, a, memval, c, res);

        info!("{:#06x}: ADC A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AdcHlSs {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL|OF|OutputRegisters::from(self.r)));

        let hl = cpu.read_reg16(Reg16::HL);
        let ss = cpu.read_reg16(self.r);
        let c  = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = hl.wrapping_add(ss).wrapping_add(c);

        cpu.write_reg16(Reg16::HL, res);

        update_flags_adc16(cpu, hl, ss, c, res);

        info!("{:#06x}: ADC HL, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OH|OL|OF));
    }
}


struct AddMemHl  ;
struct AddMemIxD ;
struct AddMemIyD ;
struct AddN      ;
struct AddR      { r: Reg8  }
struct AddHlSs   { r: Reg16 }
struct AddIxPp   { r: Reg16 }
struct AddIyRr   { r: Reg16 }

#[inline(always)]
fn update_flags_add8(cpu: &mut Cpu, op1: u8, op2: u8, res: u8) {
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
fn update_flags_add16(cpu: &mut Cpu, op1: u16, op2: u16, res: u16) {
    cpu.cond_flag  ( HALF_CARRY_FLAG   , (op1 & 0x0FFF) + (op2 & 0x0FFF) > 0x0FFF );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                            );
    cpu.cond_flag  ( CARRY_FLAG        , op1 as u32 + op2 as u32  > 0xFFFF        );
    cpu.cond_flag  ( X_FLAG            , (res >> 8) & 0x08 != 0                   );
    cpu.cond_flag  ( Y_FLAG            , (res >> 8) & 0x20 != 0                   );
}

impl Instruction for AddMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a.wrapping_add(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, memval, res);

        info!("{:#06x}: ADD A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AddMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIX));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a.wrapping_add(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, memval, res);

        if d < 0 {
            info!("{:#06x}: ADD A, (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: ADD A, (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AddMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a.wrapping_add(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, memval, res);

        if d < 0 {
            info!("{:#06x}: ADD A, (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: ADD A, (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AddN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);

        let res = a.wrapping_add(n);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, n, res);

        info!("{:#06x}: ADD A, {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AddR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a.wrapping_add(r);

        cpu.write_reg8(Reg8::A, res);

        update_flags_add8(cpu, a, r, res);

        info!("{:#06x}: ADD A, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AddHlSs {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL|OF|OutputRegisters::from(self.r)));

        let hl = cpu.read_reg16(Reg16::HL);
        let ss = cpu.read_reg16(self.r);

        let res = hl.wrapping_add(ss);

        cpu.write_reg16(Reg16::HL, res);

        update_flags_add16(cpu, hl, ss, res);

        info!("{:#06x}: ADD HL, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OH|OL|OF));
    }
}

impl Instruction for AddIxPp {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL|OIX));

        let ix = cpu.get_ix();
        let ss = cpu.read_reg16(self.r);

        let res = ix.wrapping_add(ss);

        cpu.set_ix(res);

        update_flags_add16(cpu, ix, ss, res);

        info!("{:#06x}: ADD IX, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OH|OL|OF|OIX));
    }
}

impl Instruction for AddIyRr {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL|OIY));

        let iy = cpu.get_iy();
        let ss = cpu.read_reg16(self.r);

        let res = iy.wrapping_add(ss);

        cpu.set_iy(res);

        update_flags_add16(cpu, iy, ss, res);

        info!("{:#06x}: ADD IY, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OH|OL|OF|OIY));
    }
}


struct AndR      { r: Reg8 }
struct AndN      ;
struct AndMemHl  ;
struct AndMemIxD ;
struct AndMemIyD ;

#[inline(always)]
fn update_flags_logical(cpu: &mut Cpu, res: u8) {
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
        debug!("{}", cpu.output(OA|OF|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a & r;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AndN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);

        let res = a & n;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AndMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a & memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AndMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIX));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a & memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        if d < 0 {
            info!("{:#06x}: AND A, (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: AND A, (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for AndMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a & memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        if d < 0 {
            info!("{:#06x}: AND A, (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: AND A, (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}


struct BitBMemIyD { b: u8 }

impl Instruction for BitBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF|OIY));

        let d    = cpu.read_word(cpu.get_pc()) as i8;
        let addr = ((cpu.get_iy() as i16) + d as i16) as u16;

        let memval = cpu.read_word(addr);

        cpu.cond_flag  ( ZERO_FLAG       , memval & (1 << self.b) == 0 );
        cpu.set_flag   ( HALF_CARRY_FLAG                               );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                             );

        if d < 0 {
            info!("{:#06x}: BIT {}, (IY-{:#04X})", cpu.get_pc() - 2, self.b, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: BIT {}, (IY+{:#04X})", cpu.get_pc() - 2, self.b, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OF));
    }
}


struct CallNn   ;
struct CallCcNn { cond: FlagCond }

impl Instruction for CallNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP));

        let curr_pc = cpu.get_pc();
        let nn      =  (cpu.read_word(curr_pc + 1) as u16) |
                      ((cpu.read_word(curr_pc + 2) as u16) << 8);
        let curr_sp = cpu.read_reg16(Reg16::SP);

        cpu.write_word(curr_sp - 1, (((curr_pc + 3) & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  ((curr_pc + 3) & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        info!("{:#06x}: CALL {:#06X}", curr_pc, nn);
        cpu.set_pc(nn);

        debug!("{}", cpu.output(OSP));
    }
}

impl Instruction for CallCcNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP|OF));

        let curr_pc = cpu.get_pc();
        let nn      =  (cpu.read_word(curr_pc + 1) as u16) |
                      ((cpu.read_word(curr_pc + 2) as u16) << 8);
        let curr_sp = cpu.read_reg16(Reg16::SP);
        let cc      = cpu.check_cond(self.cond);

        info!("{:#06x}: CALL {:?}, {:#06X}", curr_pc, self.cond, nn);

        if cc {
            cpu.write_word(curr_sp - 1, (((curr_pc + 3) & 0xFF00) >> 8) as u8);
            cpu.write_word(curr_sp - 2,  ((curr_pc + 3) & 0x00FF)       as u8);

            cpu.write_reg16(Reg16::SP, curr_sp - 2);

            cpu.set_pc(nn);
        } else {
            cpu.inc_pc(3);
        }

        debug!("{}", cpu.output(OSP));
    }
}


struct Ccf;

impl Instruction for Ccf {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        let c = cpu.get_flag(CARRY_FLAG);

        cpu.cond_flag  ( HALF_CARRY_FLAG   , c  );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG      );
        cpu.cond_flag  ( CARRY_FLAG        , !c );

        info!("{:#06x}: CCF", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF));
    }
}


struct CpR      { r: Reg8 }
struct CpN      ;
struct CpMemHl  ;
struct CpMemIyD ;

#[inline(always)]
fn update_flags_cp8(cpu: &mut Cpu, op1: u8, op2: u8, res: u8) {
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
        debug!("{}", cpu.output(OF|OA|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a.wrapping_sub(r);

        update_flags_cp8(cpu, a, r, res);

        info!("{:#06x}: CP {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for CpN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);

        let res = a.wrapping_sub(n);

        update_flags_cp8(cpu, a, n, res);

        info!("{:#06x}: CP {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for CpMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OH|OL));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a.wrapping_sub(memval);

        update_flags_cp8(cpu, a, memval, res);

        info!("{:#06x}: CP (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for CpMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a.wrapping_sub(memval);

        update_flags_cp8(cpu, a, memval, res);

        if d < 0 {
            info!("{:#06x}: CP (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: CP (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OF));
    }
}


struct DecR      { r: Reg8  }
struct DecMemHl  ;
struct DecMemIxD ;
struct DecMemIyD ;
struct DecSs     { r: Reg16 }

fn update_flags_dec8(cpu: &mut Cpu, op: u8, res: u8) {
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
        debug!("{}", cpu.output(OF|OutputRegisters::from(self.r)));

        let r   = cpu.read_reg8(self.r);
        let res = r.wrapping_sub(1);

        cpu.write_reg8(self.r, res);

        update_flags_dec8(cpu, r, res);

        info!("{:#06x}: DEC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF|OutputRegisters::from(self.r)));
    }
}

impl Instruction for DecMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF|OH|OL));

        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = memval.wrapping_sub(1);

        cpu.write_word(hl, res);

        update_flags_dec8(cpu, memval, res);

        info!("{:#06x}: DEC (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for DecMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF|OIX));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = memval.wrapping_sub(1);

        cpu.write_word(addr, res);

        update_flags_dec8(cpu, memval, res);

        if d < 0 {
            info!("{:#06x}: DEC (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: DEC (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for DecMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF|OIY));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = memval.wrapping_sub(1);

        cpu.write_word(addr, res);

        update_flags_dec8(cpu, memval, res);

        if d < 0 {
            info!("{:#06x}: DEC (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: DEC (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for DecSs {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)));

        let r   = cpu.read_reg16(self.r);
        let res = r.wrapping_sub(1);

        cpu.write_reg16(self.r, res);

        info!("{:#06x}: DEC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OutputRegisters::from(self.r)));
    }
}

struct Di;

impl Instruction for Di {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(ONONE));

        cpu.clear_iff1();
        cpu.clear_iff2();

        info!("{:#06x}: DI", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}


struct Djnz;

impl Instruction for Djnz {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OB));

        let b = cpu.read_reg8(Reg8::B);
        cpu.write_reg8(Reg8::B, b.wrapping_sub(1));

        let offset = cpu.read_word(cpu.get_pc() + 1) as i8 + 2;
        let target = (cpu.get_pc() as i16 + offset as i16) as u16;

        info!("{:#06x}: DJNZ {:#06X}", cpu.get_pc(), target);
        if b != 0 {
            cpu.set_pc(target);
        } else {
            cpu.inc_pc(2);
        }

        debug!("{}", cpu.output(OB));
    }
}


struct Ei;

impl Instruction for Ei {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(ONONE));

        cpu.set_iff1();
        cpu.set_iff2();

        info!("{:#06x}: EI", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}


struct ExAfAfAlt;
struct ExMemSpHl;
struct ExDeHl;

impl Instruction for ExAfAfAlt {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OA_ALT|OF_ALT));

        let af    = cpu.read_reg16(Reg16::AF);
        let afalt = cpu.read_reg16(Reg16::AF_ALT);

        cpu.write_reg16(Reg16::AF, afalt);
        cpu.write_reg16(Reg16::AF_ALT, af);

        info!("{:#06x}: EX AF, AF'", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF|OA_ALT|OF_ALT));
    }
}

impl Instruction for ExMemSpHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP|OH|OL));

        let sp = cpu.read_reg16(Reg16::SP);
        let hl = cpu.read_reg16(Reg16::HL);

        let (hlhigh, hllow) = (((hl & 0xFF00) >> 8) as u8,
                               ((hl & 0x00FF)       as u8));
        let spval =  (cpu.read_word(sp    ) as u16) |
                    ((cpu.read_word(sp + 1) as u16) << 8);

        cpu.write_reg16(Reg16::HL, spval);

        cpu.write_word(spval, hllow);
        cpu.write_word(spval + 1, hlhigh);

        info!("{:#06x}: EX (SP), HL", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OSP|OH|OL));
    }
}

impl Instruction for ExDeHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OD|OE|OH|OL));

        let de = cpu.read_reg16(Reg16::DE);
        let hl = cpu.read_reg16(Reg16::HL);

        cpu.write_reg16(Reg16::DE, hl);
        cpu.write_reg16(Reg16::HL, de);

        info!("{:#06x}: EX DE, HL", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OD|OE|OH|OL));
    }
}


struct Exx;

impl Instruction for Exx {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OB|OC|OD|OE|OH|OL|OB_ALT|OC_ALT|OD_ALT|OE_ALT|OH_ALT|OL_ALT));

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

        debug!("{}", cpu.output(OB|OC|OD|OE|OH|OL|OB_ALT|OC_ALT|OD_ALT|OE_ALT|OH_ALT|OL_ALT));
    }
}


struct Im { mode: u8 }

impl Instruction for Im {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(ONONE));

        cpu.set_im(self.mode);

        info!("{:#06x}: IM {}", cpu.get_pc() - 1, self.mode);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}


struct InAPortN;

impl Instruction for InAPortN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA));

        let n = cpu.read_word(cpu.get_pc() + 1);

        let port = Port::from_u8(n).unwrap();
        let portval = cpu.read_port(port);

        cpu.write_reg8(Reg8::A, portval);

        info!("{:#06x}: IN A, ({:#04X})", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA));
    }
}


struct IncR      { r: Reg8  }
struct IncMemHl  ;
struct IncMemIxD ;
struct IncMemIyD ;
struct IncSs     { r: Reg16 }

#[inline(always)]
fn update_flags_inc8(cpu: &mut Cpu, op: u8, res: u8) {
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
        debug!("{}", cpu.output(OF|OutputRegisters::from(self.r)));

        let r   = cpu.read_reg8(self.r);
        let res = r.wrapping_add(1);

        cpu.write_reg8(self.r, res);

        update_flags_inc8(cpu, r, res);

        info!("{:#06x}: INC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF|OutputRegisters::from(self.r)));
    }
}

impl Instruction for IncMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL|OF));

        let hl  = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = memval.wrapping_add(1);

        cpu.write_word(hl, res);

        update_flags_inc8(cpu, memval, res);

        info!("{:#06x}: INC (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for IncMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF|OIX));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = memval.wrapping_add(1);

        cpu.write_word(addr, res);

        update_flags_inc8(cpu, memval, res);

        if d < 0 {
            info!("{:#06x}: INC (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: INC (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for IncMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF|OIY));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = memval.wrapping_add(1);

        cpu.write_word(addr, res);

        update_flags_inc8(cpu, memval, res);

        if d < 0 {
            info!("{:#06x}: INC (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: INC (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OF));
    }
}

impl Instruction for IncSs {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)));

        let r   = cpu.read_reg16(self.r);
        let res = r.wrapping_add(1);

        cpu.write_reg16(self.r, res);

        info!("{:#06x}: INC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OutputRegisters::from(self.r)));
    }
}


struct JpMemHl;
struct JpNn   ;
struct JpCcNn { cond: FlagCond }

impl Instruction for JpMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL));

        let hl = cpu.read_reg16(Reg16::HL);

        info!("{:#06x}: JP (HL)", cpu.get_pc());
        cpu.set_pc(hl);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for JpNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(ONONE));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        info!("{:#06x}: JP {:#06X}", cpu.get_pc(), nn);
        cpu.set_pc(nn);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for JpCcNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        let cc = cpu.check_cond(self.cond);
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        info!("{:#06x}: JP {:?}, {:#06X}", cpu.get_pc(), self.cond, nn);
        if cc {
            cpu.set_pc(nn);
        } else {
            cpu.inc_pc(3);
        }

        debug!("{}", cpu.output(ONONE));
    }
}


struct JrZ ;
struct JrNz;
struct JrNcE;
struct JrCE;
struct JrE ;

impl Instruction for JrZ {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        let offset = cpu.read_word(cpu.get_pc() + 1) as i8 + 2;
        let target = (cpu.get_pc() as i16 + offset as i16) as u16;

        info!("{:#06x}: JR Z, {:#06X}", cpu.get_pc(), target);
        if cpu.get_flag(ZERO_FLAG) {
            cpu.set_pc(target);
        } else {
            cpu.inc_pc(2);
        }

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for JrNz {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        let offset = cpu.read_word(cpu.get_pc() + 1) as i8 + 2;
        let target = (cpu.get_pc() as i16 + offset as i16) as u16;

        info!("{:#06x}: JR NZ, {:#06X}", cpu.get_pc(), target);
        if cpu.get_flag(ZERO_FLAG) {
            cpu.inc_pc(2);
        } else {
            cpu.set_pc(target);
        }

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for JrNcE {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        let offset = cpu.read_word(cpu.get_pc() + 1) as i8 + 2;
        let target = (cpu.get_pc() as i16 + offset as i16) as u16;

        info!("{:#06x}: JR NC, {:#06X}", cpu.get_pc(), target);
        if cpu.get_flag(CARRY_FLAG) {
            cpu.inc_pc(2);
        } else {
            cpu.set_pc(target);
        }

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for JrCE {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        let offset = cpu.read_word(cpu.get_pc() + 1) as i8 + 2;
        let target = (cpu.get_pc() as i16 + offset as i16) as u16;

        info!("{:#06x}: JR C, {:#06X}", cpu.get_pc(), target);
        if cpu.get_flag(CARRY_FLAG) {
            cpu.set_pc(target);
        } else {
            cpu.inc_pc(2);
        }

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for JrE {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        let offset = cpu.read_word(cpu.get_pc() + 1) as i8 + 2;
        let target = (cpu.get_pc() as i16 + offset as i16) as u16;

        info!("{:#06x}: JR {:#06X}", cpu.get_pc(), target);
        cpu.set_pc(target);

        debug!("{}", cpu.output(ONONE));
    }
}


struct LdMemBcA  ;
struct LdMemDeA  ;
struct LdMemHlN  ;
struct LdMemHlR  { r: Reg8  }
struct LdMemIxDN ;
struct LdMemIxDR { r: Reg8  }
struct LdMemIyDN ;
struct LdMemIyDR { r: Reg8  }
struct LdMemNnA  ;
struct LdMemNnDd { r: Reg16 }
struct LdMemNnHl ;
struct LdMemNnIx ;
struct LdAMemDe  ;
struct LdAMemNn  ;
struct LdDdMemNn { r: Reg16 }
struct LdDdNn    { r: Reg16 }
struct LdRN      { r: Reg8  }
struct LdHlMemNn ;
struct LdRMemIyD { r: Reg8  }
struct LdSpHl    ;
struct LdIxNn    ;
struct LdIxMemNn ;
struct LdRR      { rt: Reg8, rs: Reg8 }
struct LdRMemHl  { r: Reg8  }
struct LdIyNn    ;

impl Instruction for LdMemBcA {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OB|OC));

        let bc = cpu.read_reg16(Reg16::BC);
        let a  = cpu.read_reg8(Reg8::A);

        cpu.write_word(bc, a);

        info!("{:#06x}: LD (BC), A", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemDeA {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OD|OE));

        let de = cpu.read_reg16(Reg16::DE);
        let a  = cpu.read_reg8(Reg8::A);

        cpu.write_word(de, a);

        info!("{:#06x}: LD (DE), A", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemHlN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL));

        let hl = cpu.read_reg16(Reg16::HL);
        let n  = cpu.read_word(cpu.get_pc() + 1);

        cpu.write_word(hl, n);

        info!("{:#06x}: LD (HL), {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemHlR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL|OutputRegisters::from(self.r)));

        let hl = cpu.read_reg16(Reg16::HL);
        let r  = cpu.read_reg8(self.r);

        cpu.write_word(hl, r);

        info!("{:#06x}: LD (HL), {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemIxDN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIX));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let n    = cpu.read_word(cpu.get_pc() + 2);
        let addr = ((cpu.get_ix() as i16) + d as i16) as u16;

        cpu.write_word(addr, n);

        if d < 0 {
            info!("{:#06x}: LD (IX-{:#04X}), {:#04X}", cpu.get_pc() - 1, (d ^ 0xFF) + 1, n);
        } else {
            info!("{:#06x}: LD (IX+{:#04X}), {:#04X}", cpu.get_pc() - 1, d, n);
        }
        cpu.inc_pc(3);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIX));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let r    = cpu.read_reg8(self.r);
        let addr = ((cpu.get_ix() as i16) + d as i16) as u16;

        cpu.write_word(addr, r);

        if d < 0 {
            info!("{:#06x}: LD (IX-{:#04X}), {:?}", cpu.get_pc() - 1, (d ^ 0xFF) + 1, self.r);
        } else {
            info!("{:#06x}: LD (IX+{:#04X}), {:?}", cpu.get_pc() - 1, d, self.r);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemIyDN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIY));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let n    = cpu.read_word(cpu.get_pc() + 2);
        let addr = ((cpu.get_iy() as i16) + d as i16) as u16;

        cpu.write_word(addr, n);

        if d < 0 {
            info!("{:#06x}: LD (IY-{:#04X}), {:#04X}", cpu.get_pc() - 1, (d ^ 0xFF) + 1, n);
        } else {
            info!("{:#06x}: LD (IY+{:#04X}), {:#04X}", cpu.get_pc() - 1, d, n);
        }
        cpu.inc_pc(3);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIY));

        let d    = cpu.read_word(cpu.get_pc() + 1) as i8;
        let r    = cpu.read_reg8(self.r);
        let addr = ((cpu.get_iy() as i16) + d as i16) as u16;

        cpu.write_word(addr, r);

        if d < 0 {
            info!("{:#06x}: LD (IY-{:#04X}), {:?}", cpu.get_pc() - 1, (d ^ 0xFF) + 1, self.r);
        } else {
            info!("{:#06x}: LD (IY+{:#04X}), {:?}", cpu.get_pc() - 1, d, self.r);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemNnA {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA));

        let a  = cpu.read_reg8(Reg8::A);
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.write_word(nn, a);

        info!("{:#06x}: LD ({:#06X}), A", cpu.get_pc(), nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemNnDd {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)));

        let r = cpu.read_reg16(self.r);
        let (rhigh, rlow) = (((r & 0xFF00) >> 8) as u8,
                             ((r & 0x00FF)       as u8));
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.write_word(nn, rlow);
        cpu.write_word(nn + 1, rhigh);

        info!("{:#06x}: LD ({:#06X}), {:?}", cpu.get_pc() - 1, nn, self.r);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemNnHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL));

        let hl = cpu.read_reg16(Reg16::HL);
        let (hlhigh, hllow) = (((hl & 0xFF00) >> 8) as u8,
                               ((hl & 0x00FF)       as u8));
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.write_word(nn, hllow);
        cpu.write_word(nn + 1, hlhigh);

        info!("{:#06x}: LD ({:#06X}), HL", cpu.get_pc(), nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdMemNnIx {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIX));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let (ixhigh, ixlow) = (((cpu.get_ix() & 0xFF00) >> 8) as u8,
                               ((cpu.get_ix() & 0x00FF)       as u8));

        cpu.write_word(nn, ixlow);
        cpu.write_word(nn + 1, ixhigh);

        info!("{:#06x}: LD ({:#06X}), IX", cpu.get_pc() - 1, nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for LdAMemDe {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OD|OE));

        let de     = cpu.read_reg16(Reg16::DE);
        let memval = cpu.read_word(de);

        cpu.write_reg8(Reg8::A, memval);

        info!("{:#06x}: LD A, (DE)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA));
    }
}

impl Instruction for LdAMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let memval = cpu.read_word(nn);

        cpu.write_reg8(Reg8::A, memval);

        info!("{:#06x}: LD A, ({:#06X})", cpu.get_pc(), nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(OA));
    }
}

impl Instruction for LdDdMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let nnmemval = (cpu.read_word(nn    ) as u16) |
                      ((cpu.read_word(nn + 1) as u16) << 8);

        cpu.write_reg16(self.r, nnmemval);

        info!("{:#06x}: LD {:?}, ({:#06X})", cpu.get_pc(), self.r, nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(OutputRegisters::from(self.r)));
    }
}

impl Instruction for LdDdNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.write_reg16(self.r, nn);

        info!("{:#06x}: LD {:?}, {:#06X}", cpu.get_pc(), self.r, nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(OutputRegisters::from(self.r)));
    }
}

impl Instruction for LdRN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)));

        let n = cpu.read_word(cpu.get_pc() + 1);

        cpu.write_reg8(self.r, n);

        info!("{:#06x}: LD {:?}, {:#04X}", cpu.get_pc(), self.r, n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OutputRegisters::from(self.r)));
    }
}

impl Instruction for LdHlMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let nnmemval = (cpu.read_word(nn    ) as u16) |
                      ((cpu.read_word(nn + 1) as u16) << 8);

        cpu.write_reg16(Reg16::HL, nnmemval);

        info!("{:#06x}: LD HL, ({:#06X})", cpu.get_pc(), nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(OH|OL));
    }
}

impl Instruction for LdRMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIY|OutputRegisters::from(self.r)));

        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.write_reg8(self.r, memval);

        if d < 0 {
            info!("{:#06x}: LD {:?}, (IY-{:#04X})", cpu.get_pc() - 1, self.r, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: LD {:?}, (IY+{:#04X})", cpu.get_pc() - 1, self.r, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OutputRegisters::from(self.r)));
    }
}

impl Instruction for LdSpHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP|OH|OL));

        let hl = cpu.read_reg16(Reg16::HL);
        cpu.write_reg16(Reg16::SP, hl);

        info!("{:#06x}: LD SP, HL", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OSP));
    }
}

impl Instruction for LdIxNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIX));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.set_ix(nn);

        info!("{:#06x}: LD IX, {:#06X}", cpu.get_pc() - 1, nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(OIX));
    }
}

impl Instruction for LdIxMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIX));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let nnmemval = (cpu.read_word(nn    ) as u16) |
                      ((cpu.read_word(nn + 1) as u16) << 8);

        cpu.set_ix(nnmemval);

        info!("{:#06x}: LD IX, TODO {:#06X}", cpu.get_pc() - 1, nnmemval);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(OIX));
    }
}

impl Instruction for LdRR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.rt) | OutputRegisters::from(self.rs)));

        let rs = cpu.read_reg8(self.rs);

        cpu.write_reg8(self.rt, rs);

        info!("{:#06x}: LD {:?}, {:?}", cpu.get_pc(), self.rt, self.rs);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OutputRegisters::from(self.rt) | OutputRegisters::from(self.rs)));
    }
}

impl Instruction for LdRMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)|OH|OL));

        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.write_reg8(self.r, memval);

        info!("{:#06x}: LD {:?}, (HL)", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OutputRegisters::from(self.r)));
    }
}

impl Instruction for LdIyNn {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIY));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.set_iy(nn);

        info!("{:#06x}: LD IY, {:#06X}", cpu.get_pc() - 1, nn);
        cpu.inc_pc(3);

        debug!("{}", cpu.output(OIY));
    }
}


struct Lddr;

impl Instruction for Lddr {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OB|OC|OD|OE|OH|OL|OF));

        let mut counter = cpu.read_reg16(Reg16::BC);
        while counter > 0 {
            let deval = cpu.read_reg16(Reg16::DE);
            let hlval = cpu.read_reg16(Reg16::HL);

            let memval = cpu.read_word(hlval);
            cpu.write_word(deval, memval);

            cpu.write_reg16(Reg16::DE, deval.wrapping_sub(1));
            cpu.write_reg16(Reg16::HL, hlval.wrapping_sub(1));

            counter -= 1;
            cpu.write_reg16(Reg16::BC, counter);
        }

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(PARITY_OVERFLOW_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        info!("{:#06x}: LDDR", cpu.get_pc() - 1);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OB|OC|OD|OE|OH|OL|OF));
    }
}


struct Ldir;

impl Instruction for Ldir {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OB|OC|OD|OE|OH|OL|OF));

        let mut counter = cpu.read_reg16(Reg16::BC);
        while counter > 0 {
            debug!("        Counter is: {}", counter);
            let de = cpu.read_reg16(Reg16::DE);
            let hl = cpu.read_reg16(Reg16::HL);

            let memval = cpu.read_word(hl);
            cpu.write_word(de, memval);

            cpu.write_reg16(Reg16::DE, de.wrapping_add(1));
            cpu.write_reg16(Reg16::HL, hl.wrapping_add(1));

            counter -= 1;
            cpu.write_reg16(Reg16::BC, counter);
        }

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(PARITY_OVERFLOW_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        info!("{:#06x}: LDIR", cpu.get_pc() - 1);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OB|OC|OD|OE|OH|OL|OF));
    }
}


struct OrR      { r: Reg8 }
struct OrN      ;
struct OrMemHl  ;
struct OrMemIxD ;
struct OrMemIyD ;

impl Instruction for OrR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a | r;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for OrN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);

        let res = a | n;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for OrMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OH|OL));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a | memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for OrMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIX));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a | memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        if d < 0 {
            info!("{:#06x}: OR A, (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: OR A, (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for OrMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a | memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        if d < 0 {
            info!("{:#06x}: OR A, (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: OR A, (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}


struct OutPortCR { r: Reg8 }
struct OutPortNA ;

impl Instruction for OutPortCR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OB|OC|OutputRegisters::from(self.r)));

        let r    = cpu.read_reg8(self.r);
        let bc   = cpu.read_reg16(Reg16::BC);
        let port = Port::from_u16(bc).unwrap();

        cpu.write_port(port, r);

        info!("{:#06x}: OUT (C), {:?}", cpu.get_pc() - 1, self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for OutPortNA {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA));

        let a    = cpu.read_reg8(Reg8::A);
        let n    = cpu.read_word(cpu.get_pc() + 1);
        let port = Port::from_u8(n).unwrap();

        cpu.write_port(port, a);

        info!("{:#06x}: OUT ({:#04X}), A", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(ONONE));
    }
}


struct PopQq { r: Reg16 }

impl Instruction for PopQq {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP|OutputRegisters::from(self.r)));

        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low  = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16(self.r, ((high as u16) << 8 ) | low as u16);
        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        info!("{:#06x}: POP {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OSP|OutputRegisters::from(self.r)));
    }
}


struct PushQq { r: Reg16 }

impl Instruction for PushQq {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OutputRegisters::from(self.r)|OSP));

        let curr_sp = cpu.read_reg16(Reg16::SP);
        let r = cpu.read_reg16(self.r);

        cpu.write_word(curr_sp - 1, ((r & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (r & 0x00FF)       as u8);
        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        info!("{:#06x}: PUSH {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OSP));
    }
}


struct ResBMemIyD { b: u8 }
struct ResBMemHl  { b: u8 }

impl Instruction for ResBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIY));

        let d      = cpu.read_word(cpu.get_pc()) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.write_word(addr, memval & !(1 << self.b));

        if d < 0 {
            info!("{:#06x}: RES {}, (IY-{:#04X})", cpu.get_pc() - 2, self.b, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: RES {}, (IY+{:#04X})", cpu.get_pc() - 2, self.b, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for ResBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL));

        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.write_word(hl, memval & !(1 << self.b));

        info!("{:#06x}: RES {}, (HL)", cpu.get_pc() - 1, self.b);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}


struct Ret   ;
struct RetCc { cond: FlagCond }

impl Instruction for Ret {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP));

        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low  = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        info!("{:#06x}: RET", cpu.get_pc());
        cpu.set_pc(((high as u16) << 8 ) | low as u16);

        debug!("{}", cpu.output(OSP));
    }
}

impl Instruction for RetCc {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP|OF));

        let cc = cpu.check_cond(self.cond);

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

        debug!("{}", cpu.output(OSP));
    }
}


struct RlR  { r: Reg8 }
struct RlcA ;

impl Instruction for RlR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF|OutputRegisters::from(self.r)));

        let r = cpu.read_reg8(self.r);

        let mut res = r.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x80 != 0 );

        info!("{:#06x}: RL {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF|OutputRegisters::from(self.r)));
    }
}

impl Instruction for RlcA {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

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

        debug!("{}", cpu.output(OA|OF));
    }
}


struct RrA;
struct RrcA;

impl Instruction for RrA {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

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

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for RrcA {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

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

        debug!("{}", cpu.output(OA|OF));
    }
}


struct Rst { addr: u8 }

impl Instruction for Rst {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OSP));

        let next_pc = cpu.get_pc() + 1;
        let curr_sp = cpu.read_reg16(Reg16::SP);

        cpu.write_word(curr_sp - 1, ((next_pc & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (next_pc & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        info!("{:#06x}: RST {:#04X}", cpu.get_pc(), self.addr);
        cpu.set_pc(self.addr as u16);

        debug!("{}", cpu.output(OSP));
    }
}


struct Scf;

impl Instruction for Scf {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OF));

        cpu.set_flag   ( CARRY_FLAG        );
        cpu.clear_flag ( HALF_CARRY_FLAG   );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG );

        info!("{:#06x}: SCF", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OF));
    }
}


struct SetBMemIyD { b: u8 }
struct SetBMemHl  { b: u8 }

impl Instruction for SetBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OIY));

        let d      = cpu.read_word(cpu.get_pc()) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.write_word(addr, memval | (1 << self.b));

        if d < 0 {
            info!("{:#06x}: SET {}, (IY-{:#04X})", cpu.get_pc() - 2, self.b, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: SET {}, (IY+{:#04X})", cpu.get_pc() - 2, self.b, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(ONONE));
    }
}

impl Instruction for SetBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL));

        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.write_word(hl, memval | (1 << self.b));

        info!("{:#06x}: SET {}, (HL)", cpu.get_pc() - 1, self.b);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(ONONE));
    }
}


struct SbcR      { r: Reg8 }
struct SbcN      ;
struct SbcMemHl  ;
struct SbcMemIxD ;
struct SbcMemIyD ;
struct SbcHlSs   { r: Reg16 }

#[inline(always)]
fn update_flags_sbc8(cpu: &mut Cpu, op1: u8, op2: u8, c: u8, res: u8) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0                                          );
    cpu.cond_flag ( ZERO_FLAG            , res == 0                                                 );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (op1 & 0x0F) < (op2 & 0x0F) + c                          );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , (op1 & 0x80 != op2 & 0x80) && (op1 & 0x80 != res & 0x80) );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                                               );
    cpu.cond_flag ( CARRY_FLAG           , (op1 as u16) < ((op2 as u16) + (c as u16))               );
    cpu.cond_flag ( X_FLAG               , res & 0x08 != 0                                          );
    cpu.cond_flag ( Y_FLAG               , res & 0x20 != 0                                          );
}

#[inline(always)]
fn update_flags_sbc16(cpu: &mut Cpu, op1: u16, op2: u16, c: u16, res: u16) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x8000 != 0                                                );
    cpu.cond_flag ( ZERO_FLAG            , res == 0                                                         );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (op1 & 0x00FF) < (op2 & 0x00FF) + c                              );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , (op1 & 0x8000 != op2 & 0x8000) && (op1 & 0x8000 != res & 0x8000) );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                                                       );
    cpu.cond_flag ( CARRY_FLAG           , (op1 as u32) < ((op2 as u32) + (c as u32))                       );
    cpu.cond_flag ( X_FLAG               , (res >> 8) & 0x08 != 0                                           );
    cpu.cond_flag ( Y_FLAG               , (res >> 8) & 0x20 != 0                                           );
}

impl Instruction for SbcR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(r).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, r, c, res);

        info!("{:#06x}: SBC A, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SbcN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(n).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, n, c, res);

        info!("{:#06x}: SBC A, {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SbcMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OH|OL));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(memval).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, memval, c, res);

        info!("{:#06x}: SBC A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SbcMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIX));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(memval).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, memval, c, res);

        if d < 0 {
            info!("{:#06x}: SBC A, (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: SBC A, (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SbcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(memval).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sbc8(cpu, a, memval, c, res);

        if d < 0 {
            info!("{:#06x}: SBC A, (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: SBC A, (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SbcHlSs {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OH|OL|OF|OutputRegisters::from(self.r)));

        let hl = cpu.read_reg16(Reg16::HL);
        let r  = cpu.read_reg16(self.r);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = hl.wrapping_sub(r).wrapping_sub(c);

        cpu.write_reg16(Reg16::HL, res);

        update_flags_sbc16(cpu, hl, r, c, res);

        info!("{:#06x}: SBC HL, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OH|OL|OF));
    }
}


struct SubR      { r: Reg8 }
struct SubN      ;
struct SubMemHl  ;
struct SubMemIxD ;
struct SubMemIyD ;

#[inline(always)]
fn update_flags_sub8(cpu: &mut Cpu, op1: u8, op2: u8, res: u8) {
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
        debug!("{}", cpu.output(OA|OF|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a.wrapping_sub(r);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, r, res);

        info!("{:#06x}: SUB {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SubN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);

        let res = a.wrapping_sub(n);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, n, res);

        info!("{:#06x}: SUB {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SubMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a.wrapping_sub(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, memval, res);

        info!("{:#06x}: SUB A, (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SubMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIX));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a.wrapping_sub(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, memval, res);

        if d < 0 {
            info!("{:#06x}: SUB A, (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: SUB A, (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for SubMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a.wrapping_sub(memval);

        cpu.write_reg8(Reg8::A, res);

        update_flags_sub8(cpu, a, memval, res);

        if d < 0 {
            info!("{:#06x}: SUB A, (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: SUB A, (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}


struct XorR      { r: Reg8 }
struct XorN      ;
struct XorMemHl  ;
struct XorMemIxD ;
struct XorMemIyD ;

impl Instruction for XorR {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OutputRegisters::from(self.r)));

        let a = cpu.read_reg8(Reg8::A);
        let r = cpu.read_reg8(self.r);

        let res = a ^ r;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for XorN {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);

        let res = a ^ n;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for XorMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF));

        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        let res = a ^ memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR (HL)", cpu.get_pc());
        cpu.inc_pc(1);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for XorMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIX));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_ix() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a ^ memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        if d < 0 {
            info!("{:#06x}: XOR A, (IX-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: XOR A, (IX+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

impl Instruction for XorMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        debug!("{}", cpu.output(OA|OF|OIY));

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(cpu.get_pc() + 1) as i8;
        let addr   = ((cpu.get_iy() as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        let res = a ^ memval;

        cpu.write_reg8(Reg8::A, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        if d < 0 {
            info!("{:#06x}: XOR A, (IY-{:#04X})", cpu.get_pc() - 1, (d ^ 0xFF) + 1);
        } else {
            info!("{:#06x}: XOR A, (IY+{:#04X})", cpu.get_pc() - 1, d);
        }
        cpu.inc_pc(2);

        debug!("{}", cpu.output(OA|OF));
    }
}

pub const INSTR_TABLE_CB: [&'static Instruction; 256] = [
    /* 0x00 */       /* 0x01 */       /* 0x02 */       /* 0x03 */       /* 0x04 */       /* 0x05 */       /* 0x06 */    /* 0x07 */
    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported, &Unsupported,

    /* 0x08 */       /* 0x09 */       /* 0x0A */       /* 0x0B */       /* 0x0C */       /* 0x0D */       /* 0x0E */    /* 0x0F */
    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported,    &Unsupported, &Unsupported,

    /* 0x10 */       /* 0x11 */       /* 0x12 */       /* 0x13 */       /* 0x14 */       /* 0x15 */       /* 0x16 */    /* 0x17 */
    &RlR{r:Reg8::B}, &RlR{r:Reg8::C}, &RlR{r:Reg8::D}, &RlR{r:Reg8::E}, &RlR{r:Reg8::H}, &RlR{r:Reg8::L}, &Unsupported, &RlR{r:Reg8::A},

    /* 0x18 */    /* 0x19 */    /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */    /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */    /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */    /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x38 */    /* 0x39 */    /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */    /* 0x41 */    /* 0x42 */    /* 0x43 */    /* 0x44 */    /* 0x45 */    /* 0x46 */    /* 0x47 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x48 */    /* 0x49 */    /* 0x4A */    /* 0x4B */    /* 0x4C */    /* 0x4D */    /* 0x4E */    /* 0x4F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x50 */    /* 0x51 */    /* 0x52 */    /* 0x53 */    /* 0x54 */    /* 0x55 */    /* 0x56 */    /* 0x57 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x58 */    /* 0x59 */    /* 0x5A */    /* 0x5B */    /* 0x5C */    /* 0x5D */    /* 0x5E */    /* 0x5F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x60 */    /* 0x61 */    /* 0x62 */    /* 0x63 */    /* 0x64 */    /* 0x65 */    /* 0x66 */    /* 0x67 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x68 */    /* 0x69 */    /* 0x6A */    /* 0x6B */    /* 0x6C */    /* 0x6D */    /* 0x6E */    /* 0x6F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x70 */    /* 0x71 */    /* 0x72 */    /* 0x73 */    /* 0x74 */    /* 0x75 */    /* 0x76 */    /* 0x77 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x78 */    /* 0x79 */    /* 0x7A */    /* 0x7B */    /* 0x7C */    /* 0x7D */    /* 0x7E */    /* 0x7F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */       /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:0}, &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */    /* 0x8D */    /* 0x8E */       /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:1}, &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */    /* 0x95 */    /* 0x96 */       /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:2}, &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */    /* 0x9D */    /* 0x9E */       /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:3}, &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */    /* 0xA5 */    /* 0xA6 */       /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:4}, &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */    /* 0xAD */    /* 0xAE */       /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:5}, &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */    /* 0xB5 */    /* 0xB6 */       /* 0xB7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:6}, &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */    /* 0xBD */    /* 0xBE */       /* 0xBF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemHl{b:7}, &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */    /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:0}, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */    /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:1}, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */    /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:2}, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */    /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:3}, &Unsupported,

    /* 0xE0 */    /* 0xE1 */    /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */    /* 0xE6 */    /* 0xE7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:4}, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */    /* 0xEF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:5}, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */    /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:6}, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */    /* 0xFF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemHl{b:7}, &Unsupported
];

pub const INSTR_TABLE_DD: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */             /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */             /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &AddIxPp{r:Reg16::BC}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */             /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */             /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &AddIxPp{r:Reg16::DE}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */             /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &LdIxNn     ,          &LdMemNnIx  , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */             /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &AddIxPp{r:Reg16::IX}, &LdIxMemNn  , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */             /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &IncMemIxD  , &DecMemIxD  , &LdMemIxDN  , &Unsupported,

    /* 0x38 */    /* 0x39 */             /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &AddIxPp{r:Reg16::SP}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */    /* 0x41 */    /* 0x42 */    /* 0x43 */    /* 0x44 */    /* 0x45 */    /* 0x46 */    /* 0x47 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x48 */    /* 0x49 */    /* 0x4A */    /* 0x4B */    /* 0x4C */    /* 0x4D */    /* 0x4E */    /* 0x4F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x50 */    /* 0x51 */    /* 0x52 */    /* 0x53 */    /* 0x54 */    /* 0x55 */    /* 0x56 */    /* 0x57 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x58 */    /* 0x59 */    /* 0x5A */    /* 0x5B */    /* 0x5C */    /* 0x5D */    /* 0x5E */    /* 0x5F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x60 */    /* 0x61 */    /* 0x62 */    /* 0x63 */    /* 0x64 */    /* 0x65 */    /* 0x66 */    /* 0x67 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x68 */    /* 0x69 */    /* 0x6A */    /* 0x6B */    /* 0x6C */    /* 0x6D */    /* 0x6E */    /* 0x6F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x70 */             /* 0x71 */             /* 0x72 */             /* 0x73 */             /* 0x74 */             /* 0x75 */             /* 0x76 */    /* 0x77 */
    &LdMemIxDR{r:Reg8::B}, &LdMemIxDR{r:Reg8::C}, &LdMemIxDR{r:Reg8::D}, &LdMemIxDR{r:Reg8::E}, &LdMemIxDR{r:Reg8::H}, &LdMemIxDR{r:Reg8::L}, &Unsupported, &LdMemIxDR{r:Reg8::A},

    /* 0x78 */    /* 0x79 */    /* 0x7A */    /* 0x7B */    /* 0x7C */    /* 0x7D */    /* 0x7E */    /* 0x7F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AddMemIxD  , &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */    /* 0x8D */    /* 0x8E */    /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AdcMemIxD  , &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */    /* 0x95 */    /* 0x96 */    /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SubMemIxD  , &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */    /* 0x9D */    /* 0x9E */    /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SbcMemIxD  , &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */    /* 0xA5 */    /* 0xA6 */    /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AndMemIxD  , &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */    /* 0xAD */    /* 0xAE */    /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &XorMemIxD  , &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */    /* 0xB5 */    /* 0xB6 */    /* 0xB7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &OrMemIxD   , &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */    /* 0xBD */    /* 0xBE */    /* 0xBF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */    /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */    /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */    /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */    /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE0 */    /* 0xE1 */           /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */            /* 0xE6 */    /* 0xE7 */
    &Unsupported, &PopQq{r:Reg16::IX}, &Unsupported, &Unsupported, &Unsupported, &PushQq{r:Reg16::IX}, &Unsupported, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */    /* 0xEF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */    /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */    /* 0xFF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported
];

pub const INSTR_TABLE_ED: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */    /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */    /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */    /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */    /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */    /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */    /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */    /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x38 */    /* 0x39 */    /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */    /* 0x41 */             /* 0x42 */             /* 0x43 */               /* 0x44 */    /* 0x45 */    /* 0x46 */    /* 0x47 */
    &Unsupported, &OutPortCR{r:Reg8::B}, &SbcHlSs{r:Reg16::BC}, &LdMemNnDd{r:Reg16::BC}, &Unsupported, &Unsupported, &Im{mode:0} , &Unsupported,

    /* 0x48 */    /* 0x49 */             /* 0x4A */             /* 0x4B */               /* 0x4C */    /* 0x4D */    /* 0x4E */    /* 0x4F */
    &Unsupported, &OutPortCR{r:Reg8::C}, &AdcHlSs{r:Reg16::BC}, &LdDdMemNn{r:Reg16::BC}, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x50 */    /* 0x51 */             /* 0x52 */             /* 0x53 */               /* 0x54 */    /* 0x55 */    /* 0x56 */    /* 0x57 */
    &Unsupported, &OutPortCR{r:Reg8::D}, &SbcHlSs{r:Reg16::DE}, &LdMemNnDd{r:Reg16::DE}, &Unsupported, &Unsupported, &Im{mode:1} , &Unsupported,

    /* 0x58 */    /* 0x59 */             /* 0x5A */             /* 0x5B */               /* 0x5C */    /* 0x5D */    /* 0x5E */    /* 0x5F */
    &Unsupported, &OutPortCR{r:Reg8::E}, &AdcHlSs{r:Reg16::DE}, &LdDdMemNn{r:Reg16::DE}, &Unsupported, &Unsupported, &Im{mode:2}, &Unsupported,

    /* 0x60 */    /* 0x61 */             /* 0x62 */             /* 0x63 */               /* 0x64 */    /* 0x65 */    /* 0x66 */    /* 0x67 */
    &Unsupported, &OutPortCR{r:Reg8::H}, &SbcHlSs{r:Reg16::HL}, &LdMemNnDd{r:Reg16::HL}, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x68 */    /* 0x69 */             /* 0x6A */             /* 0x6B */               /* 0x6C */    /* 0x6D */    /* 0x6E */    /* 0x6F */
    &Unsupported, &OutPortCR{r:Reg8::L}, &AdcHlSs{r:Reg16::HL}, &LdDdMemNn{r:Reg16::HL}, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x70 */    /* 0x71 */             /* 0x72 */             /* 0x73 */               /* 0x74 */    /* 0x75 */    /* 0x76 */    /* 0x77 */
    &Unsupported, &Unsupported         , &SbcHlSs{r:Reg16::SP}, &LdMemNnDd{r:Reg16::SP}, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x78 */    /* 0x79 */             /* 0x7A */             /* 0x7B */               /* 0x7C */    /* 0x7D */    /* 0x7E */    /* 0x7F */
    &Unsupported, &OutPortCR{r:Reg8::A}, &AdcHlSs{r:Reg16::SP}, &LdDdMemNn{r:Reg16::SP}, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */    /* 0x8D */    /* 0x8E */    /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */    /* 0x95 */    /* 0x96 */    /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */    /* 0x9D */    /* 0x9E */    /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */    /* 0xA5 */    /* 0xA6 */    /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */    /* 0xAD */    /* 0xAE */    /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */    /* 0xB5 */    /* 0xB6 */    /* 0xB7 */
    &Ldir       , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */    /* 0xBD */    /* 0xBE */    /* 0xBF */
    &Lddr       , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */    /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */    /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */    /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */    /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE0 */    /* 0xE1 */    /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */    /* 0xE6 */    /* 0xE7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */    /* 0xEF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */    /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */    /* 0xFF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported
];

pub const INSTR_TABLE_FD: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */             /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */             /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &AddIyRr{r:Reg16::BC}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */             /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */             /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &AddIyRr{r:Reg16::DE}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */             /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &LdIyNn     ,          &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */             /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &AddIyRr{r:Reg16::IY}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */             /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &IncMemIyD  , &DecMemIyD  , &LdMemIyDN,   &Unsupported,

    /* 0x38 */    /* 0x39 */             /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &AddIyRr{r:Reg16::SP}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */    /* 0x41 */    /* 0x42 */    /* 0x43 */    /* 0x44 */    /* 0x45 */    /* 0x46 */             /* 0x47 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::B}, &Unsupported,

    /* 0x48 */    /* 0x49 */    /* 0x4A */    /* 0x4B */    /* 0x4C */    /* 0x4D */    /* 0x4E */             /* 0x4F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::C}, &Unsupported,

    /* 0x50 */    /* 0x51 */    /* 0x52 */    /* 0x53 */    /* 0x54 */    /* 0x55 */    /* 0x56 */             /* 0x57 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::D}, &Unsupported,

    /* 0x58 */    /* 0x59 */    /* 0x5A */    /* 0x5B */    /* 0x5C */    /* 0x5D */    /* 0x5E */             /* 0x5F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::E}, &Unsupported,

    /* 0x60 */    /* 0x61 */    /* 0x62 */    /* 0x63 */    /* 0x64 */    /* 0x65 */    /* 0x66 */             /* 0x67 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::H}, &Unsupported,

    /* 0x68 */    /* 0x69 */    /* 0x6A */    /* 0x6B */    /* 0x6C */    /* 0x6D */    /* 0x6E */             /* 0x6F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::L}, &Unsupported,

    /* 0x70 */             /* 0x71 */             /* 0x72 */             /* 0x73 */             /* 0x74 */             /* 0x75 */             /* 0x76 */    /* 0x77 */
    &LdMemIyDR{r:Reg8::B}, &LdMemIyDR{r:Reg8::C}, &LdMemIyDR{r:Reg8::D}, &LdMemIyDR{r:Reg8::E}, &LdMemIyDR{r:Reg8::H}, &LdMemIyDR{r:Reg8::L}, &Unsupported, &LdMemIyDR{r:Reg8::A},

    /* 0x78 */    /* 0x79 */    /* 0x7A */    /* 0x7B */    /* 0x7C */    /* 0x7D */    /* 0x7E */             /* 0x7F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::A}, &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AddMemIyD  , &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */    /* 0x8D */    /* 0x8E */    /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AdcMemIyD  , &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */    /* 0x95 */    /* 0x96 */    /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SubMemIyD  , &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */    /* 0x9D */    /* 0x9E */    /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SbcMemIyD  , &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */    /* 0xA5 */    /* 0xA6 */    /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AndMemIyD  , &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */    /* 0xAD */    /* 0xAE */    /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &XorMemIyD  , &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */    /* 0xB5 */    /* 0xB6 */    /* 0xB7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &OrMemIyD   , &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */    /* 0xBD */    /* 0xBE */    /* 0xBF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &CpMemIyD   , &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */    /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */    /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */    /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */    /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE0 */    /* 0xE1 */           /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */            /* 0xE6 */    /* 0xE7 */
    &Unsupported, &PopQq{r:Reg16::IY}, &Unsupported, &Unsupported, &Unsupported, &PushQq{r:Reg16::IY}, &Unsupported, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */    /* 0xEF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */    /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */    /* 0xFF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported
];

pub const INSTR_TABLE_DDCB: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */    /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */    /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */    /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */    /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */    /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */    /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */    /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x38 */    /* 0x39 */    /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */    /* 0x41 */    /* 0x42 */    /* 0x43 */    /* 0x44 */    /* 0x45 */    /* 0x46 */    /* 0x47 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x48 */    /* 0x49 */    /* 0x4A */    /* 0x4B */    /* 0x4C */    /* 0x4D */    /* 0x4E */    /* 0x4F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x50 */    /* 0x51 */    /* 0x52 */    /* 0x53 */    /* 0x54 */    /* 0x55 */    /* 0x56 */    /* 0x57 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x58 */    /* 0x59 */    /* 0x5A */    /* 0x5B */    /* 0x5C */    /* 0x5D */    /* 0x5E */    /* 0x5F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x60 */    /* 0x61 */    /* 0x62 */    /* 0x63 */    /* 0x64 */    /* 0x65 */    /* 0x66 */    /* 0x67 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x68 */    /* 0x69 */    /* 0x6A */    /* 0x6B */    /* 0x6C */    /* 0x6D */    /* 0x6E */    /* 0x6F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x70 */    /* 0x71 */    /* 0x72 */    /* 0x73 */    /* 0x74 */    /* 0x75 */    /* 0x76 */    /* 0x77 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x78 */    /* 0x79 */    /* 0x7A */    /* 0x7B */    /* 0x7C */    /* 0x7D */    /* 0x7E */    /* 0x7F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */    /* 0x8D */    /* 0x8E */    /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */    /* 0x95 */    /* 0x96 */    /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */    /* 0x9D */    /* 0x9E */    /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */    /* 0xA5 */    /* 0xA6 */    /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */    /* 0xAD */    /* 0xAE */    /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */    /* 0xB5 */    /* 0xB6 */    /* 0xB7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */    /* 0xBD */    /* 0xBE */    /* 0xBF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */    /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */    /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */    /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */    /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE0 */    /* 0xE1 */    /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */    /* 0xE6 */    /* 0xE7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */    /* 0xEF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */    /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */    /* 0xFF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported
];

pub const INSTR_TABLE_FDCB: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */    /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */    /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */    /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */    /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */    /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */    /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */    /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x38 */    /* 0x39 */    /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */    /* 0x41 */    /* 0x42 */    /* 0x43 */    /* 0x44 */    /* 0x45 */    /* 0x46 */        /* 0x47 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:0}, &Unsupported,

    /* 0x48 */    /* 0x49 */    /* 0x4A */    /* 0x4B */    /* 0x4C */    /* 0x4D */    /* 0x4E */        /* 0x4F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:1}, &Unsupported,

    /* 0x50 */    /* 0x51 */    /* 0x52 */    /* 0x53 */    /* 0x54 */    /* 0x55 */    /* 0x56 */        /* 0x57 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:2}, &Unsupported,

    /* 0x58 */    /* 0x59 */    /* 0x5A */    /* 0x5B */    /* 0x5C */    /* 0x5D */    /* 0x5E */        /* 0x5F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:3}, &Unsupported,

    /* 0x60 */    /* 0x61 */    /* 0x62 */    /* 0x63 */    /* 0x64 */    /* 0x65 */    /* 0x66 */        /* 0x67 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:4}, &Unsupported,

    /* 0x68 */    /* 0x69 */    /* 0x6A */    /* 0x6B */    /* 0x6C */    /* 0x6D */    /* 0x6E */        /* 0x6F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:5}, &Unsupported,

    /* 0x70 */    /* 0x71 */    /* 0x72 */    /* 0x73 */    /* 0x74 */    /* 0x75 */    /* 0x76 */        /* 0x77 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:6}, &Unsupported,

    /* 0x78 */    /* 0x79 */    /* 0x7A */    /* 0x7B */    /* 0x7C */    /* 0x7D */    /* 0x7E */        /* 0x7F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &BitBMemIyD{b:7}, &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */        /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:0}, &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */    /* 0x8D */    /* 0x8E */        /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:1}, &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */    /* 0x95 */    /* 0x96 */        /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:2}, &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */    /* 0x9D */    /* 0x9E */        /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:3}, &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */    /* 0xA5 */    /* 0xA6 */        /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:4}, &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */    /* 0xAD */    /* 0xAE */        /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:5}, &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */    /* 0xB5 */    /* 0xB6 */        /* 0xB7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:6}, &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */    /* 0xBD */    /* 0xBE */        /* 0xBF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &ResBMemIyD{b:7}, &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */        /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:0}, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */        /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:1}, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */        /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:2}, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */        /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:3}, &Unsupported,

    /* 0xE0 */    /* 0xE1 */    /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */    /* 0xE6 */        /* 0xE7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:4}, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */        /* 0xEF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:5}, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */        /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:6}, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */        /* 0xFF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SetBMemIyD{b:7}, &Unsupported
];

pub const INSTR_TABLE: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */             /* 0x02 */    /* 0x03 */           /* 0x04 */        /* 0x05 */        /* 0x06 */        /* 0x07 */
    &Nop        , &LdDdNn{r:Reg16::BC} , &LdMemBcA   , &IncSs{r:Reg16::BC}, &IncR{r:Reg8::B}, &DecR{r:Reg8::B}, &LdRN{r:Reg8::B}, &RlcA       ,

    /* 0x08 */    /* 0x09 */             /* 0x0A */    /* 0x0B */           /* 0x0C */        /* 0x0D */        /* 0x0E */        /* 0x0F */
    &ExAfAfAlt  , &AddHlSs{r:Reg16::BC}, &Unsupported, &DecSs{r:Reg16::BC}, &IncR{r:Reg8::C}, &DecR{r:Reg8::C}, &LdRN{r:Reg8::C}, &RrcA       ,

    /* 0x10 */    /* 0x11 */             /* 0x12 */    /* 0x13 */           /* 0x14 */        /* 0x15 */        /* 0x16 */        /* 0x17 */
    &Djnz       , &LdDdNn{r:Reg16::DE} , &LdMemDeA   , &IncSs{r:Reg16::DE}, &IncR{r:Reg8::D}, &DecR{r:Reg8::D}, &LdRN{r:Reg8::D}, &Unsupported,

    /* 0x18 */    /* 0x19 */             /* 0x1A */    /* 0x1B */           /* 0x1C */        /* 0x1D */        /* 0x1E */        /* 0x1F */
    &JrE        , &AddHlSs{r:Reg16::DE}, &LdAMemDe   , &DecSs{r:Reg16::DE}, &IncR{r:Reg8::E}, &DecR{r:Reg8::E}, &LdRN{r:Reg8::E}, &RrA        ,

    /* 0x20 */    /* 0x21 */             /* 0x22 */    /* 0x23 */           /* 0x24 */        /* 0x25 */        /* 0x26 */        /* 0x27 */
    &JrNz       , &LdDdNn{r:Reg16::HL} , &LdMemNnHl  , &IncSs{r:Reg16::HL}, &IncR{r:Reg8::H}, &DecR{r:Reg8::H}, &LdRN{r:Reg8::H}, &Unsupported,

    /* 0x28 */    /* 0x29 */             /* 0x2A */    /* 0x2B */           /* 0x2C */        /* 0x2D */        /* 0x2E */        /* 0x2F */
    &JrZ        , &AddHlSs{r:Reg16::HL}, &LdHlMemNn  , &DecSs{r:Reg16::HL}, &IncR{r:Reg8::L}, &DecR{r:Reg8::L}, &LdRN{r:Reg8::L}, &Unsupported,

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
    &LdMemHlR{r:Reg8::H}                     , &LdMemHlR{r:Reg8::L}                   , &Unsupported                      , &LdMemHlR{r:Reg8::A},

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


#[cfg(test)]
mod test {

    use super::super::cpu::*;
    use super::super::memory::*;
    use super::*;

    #[test]
    fn add_a_r() {
        let memory = MemoryBuilder::new().finalize();
        let mut cpu = Cpu::new(memory);
        let instr = super::AddR { r:Reg8::B };

        // Test sign flag
        cpu.write_reg8(Reg8::A, 0x80);
        cpu.write_reg8(Reg8::B, 0x00);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x80);
        assert!(cpu.check_flags(SIGN_FLAG));

        // Test zero flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x00);
        cpu.write_reg8(Reg8::B, 0x00);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x00);
        assert!(cpu.check_flags(ZERO_FLAG));

        // Test half-carry flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x0F);
        cpu.write_reg8(Reg8::B, 0x0F);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x1E);
        assert!(cpu.check_flags(HALF_CARRY_FLAG | X_FLAG));

        // Test overflow flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x78);
        cpu.write_reg8(Reg8::B, 0x69);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0xE1);
        assert!(cpu.check_flags(SIGN_FLAG | PARITY_OVERFLOW_FLAG | HALF_CARRY_FLAG | Y_FLAG));

        // Test carry flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0xFF);
        cpu.write_reg8(Reg8::B, 0xFF);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0xFE);
        assert!(cpu.check_flags(SIGN_FLAG | CARRY_FLAG | HALF_CARRY_FLAG | X_FLAG | Y_FLAG));
    }

    #[test]
    fn add_hl_ss() {
        let memory = MemoryBuilder::new().finalize();
        let mut cpu = Cpu::new(memory);
        let instr = super::AddHlSs { r:Reg16::BC };

        // Test half-carry flag
        cpu.write_reg16(Reg16::HL, 0x0FFF);
        cpu.write_reg16(Reg16::BC, 0x0002);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg16(Reg16::HL) == 0x1001);
        assert!(cpu.check_flags(HALF_CARRY_FLAG));

        // Test carry flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg16(Reg16::HL, 0xFFFF);
        cpu.write_reg16(Reg16::BC, 0xFFFF);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg16(Reg16::HL) == 0xFFFE);
        assert!(cpu.check_flags(CARRY_FLAG | HALF_CARRY_FLAG | X_FLAG | Y_FLAG));
    }

    #[test]
    fn cp_r() {
        let memory = MemoryBuilder::new().finalize();
        let mut cpu = Cpu::new(memory);
        let instr = super::CpR { r:Reg8::B };

        // Test add/subtract flag
        cpu.write_reg8(Reg8::A, 0x42);
        cpu.write_reg8(Reg8::B, 0x41);
        instr.execute(&mut cpu);
        assert!(cpu.check_flags(ADD_SUBTRACT_FLAG));

        // Test zero flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x42);
        cpu.write_reg8(Reg8::B, 0x42);
        instr.execute(&mut cpu);
        assert!(cpu.check_flags(ZERO_FLAG | ADD_SUBTRACT_FLAG));

        // Test sign flag, half carry and carry flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x42);
        cpu.write_reg8(Reg8::B, 0x43);
        instr.execute(&mut cpu);
        assert!(cpu.check_flags(SIGN_FLAG | HALF_CARRY_FLAG | ADD_SUBTRACT_FLAG | CARRY_FLAG));

        // Test overflow flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x7F);
        cpu.write_reg8(Reg8::B, 0xA0);
        instr.execute(&mut cpu);
        assert!(cpu.check_flags(SIGN_FLAG | ADD_SUBTRACT_FLAG | CARRY_FLAG | PARITY_OVERFLOW_FLAG | Y_FLAG));
    }

    #[test]
    fn dec_r() {
        let memory = MemoryBuilder::new().finalize();
        let mut cpu = Cpu::new(memory);
        let instr = super::DecR { r:Reg8::A };

        // Test add/subtract flag
        cpu.write_reg8(Reg8::A, 0x42);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x41);
        assert!(cpu.check_flags(ADD_SUBTRACT_FLAG));

        // Test zero flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x01);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x00);
        assert!(cpu.check_flags(ZERO_FLAG | ADD_SUBTRACT_FLAG));

        // Test half carry
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x10);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x0F);
        assert!(cpu.check_flags(HALF_CARRY_FLAG | ADD_SUBTRACT_FLAG | X_FLAG));

        // Test overflow flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x80);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x7F);
        assert!(cpu.check_flags(PARITY_OVERFLOW_FLAG | HALF_CARRY_FLAG | ADD_SUBTRACT_FLAG | X_FLAG | Y_FLAG));

        // Test sign flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x00);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0xFF);
        assert!(cpu.check_flags(SIGN_FLAG | HALF_CARRY_FLAG | ADD_SUBTRACT_FLAG | X_FLAG | Y_FLAG));
    }

    #[test]
    fn inc_r() {
        let memory = MemoryBuilder::new().finalize();
        let mut cpu = Cpu::new(memory);
        let instr = super::IncR { r:Reg8::A };

        // Test add/subtract flag cleared
        cpu.write_reg8(Reg8::A, 0x42);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x43);
        assert!(cpu.check_flags(EMPTY_FLAGS));

        // Test zero flag and half carry
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0xFF);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x00);
        assert!(cpu.check_flags(ZERO_FLAG | HALF_CARRY_FLAG));

        // Test overflow flag and sign flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x7F);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x80);
        assert!(cpu.check_flags(PARITY_OVERFLOW_FLAG | HALF_CARRY_FLAG | SIGN_FLAG));
    }

    #[test]
    fn sbc_r() {
        let memory = MemoryBuilder::new().finalize();
        let mut cpu = Cpu::new(memory);
        let instr = super::SbcR { r:Reg8::B };

        // Test add/subtract flag
        cpu.write_reg8(Reg8::A, 0x42);
        cpu.write_reg8(Reg8::B, 0x41);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x01);
        assert!(cpu.check_flags(ADD_SUBTRACT_FLAG));

        // Test zero flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x42);
        cpu.write_reg8(Reg8::B, 0x41);
        cpu.set_flag(CARRY_FLAG);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x00);
        assert!(cpu.check_flags(ZERO_FLAG | ADD_SUBTRACT_FLAG));

        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x42);
        cpu.write_reg8(Reg8::B, 0x42);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x00);
        assert!(cpu.check_flags(ZERO_FLAG | ADD_SUBTRACT_FLAG));

        // Test sign, half carry and carry flags
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x01);
        cpu.write_reg8(Reg8::B, 0x02);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0xFF);
        assert!(cpu.check_flags(SIGN_FLAG | ADD_SUBTRACT_FLAG | HALF_CARRY_FLAG | CARRY_FLAG | X_FLAG | Y_FLAG));

        // Test overflow flag
        cpu.clear_flag(ALL_FLAGS);
        cpu.write_reg8(Reg8::A, 0x80);
        cpu.write_reg8(Reg8::B, 0x01);
        instr.execute(&mut cpu);
        assert!(cpu.read_reg8(Reg8::A) == 0x7F);
        assert!(cpu.check_flags(PARITY_OVERFLOW_FLAG | HALF_CARRY_FLAG | ADD_SUBTRACT_FLAG | X_FLAG | Y_FLAG));
    }
}

