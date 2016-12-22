use super::instructions::{Instruction, Unsupported, update_flags_sub8};
use super::cpu::*;
use ::debugger::*;


struct InRPortC { r: Reg8 }
struct InPortC  ;

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


struct OutPortCR { r: Reg8 }
struct OutPortC  ;

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


struct SbcHlSs   { r: Reg16 }

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


struct AdcHlSs   { r: Reg16 }

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


pub struct LdMemNnDd { pub r: Reg16 }
pub struct LdDdMemNn { pub r: Reg16 }

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


struct RetN;

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


struct LdIA      ;
struct LdRA      ;
struct LdAI      ;
struct LdAR      ;

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

        cpu.cond_flag  ( SIGN_FLAG            , r & 0x80 != 0  );
        cpu.cond_flag  ( ZERO_FLAG            , r == 0         );
        cpu.clear_flag ( HALF_CARRY_FLAG                       );
        let iff2 = cpu.get_iff2();
        cpu.cond_flag  ( PARITY_OVERFLOW_FLAG , iff2           );
        cpu.clear_flag ( ADD_SUBTRACT_FLAG                     );

        info!("{:#06x}: LD A,R", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OR, OA)
    }
}

struct Ldi;
struct Ldir;
struct Ldd;
struct Lddr;

#[inline(always)]
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

#[inline(always)]
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


struct Cpi;
struct Cpir;
struct Cpd;
struct Cpdr;

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


struct Ini ;
struct Inir;
struct Ind ;
struct Indr;

// TODO
impl Instruction for Ini {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: INI", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        //unreachable!();
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
        //unreachable!();
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
        //unreachable!();
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
        //unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
    }
}


struct Outi;
struct Otir;
struct Outd;
struct Otdr;

// TODO
impl Instruction for Outi {
    fn execute(&self, cpu: &mut Cpu) {
        info!("{:#06x}: OUTI", cpu.get_pc() - 1);
        cpu.inc_pc(1);
        //unreachable!();
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
        //unreachable!();
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
        //unreachable!();
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
        //unreachable!();
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (ONONE, ONONE)
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

