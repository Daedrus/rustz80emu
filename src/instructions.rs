use super::cpu::*;
use super::debugger::*;


pub trait Instruction {
    fn execute(&self, &mut Cpu);
    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters);
}


struct Unsupported;

impl Instruction for Unsupported {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        panic!("Unsupported instruction {:#x} at address {:#06x}", cpu.read_word(curr_pc), curr_pc);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


struct Nop;

impl Instruction for Nop {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: NOP", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
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

impl Instruction for AdcMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(memval).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_adc8(cpu, a, memval, c, res);

        info!("{:#06x}: ADC A, (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for AdcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_add(memval).wrapping_add(c);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_adc8(cpu, a, memval, c, res);

        info!("{:#06x}: ADC A, (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
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

impl Instruction for AdcHlSs {
    fn execute(&self, cpu: &mut Cpu) {
        let hl = cpu.read_reg16(Reg16::HL);
        let ss = cpu.read_reg16(self.r);
        let c  = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = hl.wrapping_add(ss).wrapping_add(c);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(Reg16::HL, res);
        cpu.write_reg16(Reg16::WZ, hl + 1);

        update_flags_adc16(cpu, hl, ss, c, res);

        info!("{:#06x}: ADC HL, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OF|OWZ|OutputRegisters::from(self.r), OH|OL|OF|OWZ)
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

impl Instruction for AddMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a.wrapping_add(memval);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_add8(cpu, a, memval, res);

        info!("{:#06x}: ADD A, (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for AddMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a.wrapping_add(memval);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_add8(cpu, a, memval, res);

        info!("{:#06x}: ADD A, (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
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

impl Instruction for AddIxPp {
    fn execute(&self, cpu: &mut Cpu) {
        let ix = cpu.read_reg16(Reg16::IX);
        let ss = cpu.read_reg16(self.r);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        let res = ix.wrapping_add(ss);

        cpu.write_reg16(Reg16::IX, res);

        update_flags_add16(cpu, ix, ss, res);

        info!("{:#06x}: ADD IX, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OIX, OH|OL|OF|OIX)
    }
}

impl Instruction for AddIyRr {
    fn execute(&self, cpu: &mut Cpu) {
        let iy = cpu.read_reg16(Reg16::IY);
        let ss = cpu.read_reg16(self.r);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        let res = iy.wrapping_add(ss);

        cpu.write_reg16(Reg16::IY, res);

        update_flags_add16(cpu, iy, ss, res);

        info!("{:#06x}: ADD IY, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OIY, OH|OL|OF|OIY)
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

impl Instruction for AndMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a & memval;

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND A, (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for AndMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a & memval;

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.set_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: AND A, (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }
}


struct BitBR      { b: u8, r: Reg8 }
struct BitBMemHl  { b: u8 }
struct BitBMemIxD { b: u8 }
struct BitBMemIyD { b: u8 }

#[inline(always)]
fn update_flags_bit(cpu: &mut Cpu, b: u8, bit_is_set: bool) {
    cpu.cond_flag  ( SIGN_FLAG            , b == 7 && bit_is_set );
    cpu.cond_flag  ( ZERO_FLAG            , !bit_is_set          );
    cpu.set_flag   ( HALF_CARRY_FLAG                             );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , !bit_is_set          );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                           );
}

#[inline(always)]
fn update_xyflags_bit(cpu: &mut Cpu) {
    let wz = cpu.read_reg16(Reg16::WZ);

    cpu.cond_flag  ( X_FLAG, wz & 0x0800 != 0 );
    cpu.cond_flag  ( Y_FLAG, wz & 0x2000 != 0 );
}

impl Instruction for BitBR {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);

        update_flags_bit(cpu, self.b, val & (1 << self.b) != 0);
        cpu.cond_flag ( X_FLAG , val & 0x08 != 0 );
        cpu.cond_flag ( Y_FLAG , val & 0x20 != 0 );

        info!("{:#06x}: BIT {}, {:?}", cpu.get_pc() - 1, self.b, self.r);

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF)
    }
}

impl Instruction for BitBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        update_flags_bit(cpu, self.b, memval & (1 << self.b) != 0);
        update_xyflags_bit(cpu);

        info!("{:#06x}: BIT {}, (HL)", cpu.get_pc() - 1, self.b);

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for BitBMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        update_flags_bit(cpu, self.b, memval & (1 << self.b) != 0);
        update_xyflags_bit(cpu);

        info!("{:#06x}: BIT {}, (IX{:+#04X})", cpu.get_pc() - 2, self.b, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX, OF)
    }
}

impl Instruction for BitBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        update_flags_bit(cpu, self.b, memval & (1 << self.b) != 0);
        update_xyflags_bit(cpu);

        info!("{:#06x}: BIT {}, (IY{:+#04X})", cpu.get_pc() - 2, self.b, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY, OF)
    }
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

            let nn =  (cpu.zero_cycle_read_word(curr_pc + 1) as u16) |
                     ((cpu.zero_cycle_read_word(curr_pc + 2) as u16) << 8);

            info!("{:#06x}: CALL {:?}, {:#06X}", curr_pc, self.cond, nn);
            cpu.inc_pc(3);
            cpu.write_reg16(Reg16::WZ, nn);
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


struct CpR      { r: Reg8 }
struct CpN      ;
struct CpMemHl  ;
struct CpMemIxD ;
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

impl Instruction for CpMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a.wrapping_sub(memval);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_cp8(cpu, a, memval, res);

        info!("{:#06x}: CP (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for CpMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a.wrapping_sub(memval);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_cp8(cpu, a, memval, res);

        info!("{:#06x}: CP (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OF|OWZ)
    }
}


struct Cpd;
struct Cpdr;

#[inline(always)]
fn update_flags_cpd(cpu: &mut Cpu, a: u8, memval: u8, bc: u16, res: u8) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0              );
    cpu.cond_flag ( ZERO_FLAG            , res == 0                     );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (a & 0x0F) < (memval & 0x0F) );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , bc != 0                      );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                   );

    let res = if cpu.get_flag(HALF_CARRY_FLAG) { res - 1 } else { res };

    cpu.cond_flag ( X_FLAG               , res & 0x08 != 0              );
    cpu.cond_flag ( Y_FLAG               , res & 0x02 != 0              );
}

#[inline(always)]
fn cpd(cpu: &mut Cpu) {
    let bc     = cpu.read_reg16(Reg16::BC);
    let hl     = cpu.read_reg16(Reg16::HL);
    let a      = cpu.read_reg8(Reg8::A);
    let memval = cpu.read_word(hl);

    let res = a.wrapping_sub(memval);

    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);

    cpu.write_reg16(Reg16::BC, bc.wrapping_sub(1));
    cpu.write_reg16(Reg16::HL, hl.wrapping_sub(1));

    update_flags_cpd(cpu, a, memval, bc.wrapping_sub(1), res);
}

impl Instruction for Cpd {
    fn execute(&self, cpu: &mut Cpu) {
        cpd(cpu);

        let wz = cpu.read_reg16(Reg16::WZ);
        cpu.write_reg16(Reg16::WZ, wz - 1);

        info!("{:#06x}: CPD", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OB|OC|OH|OL|OF|OWZ, OB|OC|OH|OL|OF|OWZ)
    }
}

impl Instruction for Cpdr {
    fn execute(&self, cpu: &mut Cpu) {
        cpd(cpu);

        info!("{:#06x}: CPDR", cpu.get_pc() - 1);
        if cpu.get_flag(PARITY_OVERFLOW_FLAG) && !cpu.get_flag(ZERO_FLAG) {
            let hl = cpu.read_reg16(Reg16::HL);
            cpu.contend_read_no_mreq(hl + 1);
            cpu.contend_read_no_mreq(hl + 1);
            cpu.contend_read_no_mreq(hl + 1);
            cpu.contend_read_no_mreq(hl + 1);
            cpu.contend_read_no_mreq(hl + 1);

            let curr_pc = cpu.get_pc();
            cpu.write_reg16(Reg16::WZ, curr_pc + 1);

            cpu.dec_pc(1);
        } else {
            cpu.inc_pc(1);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OB|OC|OH|OL|OF|OWZ, OB|OC|OH|OL|OF|OWZ)
    }
}


struct Cpi;
struct Cpir;

#[inline(always)]
fn update_flags_cpi(cpu: &mut Cpu, a: u8, memval: u8, bc: u16, res: u8) {
    cpu.cond_flag ( SIGN_FLAG            , res & 0x80 != 0              );
    cpu.cond_flag ( ZERO_FLAG            , res == 0                     );
    cpu.cond_flag ( HALF_CARRY_FLAG      , (a & 0x0F) < (memval & 0x0F) );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , bc != 0                      );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                   );

    let res = if cpu.get_flag(HALF_CARRY_FLAG) { res - 1 } else { res };

    cpu.cond_flag ( X_FLAG               , res & 0x08 != 0              );
    cpu.cond_flag ( Y_FLAG               , res & 0x02 != 0              );
}

#[inline(always)]
fn cpi(cpu: &mut Cpu) {
    let bc     = cpu.read_reg16(Reg16::BC);
    let hl     = cpu.read_reg16(Reg16::HL);
    let a      = cpu.read_reg8(Reg8::A);
    let memval = cpu.read_word(hl);

    let res = a.wrapping_sub(memval);

    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);
    cpu.contend_read_no_mreq(hl);

    cpu.write_reg16(Reg16::BC, bc.wrapping_sub(1));
    cpu.write_reg16(Reg16::HL, hl.wrapping_add(1));

    update_flags_cpi(cpu, a, memval, bc.wrapping_sub(1), res);
}

impl Instruction for Cpi {
    fn execute(&self, cpu: &mut Cpu) {
        cpi(cpu);

        let wz = cpu.read_reg16(Reg16::WZ);
        cpu.write_reg16(Reg16::WZ, wz + 1);

        info!("{:#06x}: CPI", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OB|OC|OH|OL|OF|OWZ, OB|OC|OH|OL|OF|OWZ)
    }
}

impl Instruction for Cpir {
    fn execute(&self, cpu: &mut Cpu) {
        cpi(cpu);

        info!("{:#06x}: CPIR", cpu.get_pc() - 1);
        if cpu.get_flag(PARITY_OVERFLOW_FLAG) && !cpu.get_flag(ZERO_FLAG) {
            let hl = cpu.read_reg16(Reg16::HL);
            cpu.contend_read_no_mreq(hl - 1);
            cpu.contend_read_no_mreq(hl - 1);
            cpu.contend_read_no_mreq(hl - 1);
            cpu.contend_read_no_mreq(hl - 1);
            cpu.contend_read_no_mreq(hl - 1);

            let curr_pc = cpu.get_pc();
            cpu.write_reg16(Reg16::WZ, curr_pc + 1);

            cpu.dec_pc(1);
        } else {
            cpu.inc_pc(1);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OB|OC|OH|OL|OF|OWZ, OB|OC|OH|OL|OF|OWZ)
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

impl Instruction for DecMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let addr = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        cpu.contend_read_no_mreq(addr);

        let res = memval.wrapping_sub(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_dec8(cpu, memval, res);

        info!("{:#06x}: DEC (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for DecMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let addr = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        cpu.contend_read_no_mreq(addr);

        let res = memval.wrapping_sub(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_dec8(cpu, memval, res);

        info!("{:#06x}: DEC (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
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

            let offset = cpu.zero_cycle_read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            info!("{:#06x}: DJNZ {:#06X}", cpu.get_pc(), target);
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
struct ExMemSpIx;
struct ExMemSpIy;
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

impl Instruction for ExMemSpIx {
    fn execute(&self, cpu: &mut Cpu) {
        let sp = cpu.read_reg16(Reg16::SP);
        let ix = cpu.read_reg16(Reg16::IX);

        let (ixhigh, ixlow) = (((ix & 0xFF00) >> 8) as u8,
                               ((ix & 0x00FF)       as u8));
        let memval = (cpu.read_word(sp    ) as u16) |
                    ((cpu.read_word(sp + 1) as u16) << 8);

        cpu.contend_read_no_mreq(sp + 1);

        cpu.write_reg16(Reg16::IX, memval);

        cpu.write_word(sp + 1, ixhigh);
        cpu.write_word(sp, ixlow);

        cpu.contend_write_no_mreq(sp);
        cpu.contend_write_no_mreq(sp);

        info!("{:#06x}: EX (SP), IX", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OIX, OIX)
    }
}

impl Instruction for ExMemSpIy {
    fn execute(&self, cpu: &mut Cpu) {
        let sp = cpu.read_reg16(Reg16::SP);
        let iy = cpu.read_reg16(Reg16::IY);

        let (iyhigh, iylow) = (((iy & 0xFF00) >> 8) as u8,
                               ((iy & 0x00FF)       as u8));
        let memval = (cpu.read_word(sp    ) as u16) |
                    ((cpu.read_word(sp + 1) as u16) << 8);

        cpu.contend_read_no_mreq(sp + 1);

        cpu.write_reg16(Reg16::IY, memval);

        cpu.write_word(sp + 1, iyhigh);
        cpu.write_word(sp, iylow);

        cpu.contend_write_no_mreq(sp);
        cpu.contend_write_no_mreq(sp);

        info!("{:#06x}: EX (SP), IY", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OIY, OIY)
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

struct Im { mode: u8 }

impl Instruction for Im {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_im(self.mode);

        info!("{:#06x}: IM {}", cpu.get_pc() - 1, self.mode);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


struct InAPortN ;
struct InRPortC { r: Reg8 }
struct InPortC  ;
struct Ini      ;
struct Inir     ;
struct Ind      ;
struct Indr     ;

#[inline(always)]
fn update_flags_in(cpu: &mut Cpu, portval: u8) {
    cpu.cond_flag  ( SIGN_FLAG            , portval & 0x80 != 0           );
    cpu.cond_flag  ( ZERO_FLAG            , portval == 0                  );
    cpu.clear_flag ( HALF_CARRY_FLAG                                      );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , portval.count_ones() % 2 == 0 );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                                    );
    cpu.cond_flag  ( X_FLAG               , portval & 0x08 != 0           );
    cpu.cond_flag  ( Y_FLAG               , portval & 0x20 != 0           );
}

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

impl Instruction for InRPortC {
    fn execute(&self, cpu: &mut Cpu) {
        let port = cpu.read_reg16(Reg16::BC);

        let portval = cpu.read_port(port);

        cpu.write_reg8(self.r, portval);

        update_flags_in(cpu, portval);

        info!("{:#06x}: IN {:?}, (C)", cpu.get_pc() - 1, self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.r)|OC, OutputRegisters::from(self.r))
    }
}

impl Instruction for InPortC {
    fn execute(&self, cpu: &mut Cpu) {
        let port = cpu.read_reg16(Reg16::BC);

        let portval = cpu.read_port(port);

        update_flags_in(cpu, portval);

        info!("{:#06x}: IN (C)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OC, OF)
    }
}

// TODO
impl Instruction for Ini {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: INI", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}

// TODO
impl Instruction for Inir {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: INIR", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}

// TODO
impl Instruction for Ind {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: IND", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}

// TODO
impl Instruction for Indr {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: INDR", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
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

impl Instruction for IncMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let addr = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        cpu.contend_read_no_mreq(addr);

        let res = memval.wrapping_add(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_inc8(cpu, memval, res);

        info!("{:#06x}: INC (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for IncMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let addr = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        cpu.contend_read_no_mreq(addr);

        let res = memval.wrapping_add(1);
        cpu.write_reg16(Reg16::WZ, addr);

        cpu.write_word(addr, res);

        update_flags_inc8(cpu, memval, res);

        info!("{:#06x}: INC (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
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
struct JpIx   ;
struct JpIy   ;
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

impl Instruction for JpIx {
    fn execute(&self, cpu: &mut Cpu) {
        let ix = cpu.read_reg16(Reg16::IX);

        info!("{:#06x}: JP IX", cpu.get_pc());
        cpu.set_pc(ix);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX, ONONE)
    }
}

impl Instruction for JpIy {
    fn execute(&self, cpu: &mut Cpu) {
        let iy = cpu.read_reg16(Reg16::IY);

        info!("{:#06x}: JP IY", cpu.get_pc());
        cpu.set_pc(iy);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY, ONONE)
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

            let nn =  (cpu.zero_cycle_read_word(curr_pc + 1) as u16) |
                     ((cpu.zero_cycle_read_word(curr_pc + 2) as u16) << 8);

            info!("{:#06x}: JP {:?}, {:#06X}", cpu.get_pc(), self.cond, nn);
            cpu.inc_pc(3);

            cpu.write_reg16(Reg16::WZ, nn);
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

            let offset = cpu.zero_cycle_read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            info!("{:#06x}: JR Z, {:#06X}", cpu.get_pc(), target);
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

            let offset = cpu.zero_cycle_read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            info!("{:#06x}: JR NZ, {:#06X}", cpu.get_pc(), target);
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

            let offset = cpu.zero_cycle_read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            info!("{:#06x}: JR NC, {:#06X}", cpu.get_pc(), target);
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

            let offset = cpu.zero_cycle_read_word(curr_pc + 1) as i8 + 2;
            let target = (cpu.get_pc() as i16 + offset as i16) as u16;

            info!("{:#06x}: JR C, {:#06X}", cpu.get_pc(), target);
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
struct LdAMemBc  ;
struct LdAMemDe  ;
struct LdAMemNn  ;
struct LdDdMemNn { r: Reg16 }
struct LdDdNn    { r: Reg16 }
struct LdRN      { r: Reg8  }
struct LdHlMemNn ;
struct LdRMemIxD { r: Reg8  }
struct LdRMemIyD { r: Reg8  }
struct LdSpHl    ;
struct LdSpIx    ;
struct LdSpIy    ;
struct LdRR      { rt: Reg8, rs: Reg8 }
struct LdRMemHl  { r: Reg8  }
struct LdIA      ;
struct LdRA      ;
struct LdAI      ;
struct LdAR      ;

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

impl Instruction for LdMemIxDN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let n    = cpu.read_word(curr_pc + 2);
        let addr = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 2);
        cpu.contend_read_no_mreq(curr_pc + 2);

        cpu.write_word(addr, n);
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: LD (IX{:+#04X}), {:#04X}", cpu.get_pc() - 1, d, n);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OWZ, OWZ)
    }
}

impl Instruction for LdMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let r    = cpu.read_reg8(self.r);
        let addr = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        cpu.write_word(addr, r);
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: LD (IX{:+#04X}), {:?}", cpu.get_pc() - 1, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OWZ, OWZ)
    }
}

impl Instruction for LdMemIyDN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let n    = cpu.read_word(curr_pc + 2);
        let addr = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 2);
        cpu.contend_read_no_mreq(curr_pc + 2);

        cpu.write_word(addr, n);
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: LD (IY{:+#04X}), {:#04X}", cpu.get_pc() - 1, d, n);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
    }
}

impl Instruction for LdMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d    = cpu.read_word(curr_pc + 1) as i8;
        let r    = cpu.read_reg8(self.r);
        let addr = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        cpu.write_word(addr, r);
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: LD (IY{:+#04X}), {:?}", cpu.get_pc() - 1, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
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

impl Instruction for LdMemNnDd {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let r = cpu.read_reg16(self.r);
        let (rhigh, rlow) = (((r & 0xFF00) >> 8) as u8,
                             ((r & 0x00FF)       as u8));
        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);

        cpu.write_word(nn, rlow);
        cpu.write_word(nn + 1, rhigh);
        cpu.write_reg16(Reg16::WZ, nn + 1);

        info!("{:#06x}: LD ({:#06X}), {:?}", cpu.get_pc() - 1, nn, self.r);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.r)|OWZ, OWZ)
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

impl Instruction for LdDdMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);
        let nnmemval = (cpu.read_word(nn    ) as u16) |
                      ((cpu.read_word(nn + 1) as u16) << 8);

        cpu.write_reg16(self.r, nnmemval);
        cpu.write_reg16(Reg16::WZ, nn + 1);

        info!("{:#06x}: LD {:?}, ({:#06X})", cpu.get_pc(), self.r, nn);
        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OutputRegisters::from(self.r)|OWZ, OutputRegisters::from(self.r)|OWZ)
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

impl Instruction for LdRMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        cpu.write_reg8(self.r, memval);
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: LD {:?}, (IX{:+#04X})", cpu.get_pc() - 1, self.r, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OWZ|OutputRegisters::from(self.r), OWZ|OutputRegisters::from(self.r))
    }
}

impl Instruction for LdRMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        cpu.write_reg8(self.r, memval);
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: LD {:?}, (IY{:+#04X})", cpu.get_pc() - 1, self.r, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ|OutputRegisters::from(self.r), OWZ|OutputRegisters::from(self.r))
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

impl Instruction for LdSpIx {
    fn execute(&self, cpu: &mut Cpu) {
        let ix = cpu.read_reg16(Reg16::IX);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(Reg16::SP, ix);

        info!("{:#06x}: LD SP, IX", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OIX, OSP)
    }
}

impl Instruction for LdSpIy {
    fn execute(&self, cpu: &mut Cpu) {
        let iy = cpu.read_reg16(Reg16::IY);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(Reg16::SP, iy);

        info!("{:#06x}: LD SP, IY", cpu.get_pc());
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OIY, OSP)
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

impl Instruction for LdIA {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg8(Reg8::I, a);

        info!("{:#06x}: LD I,A", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OI, OI)
    }
}

impl Instruction for LdRA {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg8(Reg8::R, a);

        info!("{:#06x}: LD R,A", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OR, OR)
    }
}

impl Instruction for LdAI {
    fn execute(&self, cpu: &mut Cpu) {
        let i = cpu.read_reg8(Reg8::I);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg8(Reg8::A, i);

        cpu.cond_flag  ( SIGN_FLAG            , i & 0x80 != 0  );
        cpu.cond_flag  ( ZERO_FLAG            , i == 0         );
        cpu.clear_flag ( HALF_CARRY_FLAG                       );
        let iff2 = cpu.get_iff2();
        cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , iff2           );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                     );

        info!("{:#06x}: LD A,I", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OI, OA)
    }
}

impl Instruction for LdAR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(Reg8::R);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg8(Reg8::A, r);

        info!("{:#06x}: LD A,R", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OR, OA)
    }
}


struct Ldd;
struct Lddr;

fn ldd(cpu: &mut Cpu) {
    let bc     = cpu.read_reg16(Reg16::BC);
    let de     = cpu.read_reg16(Reg16::DE);
    let hl     = cpu.read_reg16(Reg16::HL);
    let memval = cpu.read_word(hl);

    cpu.write_word(de, memval);

    cpu.contend_write_no_mreq(de);
    cpu.contend_write_no_mreq(de);

    cpu.write_reg16(Reg16::BC, bc.wrapping_sub(1));
    cpu.write_reg16(Reg16::DE, de.wrapping_sub(1));
    cpu.write_reg16(Reg16::HL, hl.wrapping_sub(1));

    cpu.clear_flag ( HALF_CARRY_FLAG                                );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , bc.wrapping_sub(1) != 0 );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                              );

    let a     = cpu.read_reg8(Reg8::A);
    let xyval = a.wrapping_add(memval);
    cpu.cond_flag  ( X_FLAG               , xyval & 0x08 != 0       );
    cpu.cond_flag  ( Y_FLAG               , xyval & 0x02 != 0       );
}

impl Instruction for Ldd {
    fn execute(&self, cpu: &mut Cpu) {
        ldd(cpu);

        info!("{:#06x}: LDD", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB|OC|OD|OE|OH|OL|OF, OB|OC|OD|OE|OH|OL|OF)
    }
}

impl Instruction for Lddr {
    fn execute(&self, cpu: &mut Cpu) {
        ldd(cpu);

        info!("{:#06x}: LDDR", cpu.get_pc() - 1);
        if cpu.get_flag(PARITY_OVERFLOW_FLAG) {
            let de = cpu.read_reg16(Reg16::DE);
            cpu.contend_write_no_mreq(de + 1);
            cpu.contend_write_no_mreq(de + 1);
            cpu.contend_write_no_mreq(de + 1);
            cpu.contend_write_no_mreq(de + 1);
            cpu.contend_write_no_mreq(de + 1);

            let curr_pc = cpu.get_pc();
            cpu.write_reg16(Reg16::WZ, curr_pc + 1);

            cpu.dec_pc(1);
        } else {
            cpu.inc_pc(1);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB|OC|OD|OE|OH|OL|OF|OWZ, OB|OC|OD|OE|OH|OL|OF|OWZ)
    }
}


struct Ldi;
struct Ldir;

fn ldi(cpu: &mut Cpu) {
    let bc     = cpu.read_reg16(Reg16::BC);
    let de     = cpu.read_reg16(Reg16::DE);
    let hl     = cpu.read_reg16(Reg16::HL);
    let memval = cpu.read_word(hl);

    cpu.write_word(de, memval);

    cpu.contend_write_no_mreq(de);
    cpu.contend_write_no_mreq(de);

    cpu.write_reg16(Reg16::BC, bc.wrapping_sub(1));
    cpu.write_reg16(Reg16::DE, de.wrapping_add(1));
    cpu.write_reg16(Reg16::HL, hl.wrapping_add(1));

    cpu.clear_flag ( HALF_CARRY_FLAG                                );
    cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , bc.wrapping_sub(1) != 0 );
    cpu.clear_flag ( ADD_SUBTRACT_FLAG                              );

    let a     = cpu.read_reg8(Reg8::A);
    let xyval = a.wrapping_add(memval);
    cpu.cond_flag  ( X_FLAG               , xyval & 0x08 != 0       );
    cpu.cond_flag  ( Y_FLAG               , xyval & 0x02 != 0       );
}

impl Instruction for Ldi {
    fn execute(&self, cpu: &mut Cpu) {
        ldi(cpu);

        info!("{:#06x}: LDI", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB|OC|OD|OE|OH|OL|OF, OB|OC|OD|OE|OH|OL|OF)
    }
}

impl Instruction for Ldir {
    fn execute(&self, cpu: &mut Cpu) {
        ldi(cpu);

        info!("{:#06x}: LDIR", cpu.get_pc() - 1);
        if cpu.get_flag(PARITY_OVERFLOW_FLAG) {
            let de = cpu.read_reg16(Reg16::DE);
            cpu.contend_write_no_mreq(de - 1);
            cpu.contend_write_no_mreq(de - 1);
            cpu.contend_write_no_mreq(de - 1);
            cpu.contend_write_no_mreq(de - 1);
            cpu.contend_write_no_mreq(de - 1);

            let curr_pc = cpu.get_pc();
            cpu.write_reg16(Reg16::WZ, curr_pc + 1);

            cpu.dec_pc(1);
        } else {
            cpu.inc_pc(1);
        }
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB|OC|OD|OE|OH|OL|OF|OWZ, OB|OC|OD|OE|OH|OL|OF|OWZ)
    }
}


struct Neg;

impl Instruction for Neg {
    fn execute(&self, cpu: &mut Cpu) {
        let a = cpu.read_reg8(Reg8::A);

        let neg = 0u8.wrapping_sub(a);

        cpu.write_reg8(Reg8::A, neg);

        update_flags_sub8(cpu, 0u8, a, neg);

        info!("{:#06x}: NEG", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF, OA|OF)
    }
}


struct OrR      { r: Reg8 }
struct OrN      ;
struct OrMemHl  ;
struct OrMemIxD ;
struct OrMemIyD ;

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

impl Instruction for OrMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a | memval;

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR A, (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for OrMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a | memval;

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: OR A, (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }
}


struct OutPortCR { r: Reg8 }
struct OutPortNA ;
struct OutPortC  ;
struct Outi      ;
struct Otir      ;
struct Outd      ;
struct Otdr      ;

impl Instruction for OutPortCR {
    fn execute(&self, cpu: &mut Cpu) {
        let r    = cpu.read_reg8(self.r);
        let port = cpu.read_reg16(Reg16::BC);

        cpu.write_port(port, r);

        info!("{:#06x}: OUT (C), {:?}", cpu.get_pc() - 1, self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB|OC|OutputRegisters::from(self.r), ONONE)
    }
}

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

impl Instruction for OutPortC {
    fn execute(&self, cpu: &mut Cpu) {
        let port = cpu.read_reg16(Reg16::BC);

        cpu.write_port(port, 0);

        info!("{:#06x}: OUT (C), 0", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OB|OC, ONONE)
    }
}

// TODO
impl Instruction for Outi {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: OUTI", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}

// TODO
impl Instruction for Otir {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: OTIR", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}

// TODO
impl Instruction for Outd {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: OUTI", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}

// TODO
impl Instruction for Otdr {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: OTIR", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


struct PopQq { r: Reg16 }

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


struct PushQq { r: Reg16 }

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


struct ResBR       { b: u8, r: Reg8 }
struct ResBMemIxD  { b: u8 }
struct ResBMemIyD  { b: u8 }
struct ResBMemHl   { b: u8 }
struct ResBMemIxDR { b: u8, r: Reg8 }
struct ResBMemIyDR { b: u8, r: Reg8 }

impl Instruction for ResBR {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);

        cpu.write_reg8(self.r, val & !(1 << self.b));

        info!("{:#06x}: RES {}, {:?}", cpu.get_pc() - 1, self.b, self.r);

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF)
    }
}

impl Instruction for ResBMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval & !(1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: RES {}, (IX{:+#04X})", cpu.get_pc() - 2, self.b, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OWZ, OWZ)
    }
}

impl Instruction for ResBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval & !(1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: RES {}, (IY{:+#04X})", cpu.get_pc() - 2, self.b, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
    }
}

impl Instruction for ResBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        cpu.write_word(hl, memval & !(1 << self.b));

        info!("{:#06x}: RES {}, (HL)", cpu.get_pc() - 1, self.b);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, ONONE)
    }
}

impl Instruction for ResBMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval & !(1 << self.b));
        cpu.write_word(addr, memval & !(1 << self.b));

        info!("{:#06x}: RES {}, (IX{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}

impl Instruction for ResBMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval & !(1 << self.b));
        cpu.write_word(addr, memval & !(1 << self.b));

        info!("{:#06x}: RES {}, (IY{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}


struct Ret   ;
struct RetN  ;
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

impl Instruction for RetN {
    fn execute(&self, cpu: &mut Cpu) {
        if cpu.get_iff2() { cpu.set_iff1(); } else { cpu.clear_iff1(); }

        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low  = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        info!("{:#06x}: RETN", cpu.get_pc() - 1);
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


struct RlR        { r: Reg8 }
struct RlMemHl    ;
struct RlMemIxD   ;
struct RlMemIyD   ;
struct RlA        ;
struct RlMemIxDR  { r: Reg8 }
struct RlMemIyDR  { r: Reg8 }
struct RlcR       { r: Reg8 }
struct RlcMemHl   ;
struct RlcMemIxD  ;
struct RlcMemIyD  ;
struct RlcA       ;
struct RlcMemIxDR { r: Reg8 }
struct RlcMemIyDR { r: Reg8 }

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

impl Instruction for RlR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let mut res = r.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x80 != 0 );

        info!("{:#06x}: RL {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RlMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let mut res = memval.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RL (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for RlMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RL (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for RlMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RL (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }
}

impl Instruction for RlMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RL (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RlMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RL (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RlcR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r.rotate_left(1);

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG           );
        cpu.cond_flag  ( CARRY_FLAG, r & 0x80 != 0 );

        info!("{:#06x}: RLC {:?}", cpu.get_pc() - 1, self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RlcMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval.rotate_left(1);

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RLC (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for RlcMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_left(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RLC (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for RlcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_left(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RLC (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
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

impl Instruction for RlcMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_left(1);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RLC (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RlcMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_left(1);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: RLC (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}


struct Rld;

impl Instruction for Rld {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);
        let alow   = a & 0x0F;

        let a = (a & 0xF0) | ((memval >> 4) & 0x0F);
        let memval = memval << 4 | alow;

        cpu.contend_read_no_mreq(hl);
        cpu.contend_read_no_mreq(hl);
        cpu.contend_read_no_mreq(hl);
        cpu.contend_read_no_mreq(hl);

        cpu.write_reg8(Reg8::A, a);
        cpu.write_word(hl, memval);
        cpu.write_reg16(Reg16::WZ, hl + 1);

        cpu.cond_flag  ( SIGN_FLAG            , a & 0x80 != 0           );
        cpu.cond_flag  ( ZERO_FLAG            , a == 0                  );
        cpu.clear_flag ( HALF_CARRY_FLAG                                );
        cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , a.count_ones() % 2 == 0 );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                              );
        cpu.cond_flag  ( X_FLAG               , a & 0x08 != 0           );
        cpu.cond_flag  ( Y_FLAG               , a & 0x20 != 0           );

        info!("{:#06x}: RLD", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OH|OL|OWZ, OA|OF|OWZ)
    }
}


struct RrR        { r: Reg8 }
struct RrMemHl    ;
struct RrMemIxD   ;
struct RrMemIyD   ;
struct RrA        ;
struct RrMemIxDR  { r: Reg8 }
struct RrMemIyDR  { r: Reg8 }
struct RrcR       { r: Reg8 }
struct RrcMemHl   ;
struct RrcMemIxD  ;
struct RrcMemIyD  ;
struct RrcA       ;
struct RrcMemIxDR { r: Reg8 }
struct RrcMemIyDR { r: Reg8 }

impl Instruction for RrR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let mut res = r.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG           );
        cpu.cond_flag  ( CARRY_FLAG, r & 0x01 != 0 );

        info!("{:#06x}: RR {:?}", cpu.get_pc() - 1, self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RrMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let mut res = memval.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RR (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for RrMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RR (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for RrMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RR (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }
}

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

impl Instruction for RrMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RR (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RrMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RR (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RrcR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r.rotate_right(1);

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG           );
        cpu.cond_flag  ( CARRY_FLAG, r & 0x01 != 0 );

        info!("{:#06x}: RRC {:?}", cpu.get_pc() - 1, self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RrcMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval.rotate_right(1);

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RRC (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for RrcMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_right(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RRC (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for RrcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_right(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RRC (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
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

impl Instruction for RrcMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_right(1);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RRC (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RrcMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_right(1);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: RRC (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}


struct Rrd;

impl Instruction for Rrd {
    fn execute(&self, cpu: &mut Cpu) {
        let a      = cpu.read_reg8(Reg8::A);
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);
        let alow   = a & 0x0F;

        let a = (a & 0xF0) | (memval & 0x0F);
        let memval = ((alow << 4) & 0xF0) | ((memval >> 4) & 0x0F);

        cpu.contend_read_no_mreq(hl);
        cpu.contend_read_no_mreq(hl);
        cpu.contend_read_no_mreq(hl);
        cpu.contend_read_no_mreq(hl);

        cpu.write_reg8(Reg8::A, a);
        cpu.write_word(hl, memval);
        cpu.write_reg16(Reg16::WZ, hl + 1);

        cpu.cond_flag  ( SIGN_FLAG            , a & 0x80 != 0           );
        cpu.cond_flag  ( ZERO_FLAG            , a == 0                  );
        cpu.clear_flag ( HALF_CARRY_FLAG                                );
        cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , a.count_ones() % 2 == 0 );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                              );
        cpu.cond_flag  ( X_FLAG               , a & 0x08 != 0           );
        cpu.cond_flag  ( Y_FLAG               , a & 0x20 != 0           );

        info!("{:#06x}: RRD", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OH|OL|OWZ, OA|OF|OWZ)
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


struct SetBR       { b: u8, r: Reg8 }
struct SetBMemIxD  { b: u8 }
struct SetBMemIyD  { b: u8 }
struct SetBMemHl   { b: u8 }
struct SetBMemIxDR { b: u8, r: Reg8 }
struct SetBMemIyDR { b: u8, r: Reg8 }

impl Instruction for SetBR {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);

        cpu.write_reg8(self.r, val | (1 << self.b));

        info!("{:#06x}: SET {}, {:?}", cpu.get_pc() - 1, self.b, self.r);

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF)
    }
}

impl Instruction for SetBMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval | (1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: SET {}, (IX{:+#04X})", cpu.get_pc() - 2, self.b, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OWZ, OWZ)
    }
}

impl Instruction for SetBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval | (1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        info!("{:#06x}: SET {}, (IY{:+#04X})", cpu.get_pc() - 2, self.b, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
    }
}

impl Instruction for SetBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        cpu.write_word(hl, memval | (1 << self.b));

        info!("{:#06x}: SET {}, (HL)", cpu.get_pc() - 1, self.b);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, ONONE)
    }
}

impl Instruction for SetBMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval | (1 << self.b));
        cpu.write_word(addr, memval | (1 << self.b));

        info!("{:#06x}: SET {}, (IX{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}

impl Instruction for SetBMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval | (1 << self.b));
        cpu.write_word(addr, memval | (1 << self.b));

        info!("{:#06x}: SET {}, (IY{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
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
    cpu.cond_flag ( HALF_CARRY_FLAG      , (op1 & 0x0FFF) < (op2 & 0x0FFF) + c                              );
    cpu.cond_flag ( PARITY_OVERFLOW_FLAG , (op1 & 0x8000 != op2 & 0x8000) && (op1 & 0x8000 != res & 0x8000) );
    cpu.set_flag  ( ADD_SUBTRACT_FLAG                                                                       );
    cpu.cond_flag ( CARRY_FLAG           , (op1 as u32) < ((op2 as u32) + (c as u32))                       );
    cpu.cond_flag ( X_FLAG               , (res >> 8) & 0x08 != 0                                           );
    cpu.cond_flag ( Y_FLAG               , (res >> 8) & 0x20 != 0                                           );
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

impl Instruction for SbcMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(memval).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_sbc8(cpu, a, memval, c, res);

        info!("{:#06x}: SBC A, (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for SbcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);
        let c      = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = a.wrapping_sub(memval).wrapping_sub(c);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_sbc8(cpu, a, memval, c, res);

        info!("{:#06x}: SBC A, (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for SbcHlSs {
    fn execute(&self, cpu: &mut Cpu) {
        let hl = cpu.read_reg16(Reg16::HL);
        let r  = cpu.read_reg16(self.r);
        let c = if cpu.get_flag(CARRY_FLAG) { 1 } else { 0 };

        let res = hl.wrapping_sub(r).wrapping_sub(c);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(Reg16::HL, res);
        cpu.write_reg16(Reg16::WZ, hl + 1);

        update_flags_sbc16(cpu, hl, r, c, res);

        info!("{:#06x}: SBC HL, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OF|OWZ|OutputRegisters::from(self.r), OH|OL|OF|OWZ)
    }
}


struct SlaR       { r: Reg8 }
struct SlaMemHl   ;
struct SlaMemIxD  ;
struct SlaMemIyD  ;
struct SlaMemIxDR { r: Reg8 }
struct SlaMemIyDR { r: Reg8 }

impl Instruction for SlaR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r << 1;

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x80 != 0 );

        info!("{:#06x}: SLA {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SlaMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval << 1;

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLA (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for SlaMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLA (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for SlaMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLA (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OF|OWZ)
    }
}

impl Instruction for SlaMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLA (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SlaMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLA (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}


struct SllR       { r: Reg8 }
struct SllMemHl   ;
struct SllMemIxD  ;
struct SllMemIyD  ;
struct SllMemIxDR { r: Reg8 }
struct SllMemIyDR { r: Reg8 }

impl Instruction for SllR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r << 1 | 0x1;

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x80 != 0 );

        info!("{:#06x}: SLL {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SllMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval << 1 | 0x1;

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLL (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for SllMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1 | 0x1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLL (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for SllMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1 | 0x1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLL (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }
}

impl Instruction for SllMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1 | 0x1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLL (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SllMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1 | 0x1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        info!("{:#06x}: SLL (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}


struct SraR       { r: Reg8 }
struct SraMemHl   ;
struct SraMemIxD  ;
struct SraMemIyD  ;
struct SraMemIxDR { r: Reg8 }
struct SraMemIyDR { r: Reg8 }

impl Instruction for SraR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r >> 1 | (r & 0x80);

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x01 != 0 );

        info!("{:#06x}: SRA {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SraMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval >> 1 | (memval & 0x80);

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRA (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for SraMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1 | (memval & 0x80);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRA (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for SraMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1 | (memval & 0x80);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRA (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }
}

impl Instruction for SraMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1 | (memval & 0x80);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRA (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SraMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1 | (memval & 0x80);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRA (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}


struct SrlR       { r: Reg8 }
struct SrlMemHl   ;
struct SrlMemIxD  ;
struct SrlMemIyD  ;
struct SrlMemIxDR { r: Reg8 }
struct SrlMemIyDR { r: Reg8 }

impl Instruction for SrlR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r >> 1;

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x01 != 0 );

        info!("{:#06x}: SRL {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SrlMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        let res = memval >> 1;

        cpu.write_word(hl, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRL (HL)", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }
}

impl Instruction for SrlMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRL (IX{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}

impl Instruction for SrlMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRL (IY{:+#04X})", cpu.get_pc() - 2, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }
}

impl Instruction for SrlMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRL (IX{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SrlMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let d      = cpu.zero_cycle_read_word(curr_pc) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        info!("{:#06x}: SRL (IY{:+#04X}), {:?}", cpu.get_pc() - 2, d, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
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

impl Instruction for SubMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a.wrapping_sub(memval);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_sub8(cpu, a, memval, res);

        info!("{:#06x}: SUB A, (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for SubMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a.wrapping_sub(memval);

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_sub8(cpu, a, memval, res);

        info!("{:#06x}: SUB A, (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }
}


struct XorR      { r: Reg8 }
struct XorN      ;
struct XorMemHl  ;
struct XorMemIxD ;
struct XorMemIyD ;

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

impl Instruction for XorMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IX) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a ^ memval;

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR A, (IX{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }
}

impl Instruction for XorMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();

        let a      = cpu.read_reg8(Reg8::A);
        let d      = cpu.read_word(curr_pc + 1) as i8;
        let addr   = ((cpu.read_reg16(Reg16::IY) as i16) + d as i16) as u16;

        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);
        cpu.contend_read_no_mreq(curr_pc + 1);

        let memval = cpu.read_word(addr);

        let res = a ^ memval;

        cpu.write_reg8(Reg8::A, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag(HALF_CARRY_FLAG);

        info!("{:#06x}: XOR A, (IY{:+#04X})", cpu.get_pc() - 1, d);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }
}

pub const INSTR_TABLE_CB: [&'static Instruction; 256] = [
    /* 0x00 */             /* 0x01 */             /* 0x02 */             /* 0x03 */             /* 0x04 */             /* 0x05 */             /* 0x06 */       /* 0x07 */
    &RlcR{r:Reg8::B}     , &RlcR{r:Reg8::C}     , &RlcR{r:Reg8::D}     , &RlcR{r:Reg8::E}     , &RlcR{r:Reg8::H}     , &RlcR{r:Reg8::L}     , &RlcMemHl      , &RlcR{r:Reg8::A}     ,

    /* 0x08 */             /* 0x09 */             /* 0x0A */             /* 0x0B */             /* 0x0C */             /* 0x0D */             /* 0x0E */       /* 0x0F */
    &RrcR{r:Reg8::B}     , &RrcR{r:Reg8::C}     , &RrcR{r:Reg8::D}     , &RrcR{r:Reg8::E}     , &RrcR{r:Reg8::H}     , &RrcR{r:Reg8::L}     , &RrcMemHl      , &RrcR{r:Reg8::A}     ,

    /* 0x10 */             /* 0x11 */             /* 0x12 */             /* 0x13 */             /* 0x14 */             /* 0x15 */             /* 0x16 */       /* 0x17 */
    &RlR{r:Reg8::B}      , &RlR{r:Reg8::C}      , &RlR{r:Reg8::D}      , &RlR{r:Reg8::E}      , &RlR{r:Reg8::H}      , &RlR{r:Reg8::L}      , &RlMemHl       , &RlR{r:Reg8::A}      ,

    /* 0x18 */             /* 0x19 */             /* 0x1A */             /* 0x1B */             /* 0x1C */             /* 0x1D */             /* 0x1E */       /* 0x1F */
    &RrR{r:Reg8::B}      , &RrR{r:Reg8::C}      , &RrR{r:Reg8::D}      , &RrR{r:Reg8::E}      , &RrR{r:Reg8::H}      , &RrR{r:Reg8::L}      , &RrMemHl       , &RrR{r:Reg8::A}      ,

    /* 0x20 */             /* 0x21 */             /* 0x22 */             /* 0x23 */             /* 0x24 */             /* 0x25 */             /* 0x26 */       /* 0x27 */
    &SlaR{r:Reg8::B}     , &SlaR{r:Reg8::C}     , &SlaR{r:Reg8::D}     , &SlaR{r:Reg8::E}     , &SlaR{r:Reg8::H}     , &SlaR{r:Reg8::L}     , &SlaMemHl      , &SlaR{r:Reg8::A}     ,

    /* 0x28 */             /* 0x29 */             /* 0x2A */             /* 0x2B */             /* 0x2C */             /* 0x2D */             /* 0x2E */       /* 0x2F */
    &SraR{r:Reg8::B}     , &SraR{r:Reg8::C}     , &SraR{r:Reg8::D}     , &SraR{r:Reg8::E}     , &SraR{r:Reg8::H}     , &SraR{r:Reg8::L}     , &SraMemHl      , &SraR{r:Reg8::A}     ,

    /* 0x30 */             /* 0x31 */             /* 0x32 */             /* 0x33 */             /* 0x34 */             /* 0x35 */             /* 0x36 */       /* 0x37 */
    &SllR{r:Reg8::B}     , &SllR{r:Reg8::C}     , &SllR{r:Reg8::D}     , &SllR{r:Reg8::E}     , &SllR{r:Reg8::H}     , &SllR{r:Reg8::L}     , &SllMemHl      , &SllR{r:Reg8::A}     ,

    /* 0x38 */             /* 0x39 */             /* 0x3A */             /* 0x3B */             /* 0x3C */             /* 0x3D */             /* 0x3E */       /* 0x3F */
    &SrlR{r:Reg8::B}     , &SrlR{r:Reg8::C}     , &SrlR{r:Reg8::D}     , &SrlR{r:Reg8::E}     , &SrlR{r:Reg8::H}     , &SrlR{r:Reg8::L}     , &SrlMemHl      , &SrlR{r:Reg8::A}     ,

    /* 0x40 */             /* 0x41 */             /* 0x42 */             /* 0x43 */             /* 0x44 */             /* 0x45 */             /* 0x46 */       /* 0x47 */
    &BitBR{b:0,r:Reg8::B}, &BitBR{b:0,r:Reg8::C}, &BitBR{b:0,r:Reg8::D}, &BitBR{b:0,r:Reg8::E}, &BitBR{b:0,r:Reg8::H}, &BitBR{b:0,r:Reg8::L}, &BitBMemHl{b:0}, &BitBR{b:0,r:Reg8::A},

    /* 0x48 */             /* 0x49 */             /* 0x4A */             /* 0x4B */             /* 0x4C */             /* 0x4D */             /* 0x4E */       /* 0x4F */
    &BitBR{b:1,r:Reg8::B}, &BitBR{b:1,r:Reg8::C}, &BitBR{b:1,r:Reg8::D}, &BitBR{b:1,r:Reg8::E}, &BitBR{b:1,r:Reg8::H}, &BitBR{b:1,r:Reg8::L}, &BitBMemHl{b:1}, &BitBR{b:1,r:Reg8::A},

    /* 0x50 */             /* 0x51 */             /* 0x52 */             /* 0x53 */             /* 0x54 */             /* 0x55 */             /* 0x56 */       /* 0x57 */
    &BitBR{b:2,r:Reg8::B}, &BitBR{b:2,r:Reg8::C}, &BitBR{b:2,r:Reg8::D}, &BitBR{b:2,r:Reg8::E}, &BitBR{b:2,r:Reg8::H}, &BitBR{b:2,r:Reg8::L}, &BitBMemHl{b:2}, &BitBR{b:2,r:Reg8::A},

    /* 0x58 */             /* 0x59 */             /* 0x5A */             /* 0x5B */             /* 0x5C */             /* 0x5D */             /* 0x5E */       /* 0x5F */
    &BitBR{b:3,r:Reg8::B}, &BitBR{b:3,r:Reg8::C}, &BitBR{b:3,r:Reg8::D}, &BitBR{b:3,r:Reg8::E}, &BitBR{b:3,r:Reg8::H}, &BitBR{b:3,r:Reg8::L}, &BitBMemHl{b:3}, &BitBR{b:3,r:Reg8::A},

    /* 0x60 */             /* 0x61 */             /* 0x62 */             /* 0x63 */             /* 0x64 */             /* 0x65 */             /* 0x66 */       /* 0x67 */
    &BitBR{b:4,r:Reg8::B}, &BitBR{b:4,r:Reg8::C}, &BitBR{b:4,r:Reg8::D}, &BitBR{b:4,r:Reg8::E}, &BitBR{b:4,r:Reg8::H}, &BitBR{b:4,r:Reg8::L}, &BitBMemHl{b:4}, &BitBR{b:4,r:Reg8::A},

    /* 0x68 */             /* 0x69 */             /* 0x6A */             /* 0x6B */             /* 0x6C */             /* 0x6D */             /* 0x6E */       /* 0x6F */
    &BitBR{b:5,r:Reg8::B}, &BitBR{b:5,r:Reg8::C}, &BitBR{b:5,r:Reg8::D}, &BitBR{b:5,r:Reg8::E}, &BitBR{b:5,r:Reg8::H}, &BitBR{b:5,r:Reg8::L}, &BitBMemHl{b:5}, &BitBR{b:5,r:Reg8::A},

    /* 0x70 */             /* 0x71 */             /* 0x72 */             /* 0x73 */             /* 0x74 */             /* 0x75 */             /* 0x76 */       /* 0x77 */
    &BitBR{b:6,r:Reg8::B}, &BitBR{b:6,r:Reg8::C}, &BitBR{b:6,r:Reg8::D}, &BitBR{b:6,r:Reg8::E}, &BitBR{b:6,r:Reg8::H}, &BitBR{b:6,r:Reg8::L}, &BitBMemHl{b:6}, &BitBR{b:6,r:Reg8::A},

    /* 0x78 */             /* 0x79 */             /* 0x7A */             /* 0x7B */             /* 0x7C */             /* 0x7D */             /* 0x7E */       /* 0x7F */
    &BitBR{b:7,r:Reg8::B}, &BitBR{b:7,r:Reg8::C}, &BitBR{b:7,r:Reg8::D}, &BitBR{b:7,r:Reg8::E}, &BitBR{b:7,r:Reg8::H}, &BitBR{b:7,r:Reg8::L}, &BitBMemHl{b:7}, &BitBR{b:7,r:Reg8::A},

    /* 0x80 */             /* 0x81 */             /* 0x82 */             /* 0x83 */             /* 0x84 */             /* 0x85 */             /* 0x86 */       /* 0x87 */
    &ResBR{b:0,r:Reg8::B}, &ResBR{b:0,r:Reg8::C}, &ResBR{b:0,r:Reg8::D}, &ResBR{b:0,r:Reg8::E}, &ResBR{b:0,r:Reg8::H}, &ResBR{b:0,r:Reg8::L}, &ResBMemHl{b:0}, &ResBR{b:0,r:Reg8::A},

    /* 0x88 */             /* 0x89 */             /* 0x8A */             /* 0x8B */             /* 0x8C */             /* 0x8D */             /* 0x8E */       /* 0x8F */
    &ResBR{b:1,r:Reg8::B}, &ResBR{b:1,r:Reg8::C}, &ResBR{b:1,r:Reg8::D}, &ResBR{b:1,r:Reg8::E}, &ResBR{b:1,r:Reg8::H}, &ResBR{b:1,r:Reg8::L}, &ResBMemHl{b:1}, &ResBR{b:1,r:Reg8::A},

    /* 0x90 */             /* 0x91 */             /* 0x92 */             /* 0x93 */             /* 0x94 */             /* 0x95 */             /* 0x96 */       /* 0x97 */
    &ResBR{b:2,r:Reg8::B}, &ResBR{b:2,r:Reg8::C}, &ResBR{b:2,r:Reg8::D}, &ResBR{b:2,r:Reg8::E}, &ResBR{b:2,r:Reg8::H}, &ResBR{b:2,r:Reg8::L}, &ResBMemHl{b:2}, &ResBR{b:2,r:Reg8::A},

    /* 0x98 */             /* 0x99 */             /* 0x9A */             /* 0x9B */             /* 0x9C */             /* 0x9D */             /* 0x9E */       /* 0x9F */
    &ResBR{b:3,r:Reg8::B}, &ResBR{b:3,r:Reg8::C}, &ResBR{b:3,r:Reg8::D}, &ResBR{b:3,r:Reg8::E}, &ResBR{b:3,r:Reg8::H}, &ResBR{b:3,r:Reg8::L}, &ResBMemHl{b:3}, &ResBR{b:3,r:Reg8::A},

    /* 0xA0 */             /* 0xA1 */             /* 0xA2 */             /* 0xA3 */             /* 0xA4 */             /* 0xA5 */             /* 0xA6 */       /* 0xA7 */
    &ResBR{b:4,r:Reg8::B}, &ResBR{b:4,r:Reg8::C}, &ResBR{b:4,r:Reg8::D}, &ResBR{b:4,r:Reg8::E}, &ResBR{b:4,r:Reg8::H}, &ResBR{b:4,r:Reg8::L}, &ResBMemHl{b:4}, &ResBR{b:4,r:Reg8::A},

    /* 0xA8 */             /* 0xA9 */             /* 0xAA */             /* 0xAB */             /* 0xAC */             /* 0xAD */             /* 0xAE */       /* 0xAF */
    &ResBR{b:5,r:Reg8::B}, &ResBR{b:5,r:Reg8::C}, &ResBR{b:5,r:Reg8::D}, &ResBR{b:5,r:Reg8::E}, &ResBR{b:5,r:Reg8::H}, &ResBR{b:5,r:Reg8::L}, &ResBMemHl{b:5}, &ResBR{b:5,r:Reg8::A},

    /* 0xB0 */             /* 0xB1 */             /* 0xB2 */             /* 0xB3 */             /* 0xB4 */             /* 0xB5 */             /* 0xB6 */       /* 0xB7 */
    &ResBR{b:6,r:Reg8::B}, &ResBR{b:6,r:Reg8::C}, &ResBR{b:6,r:Reg8::D}, &ResBR{b:6,r:Reg8::E}, &ResBR{b:6,r:Reg8::H}, &ResBR{b:6,r:Reg8::L}, &ResBMemHl{b:6}, &ResBR{b:6,r:Reg8::A},

    /* 0xB8 */             /* 0xB9 */             /* 0xBA */             /* 0xBB */             /* 0xBC */             /* 0xBD */             /* 0xBE */       /* 0xBF */
    &ResBR{b:7,r:Reg8::B}, &ResBR{b:7,r:Reg8::C}, &ResBR{b:7,r:Reg8::D}, &ResBR{b:7,r:Reg8::E}, &ResBR{b:7,r:Reg8::H}, &ResBR{b:7,r:Reg8::L}, &ResBMemHl{b:7}, &ResBR{b:7,r:Reg8::A},

    /* 0xC0 */             /* 0xC1 */             /* 0xC2 */             /* 0xC3 */             /* 0xC4 */             /* 0xC5 */             /* 0xC6 */       /* 0xC7 */
    &SetBR{b:0,r:Reg8::B}, &SetBR{b:0,r:Reg8::C}, &SetBR{b:0,r:Reg8::D}, &SetBR{b:0,r:Reg8::E}, &SetBR{b:0,r:Reg8::H}, &SetBR{b:0,r:Reg8::L}, &SetBMemHl{b:0}, &SetBR{b:0,r:Reg8::A},

    /* 0xC8 */             /* 0xC9 */             /* 0xCA */             /* 0xCB */             /* 0xCC */             /* 0xCD */             /* 0xCE */       /* 0xCF */
    &SetBR{b:1,r:Reg8::B}, &SetBR{b:1,r:Reg8::C}, &SetBR{b:1,r:Reg8::D}, &SetBR{b:1,r:Reg8::E}, &SetBR{b:1,r:Reg8::H}, &SetBR{b:1,r:Reg8::L}, &SetBMemHl{b:1}, &SetBR{b:1,r:Reg8::A},

    /* 0xD0 */             /* 0xD1 */             /* 0xD2 */             /* 0xD3 */             /* 0xD4 */             /* 0xD5 */             /* 0xD6 */       /* 0xD7 */
    &SetBR{b:2,r:Reg8::B}, &SetBR{b:2,r:Reg8::C}, &SetBR{b:2,r:Reg8::D}, &SetBR{b:2,r:Reg8::E}, &SetBR{b:2,r:Reg8::H}, &SetBR{b:2,r:Reg8::L}, &SetBMemHl{b:2}, &SetBR{b:2,r:Reg8::A},

    /* 0xD8 */             /* 0xD9 */             /* 0xDA */             /* 0xDB */             /* 0xDC */             /* 0xDD */             /* 0xDE */       /* 0xDF */
    &SetBR{b:3,r:Reg8::B}, &SetBR{b:3,r:Reg8::C}, &SetBR{b:3,r:Reg8::D}, &SetBR{b:3,r:Reg8::E}, &SetBR{b:3,r:Reg8::H}, &SetBR{b:3,r:Reg8::L}, &SetBMemHl{b:3}, &SetBR{b:3,r:Reg8::A},

    /* 0xE0 */             /* 0xE1 */             /* 0xE2 */             /* 0xE3 */             /* 0xE4 */             /* 0xE5 */             /* 0xE6 */       /* 0xE7 */
    &SetBR{b:4,r:Reg8::B}, &SetBR{b:4,r:Reg8::C}, &SetBR{b:4,r:Reg8::D}, &SetBR{b:4,r:Reg8::E}, &SetBR{b:4,r:Reg8::H}, &SetBR{b:4,r:Reg8::L}, &SetBMemHl{b:4}, &SetBR{b:4,r:Reg8::A},

    /* 0xE8 */             /* 0xE9 */             /* 0xEA */             /* 0xEB */             /* 0xEC */             /* 0xED */             /* 0xEE */       /* 0xEF */
    &SetBR{b:5,r:Reg8::B}, &SetBR{b:5,r:Reg8::C}, &SetBR{b:5,r:Reg8::D}, &SetBR{b:5,r:Reg8::E}, &SetBR{b:5,r:Reg8::H}, &SetBR{b:5,r:Reg8::L}, &SetBMemHl{b:5}, &SetBR{b:5,r:Reg8::A},

    /* 0xF0 */             /* 0xF1 */             /* 0xF2 */             /* 0xF3 */             /* 0xF4 */             /* 0xF5 */             /* 0xF6 */       /* 0xF7 */
    &SetBR{b:6,r:Reg8::B}, &SetBR{b:6,r:Reg8::C}, &SetBR{b:6,r:Reg8::D}, &SetBR{b:6,r:Reg8::E}, &SetBR{b:6,r:Reg8::H}, &SetBR{b:6,r:Reg8::L}, &SetBMemHl{b:6}, &SetBR{b:6,r:Reg8::A},

    /* 0xF8 */             /* 0xF9 */             /* 0xFA */             /* 0xFB */             /* 0xFC */             /* 0xFD */             /* 0xFE */       /* 0xFF */
    &SetBR{b:7,r:Reg8::B}, &SetBR{b:7,r:Reg8::C}, &SetBR{b:7,r:Reg8::D}, &SetBR{b:7,r:Reg8::E}, &SetBR{b:7,r:Reg8::H}, &SetBR{b:7,r:Reg8::L}, &SetBMemHl{b:7}, &SetBR{b:7,r:Reg8::A}
];

pub const INSTR_TABLE_DD: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */             /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Nop        , &Unsupported,          &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */             /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &AddIxPp{r:Reg16::BC}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */             /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */             /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &AddIxPp{r:Reg16::DE}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */             /* 0x22 */                 /* 0x23 */           /* 0x24 */          /* 0x25 */          /* 0x26 */          /* 0x27 */
    &Unsupported, &LdDdNn{r:Reg16::IX} , &LdMemNnDd{r:Reg16::IX}  , &IncSs{r:Reg16::IX}, &IncR{r:Reg8::IXH}, &DecR{r:Reg8::IXH}, &LdRN{r:Reg8::IXH}, &Unsupported,

    /* 0x28 */    /* 0x29 */             /* 0x2A */                 /* 0x2B */           /* 0x2C */          /* 0x2D */          /* 0x2E */          /* 0x2F */
    &Unsupported, &AddIxPp{r:Reg16::IX}, &LdDdMemNn{r:Reg16::IX}  , &DecSs{r:Reg16::IX}, &IncR{r:Reg8::IXL}, &DecR{r:Reg8::IXL}, &LdRN{r:Reg8::IXL}, &Unsupported,

    /* 0x30 */    /* 0x31 */             /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &IncMemIxD  , &DecMemIxD  , &LdMemIxDN  , &Unsupported,

    /* 0x38 */    /* 0x39 */             /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &AddIxPp{r:Reg16::SP}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */          /* 0x41 */        /* 0x42 */         /* 0x43 */         /* 0x44 */                      /* 0x45 */                      /* 0x46 */             /* 0x47 */
    INSTR_TABLE[0x40], INSTR_TABLE[0x41], INSTR_TABLE[0x42], INSTR_TABLE[0x43], &LdRR{rt:Reg8::B,rs:Reg8::IXH}, &LdRR{rt:Reg8::B,rs:Reg8::IXL}, &LdRMemIxD{r:Reg8::B}, INSTR_TABLE[0x47],

    /* 0x48 */          /* 0x49 */        /* 0x4A */         /* 0x4B */         /* 0x4C */                      /* 0x4D */                      /* 0x4E */             /* 0x4F */
    INSTR_TABLE[0x48], INSTR_TABLE[0x49], INSTR_TABLE[0x4A], INSTR_TABLE[0x4B], &LdRR{rt:Reg8::C,rs:Reg8::IXH}, &LdRR{rt:Reg8::C,rs:Reg8::IXL}, &LdRMemIxD{r:Reg8::C}, INSTR_TABLE[0x4F],

    /* 0x50 */          /* 0x51 */        /* 0x52 */         /* 0x53 */         /* 0x54 */                      /* 0x55 */                      /* 0x56 */             /* 0x57 */
    INSTR_TABLE[0x50], INSTR_TABLE[0x51], INSTR_TABLE[0x52], INSTR_TABLE[0x53], &LdRR{rt:Reg8::D,rs:Reg8::IXH}, &LdRR{rt:Reg8::D,rs:Reg8::IXL}, &LdRMemIxD{r:Reg8::D}, INSTR_TABLE[0x57],

    /* 0x58 */          /* 0x59 */        /* 0x5A */         /* 0x5B */         /* 0x5C */                      /* 0x5D */                      /* 0x5E */             /* 0x5F */
    INSTR_TABLE[0x58], INSTR_TABLE[0x59], INSTR_TABLE[0x5A], INSTR_TABLE[0x5B], &LdRR{rt:Reg8::E,rs:Reg8::IXH}, &LdRR{rt:Reg8::E,rs:Reg8::IXL}, &LdRMemIxD{r:Reg8::E}, INSTR_TABLE[0x5F],

    /* 0x60 */                        /* 0x61 */                        /* 0x62 */                      /* 0x63 */
    &LdRR{rt:Reg8::IXH,rs:Reg8::B}  , &LdRR{rt:Reg8::IXH,rs:Reg8::C}  , &LdRR{rt:Reg8::IXH,rs:Reg8::D}, &LdRR{rt:Reg8::IXH,rs:Reg8::E},

    /* 0x64 */                        /* 0x65 */                        /* 0x66 */                      /* 0x67 */
    &LdRR{rt:Reg8::IXH,rs:Reg8::IXH}, &LdRR{rt:Reg8::IXH,rs:Reg8::IXL}, &LdRMemIxD{r:Reg8::H}         , &LdRR{rt:Reg8::IXH,rs:Reg8::A},

    /* 0x68 */                        /* 0x69 */                        /* 0x6A */                      /* 0x6B */
    &LdRR{rt:Reg8::IXL,rs:Reg8::B}  , &LdRR{rt:Reg8::IXL,rs:Reg8::C}  , &LdRR{rt:Reg8::IXL,rs:Reg8::D}, &LdRR{rt:Reg8::IXL,rs:Reg8::E},

    /* 0x6C */                        /* 0x6D */                        /* 0x6E */                      /* 0x6F */
    &LdRR{rt:Reg8::IXL,rs:Reg8::IXH}, &LdRR{rt:Reg8::IXL,rs:Reg8::IXL}, &LdRMemIxD{r:Reg8::L}         , &LdRR{rt:Reg8::IXL,rs:Reg8::A},

    /* 0x70 */             /* 0x71 */             /* 0x72 */             /* 0x73 */             /* 0x74 */             /* 0x75 */             /* 0x76 */    /* 0x77 */
    &LdMemIxDR{r:Reg8::B}, &LdMemIxDR{r:Reg8::C}, &LdMemIxDR{r:Reg8::D}, &LdMemIxDR{r:Reg8::E}, &LdMemIxDR{r:Reg8::H}, &LdMemIxDR{r:Reg8::L}, &Unsupported, &LdMemIxDR{r:Reg8::A},

    /* 0x78 */          /* 0x79 */        /* 0x7A */         /* 0x7B */         /* 0x7C */                      /* 0x7D */                      /* 0x7E */             /* 0x7F */
    INSTR_TABLE[0x78], INSTR_TABLE[0x79], INSTR_TABLE[0x7A], INSTR_TABLE[0x7B], &LdRR{rt:Reg8::A,rs:Reg8::IXH}, &LdRR{rt:Reg8::A,rs:Reg8::IXL}, &LdRMemIxD{r:Reg8::A}, INSTR_TABLE[0x7F],

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */          /* 0x85 */          /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AddR{r:Reg8::IXH}, &AddR{r:Reg8::IXL}, &AddMemIxD  , &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */          /* 0x8D */          /* 0x8E */    /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AdcR{r:Reg8::IXH}, &AdcR{r:Reg8::IXL}, &AdcMemIxD  , &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */          /* 0x95 */          /* 0x96 */    /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SubR{r:Reg8::IXH}, &SubR{r:Reg8::IXL}, &SubMemIxD  , &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */          /* 0x9D */          /* 0x9E */    /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SbcR{r:Reg8::IXH}, &SbcR{r:Reg8::IXL}, &SbcMemIxD  , &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */          /* 0xA5 */          /* 0xA6 */    /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AndR{r:Reg8::IXH}, &AndR{r:Reg8::IXL}, &AndMemIxD  , &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */          /* 0xAD */          /* 0xAE */    /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &XorR{r:Reg8::IXH}, &XorR{r:Reg8::IXL}, &XorMemIxD  , &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */          /* 0xB5 */          /* 0xB6 */    /* 0xB7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &OrR{r:Reg8::IXH} , &OrR{r:Reg8::IXL} , &OrMemIxD   , &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */          /* 0xBD */          /* 0xBE */    /* 0xBF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &CpR{r:Reg8::IXH} , &CpR{r:Reg8::IXL} , &CpMemIxD   , &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */    /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */    /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */    /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */    /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE0 */    /* 0xE1 */           /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */            /* 0xE6 */    /* 0xE7 */
    &Unsupported, &PopQq{r:Reg16::IX}, &Unsupported, &ExMemSpIx  , &Unsupported, &PushQq{r:Reg16::IX}, &Unsupported, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */    /* 0xEF */
    &Unsupported, &JpIx       , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */    /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */    /* 0xFF */
    &Unsupported, &LdSpIx     , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported
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

    /* 0x40 */            /* 0x41 */             /* 0x42 */             /* 0x43 */               /* 0x44 */    /* 0x45 */    /* 0x46 */    /* 0x47 */
    &InRPortC{r:Reg8::B}, &OutPortCR{r:Reg8::B}, &SbcHlSs{r:Reg16::BC}, &LdMemNnDd{r:Reg16::BC}, &Neg        , &RetN       , &Im{mode:0} , &LdIA       ,

    /* 0x48 */            /* 0x49 */             /* 0x4A */             /* 0x4B */               /* 0x4C */    /* 0x4D */    /* 0x4E */    /* 0x4F */
    &InRPortC{r:Reg8::C}, &OutPortCR{r:Reg8::C}, &AdcHlSs{r:Reg16::BC}, &LdDdMemNn{r:Reg16::BC}, &Neg        , &RetN       , &Im{mode:0} , &LdRA       ,

    /* 0x50 */            /* 0x51 */             /* 0x52 */             /* 0x53 */               /* 0x54 */    /* 0x55 */    /* 0x56 */    /* 0x57 */
    &InRPortC{r:Reg8::D}, &OutPortCR{r:Reg8::D}, &SbcHlSs{r:Reg16::DE}, &LdMemNnDd{r:Reg16::DE}, &Neg        , &RetN       , &Im{mode:1} , &LdAI       ,

    /* 0x58 */            /* 0x59 */             /* 0x5A */             /* 0x5B */               /* 0x5C */    /* 0x5D */    /* 0x5E */    /* 0x5F */
    &InRPortC{r:Reg8::E}, &OutPortCR{r:Reg8::E}, &AdcHlSs{r:Reg16::DE}, &LdDdMemNn{r:Reg16::DE}, &Neg        , &RetN       , &Im{mode:2} , &LdAR       ,

    /* 0x60 */            /* 0x61 */             /* 0x62 */             /* 0x63 */               /* 0x64 */    /* 0x65 */    /* 0x66 */    /* 0x67 */
    &InRPortC{r:Reg8::H}, &OutPortCR{r:Reg8::H}, &SbcHlSs{r:Reg16::HL}, &LdMemNnDd{r:Reg16::HL}, &Neg        , &RetN       , &Im{mode:0} , &Rrd        ,

    /* 0x68 */            /* 0x69 */             /* 0x6A */             /* 0x6B */               /* 0x6C */    /* 0x6D */    /* 0x6E */    /* 0x6F */
    &InRPortC{r:Reg8::L}, &OutPortCR{r:Reg8::L}, &AdcHlSs{r:Reg16::HL}, &LdDdMemNn{r:Reg16::HL}, &Neg        , &RetN       , &Im{mode:0} , &Rld        ,

    /* 0x70 */            /* 0x71 */             /* 0x72 */             /* 0x73 */               /* 0x74 */    /* 0x75 */    /* 0x76 */    /* 0x77 */
    &InPortC            , &OutPortC            , &SbcHlSs{r:Reg16::SP}, &LdMemNnDd{r:Reg16::SP}, &Neg        , &RetN       , &Im{mode:1} , &Unsupported,

    /* 0x78 */            /* 0x79 */             /* 0x7A */             /* 0x7B */               /* 0x7C */    /* 0x7D */    /* 0x7E */    /* 0x7F */
    &InRPortC{r:Reg8::A}, &OutPortCR{r:Reg8::A}, &AdcHlSs{r:Reg16::SP}, &LdDdMemNn{r:Reg16::SP}, &Neg        , &RetN       , &Im{mode:2} , &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */    /* 0x8D */    /* 0x8E */    /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */    /* 0x95 */    /* 0x96 */    /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */    /* 0x9D */    /* 0x9E */    /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */    /* 0xA5 */    /* 0xA6 */    /* 0xA7 */
    &Ldi        , &Cpi        , &Ini        , &Outi       , &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */    /* 0xAD */    /* 0xAE */    /* 0xAF */
    &Ldd        , &Cpd        , &Ind        , &Outd       , &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */    /* 0xB5 */    /* 0xB6 */    /* 0xB7 */
    &Ldir       , &Cpir       , &Inir       , &Otir       , &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */    /* 0xBD */    /* 0xBE */    /* 0xBF */
    &Lddr       , &Cpdr       , &Indr       , &Otdr       , &Unsupported, &Unsupported, &Unsupported, &Unsupported,

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

    /* 0x20 */    /* 0x21 */             /* 0x22 */               /* 0x23 */           /* 0x24 */          /* 0x25 */          /* 0x26 */          /* 0x27 */
    &Unsupported, &LdDdNn{r:Reg16::IY} , &LdMemNnDd{r:Reg16::IY}, &IncSs{r:Reg16::IY}, &IncR{r:Reg8::IYH}, &DecR{r:Reg8::IYH}, &LdRN{r:Reg8::IYH}, &Unsupported,

    /* 0x28 */    /* 0x29 */             /* 0x2A */               /* 0x2B */           /* 0x2C */          /* 0x2D */          /* 0x2E */          /* 0x2F */
    &Unsupported, &AddIyRr{r:Reg16::IY}, &LdDdMemNn{r:Reg16::IY}, &DecSs{r:Reg16::IY}, &IncR{r:Reg8::IYL}, &DecR{r:Reg8::IYL}, &LdRN{r:Reg8::IYL}, &Unsupported,

    /* 0x30 */    /* 0x31 */             /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported,          &Unsupported, &Unsupported, &IncMemIyD  , &DecMemIyD  , &LdMemIyDN,   &Unsupported,

    /* 0x38 */    /* 0x39 */             /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &AddIyRr{r:Reg16::SP}, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x40 */          /* 0x41 */        /* 0x42 */         /* 0x43 */         /* 0x44 */                      /* 0x45 */                      /* 0x46 */             /* 0x47 */
    INSTR_TABLE[0x40], INSTR_TABLE[0x41], INSTR_TABLE[0x42], INSTR_TABLE[0x43], &LdRR{rt:Reg8::B,rs:Reg8::IYH}, &LdRR{rt:Reg8::B,rs:Reg8::IYL}, &LdRMemIyD{r:Reg8::B}, INSTR_TABLE[0x47],

    /* 0x48 */          /* 0x49 */        /* 0x4A */         /* 0x4B */         /* 0x4C */                      /* 0x4D */                      /* 0x4E */             /* 0x4F */
    INSTR_TABLE[0x48], INSTR_TABLE[0x49], INSTR_TABLE[0x4A], INSTR_TABLE[0x4B], &LdRR{rt:Reg8::C,rs:Reg8::IYH}, &LdRR{rt:Reg8::C,rs:Reg8::IYL}, &LdRMemIyD{r:Reg8::C}, INSTR_TABLE[0x4F],

    /* 0x50 */          /* 0x51 */        /* 0x52 */         /* 0x53 */         /* 0x54 */                      /* 0x55 */                      /* 0x56 */             /* 0x57 */
    INSTR_TABLE[0x50], INSTR_TABLE[0x51], INSTR_TABLE[0x52], INSTR_TABLE[0x53], &LdRR{rt:Reg8::D,rs:Reg8::IYH}, &LdRR{rt:Reg8::D,rs:Reg8::IYL}, &LdRMemIyD{r:Reg8::D}, INSTR_TABLE[0x57],

    /* 0x58 */          /* 0x59 */        /* 0x5A */         /* 0x5B */         /* 0x5C */                      /* 0x5D */                      /* 0x5E */             /* 0x5F */
    INSTR_TABLE[0x58], INSTR_TABLE[0x59], INSTR_TABLE[0x5A], INSTR_TABLE[0x5B], &LdRR{rt:Reg8::E,rs:Reg8::IYH}, &LdRR{rt:Reg8::E,rs:Reg8::IYL}, &LdRMemIyD{r:Reg8::E}, INSTR_TABLE[0x5F],

    /* 0x60 */                        /* 0x61 */                        /* 0x62 */                      /* 0x63 */
    &LdRR{rt:Reg8::IYH,rs:Reg8::B}  , &LdRR{rt:Reg8::IYH,rs:Reg8::C}  , &LdRR{rt:Reg8::IYH,rs:Reg8::D}, &LdRR{rt:Reg8::IYH,rs:Reg8::E},

    /* 0x64 */                        /* 0x65 */                        /* 0x66 */                      /* 0x67 */
    &LdRR{rt:Reg8::IYH,rs:Reg8::IYH}, &LdRR{rt:Reg8::IYH,rs:Reg8::IYL}, &LdRMemIyD{r:Reg8::H}         , &LdRR{rt:Reg8::IYH,rs:Reg8::A},

    /* 0x68 */                        /* 0x69 */                        /* 0x6A */                      /* 0x6B */
    &LdRR{rt:Reg8::IYL,rs:Reg8::B}  , &LdRR{rt:Reg8::IYL,rs:Reg8::C}  , &LdRR{rt:Reg8::IYL,rs:Reg8::D}, &LdRR{rt:Reg8::IYL,rs:Reg8::E},

    /* 0x6C */                        /* 0x6D */                        /* 0x6E */                      /* 0x6F */
    &LdRR{rt:Reg8::IYL,rs:Reg8::IYH}, &LdRR{rt:Reg8::IYL,rs:Reg8::IYL}, &LdRMemIyD{r:Reg8::L}         , &LdRR{rt:Reg8::IYL,rs:Reg8::A},

    /* 0x70 */             /* 0x71 */             /* 0x72 */             /* 0x73 */             /* 0x74 */             /* 0x75 */             /* 0x76 */    /* 0x77 */
    &LdMemIyDR{r:Reg8::B}, &LdMemIyDR{r:Reg8::C}, &LdMemIyDR{r:Reg8::D}, &LdMemIyDR{r:Reg8::E}, &LdMemIyDR{r:Reg8::H}, &LdMemIyDR{r:Reg8::L}, &Unsupported, &LdMemIyDR{r:Reg8::A},

    /* 0x78 */          /* 0x79 */        /* 0x7A */         /* 0x7B */         /* 0x7C */                      /* 0x7D */                      /* 0x7E */             /* 0x7F */
    INSTR_TABLE[0x78], INSTR_TABLE[0x79], INSTR_TABLE[0x7A], INSTR_TABLE[0x7B], &LdRR{rt:Reg8::A,rs:Reg8::IYH}, &LdRR{rt:Reg8::A,rs:Reg8::IYL}, &LdRMemIyD{r:Reg8::A}, INSTR_TABLE[0x7F],

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */          /* 0x85 */          /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AddR{r:Reg8::IYH}, &AddR{r:Reg8::IYL}, &AddMemIyD  , &Unsupported,

    /* 0x88 */    /* 0x89 */    /* 0x8A */    /* 0x8B */    /* 0x8C */          /* 0x8D */          /* 0x8E */    /* 0x8F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AdcR{r:Reg8::IYH}, &AdcR{r:Reg8::IYL}, &AdcMemIyD  , &Unsupported,

    /* 0x90 */    /* 0x91 */    /* 0x92 */    /* 0x93 */    /* 0x94 */          /* 0x95 */          /* 0x96 */    /* 0x97 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SubR{r:Reg8::IYH}, &SubR{r:Reg8::IYL}, &SubMemIyD  , &Unsupported,

    /* 0x98 */    /* 0x99 */    /* 0x9A */    /* 0x9B */    /* 0x9C */          /* 0x9D */          /* 0x9E */    /* 0x9F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &SbcR{r:Reg8::IYH}, &SbcR{r:Reg8::IYL}, &SbcMemIyD  , &Unsupported,

    /* 0xA0 */    /* 0xA1 */    /* 0xA2 */    /* 0xA3 */    /* 0xA4 */          /* 0xA5 */          /* 0xA6 */    /* 0xA7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AndR{r:Reg8::IYH}, &AndR{r:Reg8::IYL}, &AndMemIyD  , &Unsupported,

    /* 0xA8 */    /* 0xA9 */    /* 0xAA */    /* 0xAB */    /* 0xAC */          /* 0xAD */          /* 0xAE */    /* 0xAF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &XorR{r:Reg8::IYH}, &XorR{r:Reg8::IYL}, &XorMemIyD  , &Unsupported,

    /* 0xB0 */    /* 0xB1 */    /* 0xB2 */    /* 0xB3 */    /* 0xB4 */          /* 0xB5 */          /* 0xB6 */    /* 0xB7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &OrR{r:Reg8::IYH} , &OrR{r:Reg8::IYL} , &OrMemIyD   , &Unsupported,

    /* 0xB8 */    /* 0xB9 */    /* 0xBA */    /* 0xBB */    /* 0xBC */          /* 0xBD */          /* 0xBE */    /* 0xBF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &CpR{r:Reg8::IYH} , &CpR{r:Reg8::IYL} , &CpMemIyD   , &Unsupported,

    /* 0xC0 */    /* 0xC1 */    /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */    /* 0xC6 */    /* 0xC7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xC8 */    /* 0xC9 */    /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */    /* 0xCE */    /* 0xCF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD0 */    /* 0xD1 */    /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */    /* 0xD6 */    /* 0xD7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xD8 */    /* 0xD9 */    /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */    /* 0xDE */    /* 0xDF */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xE0 */    /* 0xE1 */           /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */            /* 0xE6 */    /* 0xE7 */
    &Unsupported, &PopQq{r:Reg16::IY}, &Unsupported, &ExMemSpIy  , &Unsupported, &PushQq{r:Reg16::IY}, &Unsupported, &Unsupported,

    /* 0xE8 */    /* 0xE9 */    /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */    /* 0xEE */    /* 0xEF */
    &Unsupported, &JpIy       , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF0 */    /* 0xF1 */    /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */    /* 0xF6 */    /* 0xF7 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0xF8 */    /* 0xF9 */    /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */    /* 0xFE */    /* 0xFF */
    &Unsupported, &LdSpIy     , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported
];

pub const INSTR_TABLE_DDCB: [&'static Instruction; 256] = [
    /* 0x00 */              /* 0x01 */              /* 0x02 */              /* 0x03 */              /* 0x04 */              /* 0x05 */              /* 0x06 */    /* 0x07 */
    &RlcMemIxDR{r:Reg8::B}, &RlcMemIxDR{r:Reg8::C}, &RlcMemIxDR{r:Reg8::D}, &RlcMemIxDR{r:Reg8::E}, &RlcMemIxDR{r:Reg8::H}, &RlcMemIxDR{r:Reg8::L}, &RlcMemIxD  , &RlcMemIxDR{r:Reg8::A},

    /* 0x08 */              /* 0x09 */              /* 0x0A */              /* 0x0B */              /* 0x0C */              /* 0x0D */              /* 0x0E */    /* 0x0F */
    &RrcMemIxDR{r:Reg8::B}, &RrcMemIxDR{r:Reg8::C}, &RrcMemIxDR{r:Reg8::D}, &RrcMemIxDR{r:Reg8::E}, &RrcMemIxDR{r:Reg8::H}, &RrcMemIxDR{r:Reg8::L}, &RrcMemIxD  , &RrcMemIxDR{r:Reg8::A},

    /* 0x10 */              /* 0x11 */              /* 0x12 */              /* 0x13 */              /* 0x14 */              /* 0x15 */              /* 0x16 */    /* 0x17 */
    &RlMemIxDR{r:Reg8::B} , &RlMemIxDR{r:Reg8::C} , &RlMemIxDR{r:Reg8::D} , &RlMemIxDR{r:Reg8::E} , &RlMemIxDR{r:Reg8::H} , &RlMemIxDR{r:Reg8::L} , &RlMemIxD   , &RlMemIxDR{r:Reg8::A} ,

    /* 0x18 */              /* 0x19 */              /* 0x1A */              /* 0x1B */              /* 0x1C */              /* 0x1D */              /* 0x1E */    /* 0x1F */
    &RrMemIxDR{r:Reg8::B} , &RrMemIxDR{r:Reg8::C} , &RrMemIxDR{r:Reg8::D} , &RrMemIxDR{r:Reg8::E} , &RrMemIxDR{r:Reg8::H} , &RrMemIxDR{r:Reg8::L} , &RrMemIxD   , &RrMemIxDR{r:Reg8::A} ,

    /* 0x20 */              /* 0x21 */              /* 0x22 */              /* 0x23 */              /* 0x24 */              /* 0x25 */              /* 0x26 */    /* 0x27 */
    &SlaMemIxDR{r:Reg8::B}, &SlaMemIxDR{r:Reg8::C}, &SlaMemIxDR{r:Reg8::D}, &SlaMemIxDR{r:Reg8::E}, &SlaMemIxDR{r:Reg8::H}, &SlaMemIxDR{r:Reg8::L}, &SlaMemIxD  , &SlaMemIxDR{r:Reg8::A},

    /* 0x28 */              /* 0x29 */              /* 0x2A */              /* 0x2B */              /* 0x2C */              /* 0x2D */              /* 0x2E */    /* 0x2F */
    &SraMemIxDR{r:Reg8::B}, &SraMemIxDR{r:Reg8::C}, &SraMemIxDR{r:Reg8::D}, &SraMemIxDR{r:Reg8::E}, &SraMemIxDR{r:Reg8::H}, &SraMemIxDR{r:Reg8::L}, &SraMemIxD  , &SraMemIxDR{r:Reg8::A},

    /* 0x30 */              /* 0x31 */              /* 0x32 */              /* 0x33 */              /* 0x34 */              /* 0x35 */              /* 0x36 */    /* 0x37 */
    &SllMemIxDR{r:Reg8::B}, &SllMemIxDR{r:Reg8::C}, &SllMemIxDR{r:Reg8::D}, &SllMemIxDR{r:Reg8::E}, &SllMemIxDR{r:Reg8::H}, &SllMemIxDR{r:Reg8::L}, &SllMemIxD  , &SllMemIxDR{r:Reg8::A},

    /* 0x38 */              /* 0x39 */              /* 0x3A */              /* 0x3B */              /* 0x3C */              /* 0x3D */              /* 0x3E */    /* 0x3F */
    &SrlMemIxDR{r:Reg8::B}, &SrlMemIxDR{r:Reg8::C}, &SrlMemIxDR{r:Reg8::D}, &SrlMemIxDR{r:Reg8::E}, &SrlMemIxDR{r:Reg8::H}, &SrlMemIxDR{r:Reg8::L}, &SrlMemIxD  , &SrlMemIxDR{r:Reg8::A},

    /* 0x40 */        /* 0x41 */        /* 0x42 */        /* 0x43 */        /* 0x44 */        /* 0x45 */        /* 0x46 */        /* 0x47 */
    &BitBMemIxD{b:0}, &BitBMemIxD{b:0}, &BitBMemIxD{b:0}, &BitBMemIxD{b:0}, &BitBMemIxD{b:0}, &BitBMemIxD{b:0}, &BitBMemIxD{b:0}, &BitBMemIxD{b:0},

    /* 0x48 */        /* 0x49 */        /* 0x4A */        /* 0x4B */        /* 0x4C */        /* 0x4D */        /* 0x4E */        /* 0x4F */
    &BitBMemIxD{b:1}, &BitBMemIxD{b:1}, &BitBMemIxD{b:1}, &BitBMemIxD{b:1}, &BitBMemIxD{b:1}, &BitBMemIxD{b:1}, &BitBMemIxD{b:1}, &BitBMemIxD{b:1},

    /* 0x50 */        /* 0x51 */        /* 0x52 */        /* 0x53 */        /* 0x54 */        /* 0x55 */        /* 0x56 */        /* 0x57 */
    &BitBMemIxD{b:2}, &BitBMemIxD{b:2}, &BitBMemIxD{b:2}, &BitBMemIxD{b:2}, &BitBMemIxD{b:2}, &BitBMemIxD{b:2}, &BitBMemIxD{b:2}, &BitBMemIxD{b:2},

    /* 0x58 */        /* 0x59 */        /* 0x5A */        /* 0x5B */        /* 0x5C */        /* 0x5D */        /* 0x5E */        /* 0x5F */
    &BitBMemIxD{b:3}, &BitBMemIxD{b:3}, &BitBMemIxD{b:3}, &BitBMemIxD{b:3}, &BitBMemIxD{b:3}, &BitBMemIxD{b:3}, &BitBMemIxD{b:3}, &BitBMemIxD{b:3},

    /* 0x60 */        /* 0x61 */        /* 0x62 */        /* 0x63 */        /* 0x64 */        /* 0x65 */        /* 0x66 */        /* 0x67 */
    &BitBMemIxD{b:4}, &BitBMemIxD{b:4}, &BitBMemIxD{b:4}, &BitBMemIxD{b:4}, &BitBMemIxD{b:4}, &BitBMemIxD{b:4}, &BitBMemIxD{b:4}, &BitBMemIxD{b:4},

    /* 0x68 */        /* 0x69 */        /* 0x6A */        /* 0x6B */        /* 0x6C */        /* 0x6D */        /* 0x6E */        /* 0x6F */
    &BitBMemIxD{b:5}, &BitBMemIxD{b:5}, &BitBMemIxD{b:5}, &BitBMemIxD{b:5}, &BitBMemIxD{b:5}, &BitBMemIxD{b:5}, &BitBMemIxD{b:5}, &BitBMemIxD{b:5},

    /* 0x70 */        /* 0x71 */        /* 0x72 */        /* 0x73 */        /* 0x74 */        /* 0x75 */        /* 0x76 */        /* 0x77 */
    &BitBMemIxD{b:6}, &BitBMemIxD{b:6}, &BitBMemIxD{b:6}, &BitBMemIxD{b:6}, &BitBMemIxD{b:6}, &BitBMemIxD{b:6}, &BitBMemIxD{b:6}, &BitBMemIxD{b:6},

    /* 0x78 */        /* 0x79 */        /* 0x7A */        /* 0x7B */        /* 0x7C */        /* 0x7D */        /* 0x7E */        /* 0x7F */
    &BitBMemIxD{b:7}, &BitBMemIxD{b:7}, &BitBMemIxD{b:7}, &BitBMemIxD{b:7}, &BitBMemIxD{b:7}, &BitBMemIxD{b:7}, &BitBMemIxD{b:7}, &BitBMemIxD{b:7},

    /* 0x80 */                   /* 0x81 */                   /* 0x82 */                   /* 0x83 */                   /* 0x84 */                   /* 0x85 */                   /* 0x86 */        /* 0x87 */
    &ResBMemIxDR{b:0,r:Reg8::B}, &ResBMemIxDR{b:0,r:Reg8::C}, &ResBMemIxDR{b:0,r:Reg8::D}, &ResBMemIxDR{b:0,r:Reg8::E}, &ResBMemIxDR{b:0,r:Reg8::H}, &ResBMemIxDR{b:0,r:Reg8::L}, &ResBMemIxD{b:0}, &ResBMemIxDR{b:0,r:Reg8::A},

    /* 0x88 */                   /* 0x89 */                   /* 0x8A */                   /* 0x8B */                   /* 0x8C */                   /* 0x8D */                   /* 0x8E */        /* 0x8F */
    &ResBMemIxDR{b:1,r:Reg8::B}, &ResBMemIxDR{b:1,r:Reg8::C}, &ResBMemIxDR{b:1,r:Reg8::D}, &ResBMemIxDR{b:1,r:Reg8::E}, &ResBMemIxDR{b:1,r:Reg8::H}, &ResBMemIxDR{b:1,r:Reg8::L}, &ResBMemIxD{b:1}, &ResBMemIxDR{b:1,r:Reg8::A},

    /* 0x90 */                   /* 0x91 */                   /* 0x92 */                   /* 0x93 */                   /* 0x94 */                   /* 0x95 */                   /* 0x96 */        /* 0x97 */
    &ResBMemIxDR{b:2,r:Reg8::B}, &ResBMemIxDR{b:2,r:Reg8::C}, &ResBMemIxDR{b:2,r:Reg8::D}, &ResBMemIxDR{b:2,r:Reg8::E}, &ResBMemIxDR{b:2,r:Reg8::H}, &ResBMemIxDR{b:2,r:Reg8::L}, &ResBMemIxD{b:2}, &ResBMemIxDR{b:2,r:Reg8::A},

    /* 0x98 */                   /* 0x99 */                   /* 0x9A */                   /* 0x9B */                   /* 0x9C */                   /* 0x9D */                   /* 0x9E */        /* 0x9F */
    &ResBMemIxDR{b:3,r:Reg8::B}, &ResBMemIxDR{b:3,r:Reg8::C}, &ResBMemIxDR{b:3,r:Reg8::D}, &ResBMemIxDR{b:3,r:Reg8::E}, &ResBMemIxDR{b:3,r:Reg8::H}, &ResBMemIxDR{b:3,r:Reg8::L}, &ResBMemIxD{b:3}, &ResBMemIxDR{b:3,r:Reg8::A},

    /* 0xA0 */                   /* 0xA1 */                   /* 0xA2 */                   /* 0xA3 */                   /* 0xA4 */                   /* 0xA5 */                   /* 0xA6 */        /* 0xA7 */
    &ResBMemIxDR{b:4,r:Reg8::B}, &ResBMemIxDR{b:4,r:Reg8::C}, &ResBMemIxDR{b:4,r:Reg8::D}, &ResBMemIxDR{b:4,r:Reg8::E}, &ResBMemIxDR{b:4,r:Reg8::H}, &ResBMemIxDR{b:4,r:Reg8::L}, &ResBMemIxD{b:4}, &ResBMemIxDR{b:4,r:Reg8::A},

    /* 0xA8 */                   /* 0xA9 */                   /* 0xAA */                   /* 0xAB */                   /* 0xAC */                   /* 0xAD */                   /* 0xAE */        /* 0xAF */
    &ResBMemIxDR{b:5,r:Reg8::B}, &ResBMemIxDR{b:5,r:Reg8::C}, &ResBMemIxDR{b:5,r:Reg8::D}, &ResBMemIxDR{b:5,r:Reg8::E}, &ResBMemIxDR{b:5,r:Reg8::H}, &ResBMemIxDR{b:5,r:Reg8::L}, &ResBMemIxD{b:5}, &ResBMemIxDR{b:5,r:Reg8::A},

    /* 0xB0 */                   /* 0xB1 */                   /* 0xB2 */                   /* 0xB3 */                   /* 0xB4 */                   /* 0xB5 */                   /* 0xB6 */        /* 0xB7 */
    &ResBMemIxDR{b:6,r:Reg8::B}, &ResBMemIxDR{b:6,r:Reg8::C}, &ResBMemIxDR{b:6,r:Reg8::D}, &ResBMemIxDR{b:6,r:Reg8::E}, &ResBMemIxDR{b:6,r:Reg8::H}, &ResBMemIxDR{b:6,r:Reg8::L}, &ResBMemIxD{b:6}, &ResBMemIxDR{b:6,r:Reg8::A},

    /* 0xB8 */                   /* 0xB9 */                   /* 0xBA */                   /* 0xBB */                   /* 0xBC */                   /* 0xBD */                   /* 0xBE */        /* 0xBF */
    &ResBMemIxDR{b:7,r:Reg8::B}, &ResBMemIxDR{b:7,r:Reg8::C}, &ResBMemIxDR{b:7,r:Reg8::D}, &ResBMemIxDR{b:7,r:Reg8::E}, &ResBMemIxDR{b:7,r:Reg8::H}, &ResBMemIxDR{b:7,r:Reg8::L}, &ResBMemIxD{b:7}, &ResBMemIxDR{b:7,r:Reg8::A},

    /* 0xC0 */                   /* 0xC1 */                   /* 0xC2 */                   /* 0xC3 */                   /* 0xC4 */                   /* 0xC5 */                   /* 0xC6 */        /* 0xC7 */
    &SetBMemIxDR{b:0,r:Reg8::B}, &SetBMemIxDR{b:0,r:Reg8::C}, &SetBMemIxDR{b:0,r:Reg8::D}, &SetBMemIxDR{b:0,r:Reg8::E}, &SetBMemIxDR{b:0,r:Reg8::H}, &SetBMemIxDR{b:0,r:Reg8::L}, &SetBMemIxD{b:0}, &SetBMemIxDR{b:0,r:Reg8::A},

    /* 0xC8 */                   /* 0xC9 */                   /* 0xCA */                   /* 0xCB */                   /* 0xCC */                   /* 0xCD */                   /* 0xCE */        /* 0xCF */
    &SetBMemIxDR{b:1,r:Reg8::B}, &SetBMemIxDR{b:1,r:Reg8::C}, &SetBMemIxDR{b:1,r:Reg8::D}, &SetBMemIxDR{b:1,r:Reg8::E}, &SetBMemIxDR{b:1,r:Reg8::H}, &SetBMemIxDR{b:1,r:Reg8::L}, &SetBMemIxD{b:1}, &SetBMemIxDR{b:1,r:Reg8::A},

    /* 0xD0 */                   /* 0xD1 */                   /* 0xD2 */                   /* 0xD3 */                   /* 0xD4 */                   /* 0xD5 */                   /* 0xD6 */        /* 0xD7 */
    &SetBMemIxDR{b:2,r:Reg8::B}, &SetBMemIxDR{b:2,r:Reg8::C}, &SetBMemIxDR{b:2,r:Reg8::D}, &SetBMemIxDR{b:2,r:Reg8::E}, &SetBMemIxDR{b:2,r:Reg8::H}, &SetBMemIxDR{b:2,r:Reg8::L}, &SetBMemIxD{b:2}, &SetBMemIxDR{b:2,r:Reg8::A},

    /* 0xD8 */                   /* 0xD9 */                   /* 0xDA */                   /* 0xDB */                   /* 0xDC */                   /* 0xDD */                   /* 0xDE */        /* 0xDF */
    &SetBMemIxDR{b:3,r:Reg8::B}, &SetBMemIxDR{b:3,r:Reg8::C}, &SetBMemIxDR{b:3,r:Reg8::D}, &SetBMemIxDR{b:3,r:Reg8::E}, &SetBMemIxDR{b:3,r:Reg8::H}, &SetBMemIxDR{b:3,r:Reg8::L}, &SetBMemIxD{b:3}, &SetBMemIxDR{b:3,r:Reg8::A},

    /* 0xE0 */                   /* 0xE1 */                   /* 0xE2 */                   /* 0xE3 */                   /* 0xE4 */                   /* 0xE5 */                   /* 0xE6 */        /* 0xE7 */
    &SetBMemIxDR{b:4,r:Reg8::B}, &SetBMemIxDR{b:4,r:Reg8::C}, &SetBMemIxDR{b:4,r:Reg8::D}, &SetBMemIxDR{b:4,r:Reg8::E}, &SetBMemIxDR{b:4,r:Reg8::H}, &SetBMemIxDR{b:4,r:Reg8::L}, &SetBMemIxD{b:4}, &SetBMemIxDR{b:4,r:Reg8::A},

    /* 0xE8 */                   /* 0xE9 */                   /* 0xEA */                   /* 0xEB */                   /* 0xEC */                   /* 0xED */                   /* 0xEE */        /* 0xEF */
    &SetBMemIxDR{b:5,r:Reg8::B}, &SetBMemIxDR{b:5,r:Reg8::C}, &SetBMemIxDR{b:5,r:Reg8::D}, &SetBMemIxDR{b:5,r:Reg8::E}, &SetBMemIxDR{b:5,r:Reg8::H}, &SetBMemIxDR{b:5,r:Reg8::L}, &SetBMemIxD{b:5}, &SetBMemIxDR{b:5,r:Reg8::A},

    /* 0xF0 */                   /* 0xF1 */                   /* 0xF2 */                   /* 0xF3 */                   /* 0xF4 */                   /* 0xF5 */                   /* 0xF6 */        /* 0xF7 */
    &SetBMemIxDR{b:6,r:Reg8::B}, &SetBMemIxDR{b:6,r:Reg8::C}, &SetBMemIxDR{b:6,r:Reg8::D}, &SetBMemIxDR{b:6,r:Reg8::E}, &SetBMemIxDR{b:6,r:Reg8::H}, &SetBMemIxDR{b:6,r:Reg8::L}, &SetBMemIxD{b:6}, &SetBMemIxDR{b:6,r:Reg8::A},

    /* 0xF8 */                   /* 0xF9 */                   /* 0xFA */                   /* 0xFB */                   /* 0xFC */                   /* 0xFD */                   /* 0xFE */        /* 0xFF */
    &SetBMemIxDR{b:7,r:Reg8::B}, &SetBMemIxDR{b:7,r:Reg8::C}, &SetBMemIxDR{b:7,r:Reg8::D}, &SetBMemIxDR{b:7,r:Reg8::E}, &SetBMemIxDR{b:7,r:Reg8::H}, &SetBMemIxDR{b:7,r:Reg8::L}, &SetBMemIxD{b:7}, &SetBMemIxDR{b:7,r:Reg8::A},
];

pub const INSTR_TABLE_FDCB: [&'static Instruction; 256] = [
    /* 0y00 */              /* 0y01 */              /* 0y02 */              /* 0y03 */              /* 0y04 */              /* 0y05 */              /* 0y06 */    /* 0y07 */
    &RlcMemIyDR{r:Reg8::B}, &RlcMemIyDR{r:Reg8::C}, &RlcMemIyDR{r:Reg8::D}, &RlcMemIyDR{r:Reg8::E}, &RlcMemIyDR{r:Reg8::H}, &RlcMemIyDR{r:Reg8::L}, &RlcMemIyD  , &RlcMemIyDR{r:Reg8::A},

    /* 0y08 */              /* 0y09 */              /* 0y0A */              /* 0y0B */              /* 0y0C */              /* 0y0D */              /* 0y0E */    /* 0y0F */
    &RrcMemIyDR{r:Reg8::B}, &RrcMemIyDR{r:Reg8::C}, &RrcMemIyDR{r:Reg8::D}, &RrcMemIyDR{r:Reg8::E}, &RrcMemIyDR{r:Reg8::H}, &RrcMemIyDR{r:Reg8::L}, &RrcMemIyD  , &RrcMemIyDR{r:Reg8::A},

    /* 0y10 */              /* 0y11 */              /* 0y12 */              /* 0y13 */              /* 0y14 */              /* 0y15 */              /* 0y16 */    /* 0y17 */
    &RlMemIyDR{r:Reg8::B} , &RlMemIyDR{r:Reg8::C} , &RlMemIyDR{r:Reg8::D} , &RlMemIyDR{r:Reg8::E} , &RlMemIyDR{r:Reg8::H} , &RlMemIyDR{r:Reg8::L} , &RlMemIyD   , &RlMemIyDR{r:Reg8::A} ,

    /* 0y18 */              /* 0y19 */              /* 0y1A */              /* 0y1B */              /* 0y1C */              /* 0y1D */              /* 0y1E */    /* 0y1F */
    &RrMemIyDR{r:Reg8::B} , &RrMemIyDR{r:Reg8::C} , &RrMemIyDR{r:Reg8::D} , &RrMemIyDR{r:Reg8::E} , &RrMemIyDR{r:Reg8::H} , &RrMemIyDR{r:Reg8::L} , &RrMemIyD   , &RrMemIyDR{r:Reg8::A} ,

    /* 0y20 */              /* 0y21 */              /* 0y22 */              /* 0y23 */              /* 0y24 */              /* 0y25 */              /* 0y26 */    /* 0y27 */
    &SlaMemIyDR{r:Reg8::B}, &SlaMemIyDR{r:Reg8::C}, &SlaMemIyDR{r:Reg8::D}, &SlaMemIyDR{r:Reg8::E}, &SlaMemIyDR{r:Reg8::H}, &SlaMemIyDR{r:Reg8::L}, &SlaMemIyD  , &SlaMemIyDR{r:Reg8::A},

    /* 0y28 */              /* 0y29 */              /* 0y2A */              /* 0y2B */              /* 0y2C */              /* 0y2D */              /* 0y2E */    /* 0y2F */
    &SraMemIyDR{r:Reg8::B}, &SraMemIyDR{r:Reg8::C}, &SraMemIyDR{r:Reg8::D}, &SraMemIyDR{r:Reg8::E}, &SraMemIyDR{r:Reg8::H}, &SraMemIyDR{r:Reg8::L}, &SraMemIyD  , &SraMemIyDR{r:Reg8::A},

    /* 0y30 */              /* 0y31 */              /* 0y32 */              /* 0y33 */              /* 0y34 */              /* 0y35 */              /* 0y36 */    /* 0y37 */
    &SllMemIyDR{r:Reg8::B}, &SllMemIyDR{r:Reg8::C}, &SllMemIyDR{r:Reg8::D}, &SllMemIyDR{r:Reg8::E}, &SllMemIyDR{r:Reg8::H}, &SllMemIyDR{r:Reg8::L}, &SllMemIyD  , &SllMemIyDR{r:Reg8::A},

    /* 0y38 */              /* 0y39 */              /* 0y3A */              /* 0y3B */              /* 0y3C */              /* 0y3D */              /* 0y3E */    /* 0y3F */
    &SrlMemIyDR{r:Reg8::B}, &SrlMemIyDR{r:Reg8::C}, &SrlMemIyDR{r:Reg8::D}, &SrlMemIyDR{r:Reg8::E}, &SrlMemIyDR{r:Reg8::H}, &SrlMemIyDR{r:Reg8::L}, &SrlMemIyD  , &SrlMemIyDR{r:Reg8::A},

    /* 0y40 */        /* 0y41 */        /* 0y42 */        /* 0y43 */        /* 0y44 */        /* 0y45 */        /* 0y46 */        /* 0y47 */
    &BitBMemIyD{b:0}, &BitBMemIyD{b:0}, &BitBMemIyD{b:0}, &BitBMemIyD{b:0}, &BitBMemIyD{b:0}, &BitBMemIyD{b:0}, &BitBMemIyD{b:0}, &BitBMemIyD{b:0},

    /* 0y48 */        /* 0y49 */        /* 0y4A */        /* 0y4B */        /* 0y4C */        /* 0y4D */        /* 0y4E */        /* 0y4F */
    &BitBMemIyD{b:1}, &BitBMemIyD{b:1}, &BitBMemIyD{b:1}, &BitBMemIyD{b:1}, &BitBMemIyD{b:1}, &BitBMemIyD{b:1}, &BitBMemIyD{b:1}, &BitBMemIyD{b:1},

    /* 0y50 */        /* 0y51 */        /* 0y52 */        /* 0y53 */        /* 0y54 */        /* 0y55 */        /* 0y56 */        /* 0y57 */
    &BitBMemIyD{b:2}, &BitBMemIyD{b:2}, &BitBMemIyD{b:2}, &BitBMemIyD{b:2}, &BitBMemIyD{b:2}, &BitBMemIyD{b:2}, &BitBMemIyD{b:2}, &BitBMemIyD{b:2},

    /* 0y58 */        /* 0y59 */        /* 0y5A */        /* 0y5B */        /* 0y5C */        /* 0y5D */        /* 0y5E */        /* 0y5F */
    &BitBMemIyD{b:3}, &BitBMemIyD{b:3}, &BitBMemIyD{b:3}, &BitBMemIyD{b:3}, &BitBMemIyD{b:3}, &BitBMemIyD{b:3}, &BitBMemIyD{b:3}, &BitBMemIyD{b:3},

    /* 0y60 */        /* 0y61 */        /* 0y62 */        /* 0y63 */        /* 0y64 */        /* 0y65 */        /* 0y66 */        /* 0y67 */
    &BitBMemIyD{b:4}, &BitBMemIyD{b:4}, &BitBMemIyD{b:4}, &BitBMemIyD{b:4}, &BitBMemIyD{b:4}, &BitBMemIyD{b:4}, &BitBMemIyD{b:4}, &BitBMemIyD{b:4},

    /* 0y68 */        /* 0y69 */        /* 0y6A */        /* 0y6B */        /* 0y6C */        /* 0y6D */        /* 0y6E */        /* 0y6F */
    &BitBMemIyD{b:5}, &BitBMemIyD{b:5}, &BitBMemIyD{b:5}, &BitBMemIyD{b:5}, &BitBMemIyD{b:5}, &BitBMemIyD{b:5}, &BitBMemIyD{b:5}, &BitBMemIyD{b:5},

    /* 0y70 */        /* 0y71 */        /* 0y72 */        /* 0y73 */        /* 0y74 */        /* 0y75 */        /* 0y76 */        /* 0y77 */
    &BitBMemIyD{b:6}, &BitBMemIyD{b:6}, &BitBMemIyD{b:6}, &BitBMemIyD{b:6}, &BitBMemIyD{b:6}, &BitBMemIyD{b:6}, &BitBMemIyD{b:6}, &BitBMemIyD{b:6},

    /* 0y78 */        /* 0y79 */        /* 0y7A */        /* 0y7B */        /* 0y7C */        /* 0y7D */        /* 0y7E */        /* 0y7F */
    &BitBMemIyD{b:7}, &BitBMemIyD{b:7}, &BitBMemIyD{b:7}, &BitBMemIyD{b:7}, &BitBMemIyD{b:7}, &BitBMemIyD{b:7}, &BitBMemIyD{b:7}, &BitBMemIyD{b:7},

    /* 0y80 */                   /* 0y81 */                   /* 0y82 */                   /* 0y83 */                   /* 0y84 */                   /* 0y85 */                   /* 0y86 */        /* 0y87 */
    &ResBMemIyDR{b:0,r:Reg8::B}, &ResBMemIyDR{b:0,r:Reg8::C}, &ResBMemIyDR{b:0,r:Reg8::D}, &ResBMemIyDR{b:0,r:Reg8::E}, &ResBMemIyDR{b:0,r:Reg8::H}, &ResBMemIyDR{b:0,r:Reg8::L}, &ResBMemIyD{b:0}, &ResBMemIyDR{b:0,r:Reg8::A},

    /* 0y88 */                   /* 0y89 */                   /* 0y8A */                   /* 0y8B */                   /* 0y8C */                   /* 0y8D */                   /* 0y8E */        /* 0y8F */
    &ResBMemIyDR{b:1,r:Reg8::B}, &ResBMemIyDR{b:1,r:Reg8::C}, &ResBMemIyDR{b:1,r:Reg8::D}, &ResBMemIyDR{b:1,r:Reg8::E}, &ResBMemIyDR{b:1,r:Reg8::H}, &ResBMemIyDR{b:1,r:Reg8::L}, &ResBMemIyD{b:1}, &ResBMemIyDR{b:1,r:Reg8::A},

    /* 0y90 */                   /* 0y91 */                   /* 0y92 */                   /* 0y93 */                   /* 0y94 */                   /* 0y95 */                   /* 0y96 */        /* 0y97 */
    &ResBMemIyDR{b:2,r:Reg8::B}, &ResBMemIyDR{b:2,r:Reg8::C}, &ResBMemIyDR{b:2,r:Reg8::D}, &ResBMemIyDR{b:2,r:Reg8::E}, &ResBMemIyDR{b:2,r:Reg8::H}, &ResBMemIyDR{b:2,r:Reg8::L}, &ResBMemIyD{b:2}, &ResBMemIyDR{b:2,r:Reg8::A},

    /* 0y98 */                   /* 0y99 */                   /* 0y9A */                   /* 0y9B */                   /* 0y9C */                   /* 0y9D */                   /* 0y9E */        /* 0y9F */
    &ResBMemIyDR{b:3,r:Reg8::B}, &ResBMemIyDR{b:3,r:Reg8::C}, &ResBMemIyDR{b:3,r:Reg8::D}, &ResBMemIyDR{b:3,r:Reg8::E}, &ResBMemIyDR{b:3,r:Reg8::H}, &ResBMemIyDR{b:3,r:Reg8::L}, &ResBMemIyD{b:3}, &ResBMemIyDR{b:3,r:Reg8::A},

    /* 0yA0 */                   /* 0yA1 */                   /* 0yA2 */                   /* 0yA3 */                   /* 0yA4 */                   /* 0yA5 */                   /* 0yA6 */        /* 0yA7 */
    &ResBMemIyDR{b:4,r:Reg8::B}, &ResBMemIyDR{b:4,r:Reg8::C}, &ResBMemIyDR{b:4,r:Reg8::D}, &ResBMemIyDR{b:4,r:Reg8::E}, &ResBMemIyDR{b:4,r:Reg8::H}, &ResBMemIyDR{b:4,r:Reg8::L}, &ResBMemIyD{b:4}, &ResBMemIyDR{b:4,r:Reg8::A},

    /* 0yA8 */                   /* 0yA9 */                   /* 0yAA */                   /* 0yAB */                   /* 0yAC */                   /* 0yAD */                   /* 0yAE */        /* 0yAF */
    &ResBMemIyDR{b:5,r:Reg8::B}, &ResBMemIyDR{b:5,r:Reg8::C}, &ResBMemIyDR{b:5,r:Reg8::D}, &ResBMemIyDR{b:5,r:Reg8::E}, &ResBMemIyDR{b:5,r:Reg8::H}, &ResBMemIyDR{b:5,r:Reg8::L}, &ResBMemIyD{b:5}, &ResBMemIyDR{b:5,r:Reg8::A},

    /* 0yB0 */                   /* 0yB1 */                   /* 0yB2 */                   /* 0yB3 */                   /* 0yB4 */                   /* 0yB5 */                   /* 0yB6 */        /* 0yB7 */
    &ResBMemIyDR{b:6,r:Reg8::B}, &ResBMemIyDR{b:6,r:Reg8::C}, &ResBMemIyDR{b:6,r:Reg8::D}, &ResBMemIyDR{b:6,r:Reg8::E}, &ResBMemIyDR{b:6,r:Reg8::H}, &ResBMemIyDR{b:6,r:Reg8::L}, &ResBMemIyD{b:6}, &ResBMemIyDR{b:6,r:Reg8::A},

    /* 0yB8 */                   /* 0yB9 */                   /* 0yBA */                   /* 0yBB */                   /* 0yBC */                   /* 0yBD */                   /* 0yBE */        /* 0yBF */
    &ResBMemIyDR{b:7,r:Reg8::B}, &ResBMemIyDR{b:7,r:Reg8::C}, &ResBMemIyDR{b:7,r:Reg8::D}, &ResBMemIyDR{b:7,r:Reg8::E}, &ResBMemIyDR{b:7,r:Reg8::H}, &ResBMemIyDR{b:7,r:Reg8::L}, &ResBMemIyD{b:7}, &ResBMemIyDR{b:7,r:Reg8::A},

    /* 0yC0 */                   /* 0yC1 */                   /* 0yC2 */                   /* 0yC3 */                   /* 0yC4 */                   /* 0yC5 */                   /* 0yC6 */        /* 0yC7 */
    &SetBMemIyDR{b:0,r:Reg8::B}, &SetBMemIyDR{b:0,r:Reg8::C}, &SetBMemIyDR{b:0,r:Reg8::D}, &SetBMemIyDR{b:0,r:Reg8::E}, &SetBMemIyDR{b:0,r:Reg8::H}, &SetBMemIyDR{b:0,r:Reg8::L}, &SetBMemIyD{b:0}, &SetBMemIyDR{b:0,r:Reg8::A},

    /* 0yC8 */                   /* 0yC9 */                   /* 0yCA */                   /* 0yCB */                   /* 0yCC */                   /* 0yCD */                   /* 0yCE */        /* 0yCF */
    &SetBMemIyDR{b:1,r:Reg8::B}, &SetBMemIyDR{b:1,r:Reg8::C}, &SetBMemIyDR{b:1,r:Reg8::D}, &SetBMemIyDR{b:1,r:Reg8::E}, &SetBMemIyDR{b:1,r:Reg8::H}, &SetBMemIyDR{b:1,r:Reg8::L}, &SetBMemIyD{b:1}, &SetBMemIyDR{b:1,r:Reg8::A},

    /* 0yD0 */                   /* 0yD1 */                   /* 0yD2 */                   /* 0yD3 */                   /* 0yD4 */                   /* 0yD5 */                   /* 0yD6 */        /* 0yD7 */
    &SetBMemIyDR{b:2,r:Reg8::B}, &SetBMemIyDR{b:2,r:Reg8::C}, &SetBMemIyDR{b:2,r:Reg8::D}, &SetBMemIyDR{b:2,r:Reg8::E}, &SetBMemIyDR{b:2,r:Reg8::H}, &SetBMemIyDR{b:2,r:Reg8::L}, &SetBMemIyD{b:2}, &SetBMemIyDR{b:2,r:Reg8::A},

    /* 0yD8 */                   /* 0yD9 */                   /* 0yDA */                   /* 0yDB */                   /* 0yDC */                   /* 0yDD */                   /* 0yDE */        /* 0yDF */
    &SetBMemIyDR{b:3,r:Reg8::B}, &SetBMemIyDR{b:3,r:Reg8::C}, &SetBMemIyDR{b:3,r:Reg8::D}, &SetBMemIyDR{b:3,r:Reg8::E}, &SetBMemIyDR{b:3,r:Reg8::H}, &SetBMemIyDR{b:3,r:Reg8::L}, &SetBMemIyD{b:3}, &SetBMemIyDR{b:3,r:Reg8::A},

    /* 0yE0 */                   /* 0yE1 */                   /* 0yE2 */                   /* 0yE3 */                   /* 0yE4 */                   /* 0yE5 */                   /* 0yE6 */        /* 0yE7 */
    &SetBMemIyDR{b:4,r:Reg8::B}, &SetBMemIyDR{b:4,r:Reg8::C}, &SetBMemIyDR{b:4,r:Reg8::D}, &SetBMemIyDR{b:4,r:Reg8::E}, &SetBMemIyDR{b:4,r:Reg8::H}, &SetBMemIyDR{b:4,r:Reg8::L}, &SetBMemIyD{b:4}, &SetBMemIyDR{b:4,r:Reg8::A},

    /* 0yE8 */                   /* 0yE9 */                   /* 0yEA */                   /* 0yEB */                   /* 0yEC */                   /* 0yED */                   /* 0yEE */        /* 0yEF */
    &SetBMemIyDR{b:5,r:Reg8::B}, &SetBMemIyDR{b:5,r:Reg8::C}, &SetBMemIyDR{b:5,r:Reg8::D}, &SetBMemIyDR{b:5,r:Reg8::E}, &SetBMemIyDR{b:5,r:Reg8::H}, &SetBMemIyDR{b:5,r:Reg8::L}, &SetBMemIyD{b:5}, &SetBMemIyDR{b:5,r:Reg8::A},

    /* 0yF0 */                   /* 0yF1 */                   /* 0yF2 */                   /* 0yF3 */                   /* 0yF4 */                   /* 0yF5 */                   /* 0yF6 */        /* 0yF7 */
    &SetBMemIyDR{b:6,r:Reg8::B}, &SetBMemIyDR{b:6,r:Reg8::C}, &SetBMemIyDR{b:6,r:Reg8::D}, &SetBMemIyDR{b:6,r:Reg8::E}, &SetBMemIyDR{b:6,r:Reg8::H}, &SetBMemIyDR{b:6,r:Reg8::L}, &SetBMemIyD{b:6}, &SetBMemIyDR{b:6,r:Reg8::A},

    /* 0yF8 */                   /* 0yF9 */                   /* 0yFA */                   /* 0yFB */                   /* 0yFC */                   /* 0yFD */                   /* 0yFE */        /* 0yFF */
    &SetBMemIyDR{b:7,r:Reg8::B}, &SetBMemIyDR{b:7,r:Reg8::C}, &SetBMemIyDR{b:7,r:Reg8::D}, &SetBMemIyDR{b:7,r:Reg8::E}, &SetBMemIyDR{b:7,r:Reg8::H}, &SetBMemIyDR{b:7,r:Reg8::L}, &SetBMemIyD{b:7}, &SetBMemIyDR{b:7,r:Reg8::A},
];

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

