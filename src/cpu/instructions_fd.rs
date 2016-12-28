use super::instructions::{Instruction, update_flags_logical, update_flags_add8, update_flags_add16,
    update_flags_dec8, update_flags_inc8, update_flags_adc8, update_flags_sub8, update_flags_sbc8,
    update_flags_cp8, Unsupported, PopQq, PushQq, CpR, OrR, XorR, AndR, SbcR, SubR, AdcR, AddR, LdRR,
    LdRN, DecR, IncR, IncSs, DecSs, LdDdNn, INSTR_TABLE};
use super::instructions_ed::{LdDdMemNn, LdMemNnDd};
use super::cpu::*;
use ::debugger::output_registers::*;
use ::peripherals::Memory;


struct AddIyRr   { r: Reg16 }

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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL|OIY, OH|OL|OF|OIY)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: ADD IY, {:?}", cpu.get_pc(), self.r)
    }
}


struct IncMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: INC (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct DecMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: DEC (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct LdMemIyDR  { r: Reg8  }
struct LdMemIyDN  ;
struct LdRMemIyD  { r: Reg8  }

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: LD (IY{:+#04X}), {:?}", cpu.get_pc() - 1, d, self.r)
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

        cpu.inc_pc(3);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        let n = memory.read_word(cpu.get_pc() + 2);
        format!("{:#06x}: LD (IY{:+#04X}), {:#04X}", cpu.get_pc() - 1, d, n)
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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ|OutputRegisters::from(self.r), OWZ|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: LD {:?}, (IY{:+#04X})", cpu.get_pc() - 1, self.r, d)
    }
}


struct AddMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: ADD A, (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct AdcMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: ADC A, (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct SubMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: SUB A, (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct SbcMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OA|OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: SBC A, (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct AndMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: AND A, (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct XorMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: XOR A, (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct OrMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OA|OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: OR A, (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct CpMemIyD ;

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

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let d = memory.read_word(cpu.get_pc() + 1) as i8;
        format!("{:#06x}: CP (IY{:+#04X})", cpu.get_pc() - 1, d)
    }
}


struct ExMemSpIy;

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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OIY, OIY)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: EX (SP), IY", cpu.get_pc() - 1)
    }
}


struct JpIy;

impl Instruction for JpIy {
    fn execute(&self, cpu: &mut Cpu) {
        let iy = cpu.read_reg16(Reg16::IY);

        cpu.set_pc(iy);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY, ONONE)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: JP IY", cpu.get_pc() - 1)
    }
}


struct LdSpIy;

impl Instruction for LdSpIy {
    fn execute(&self, cpu: &mut Cpu) {
        let iy = cpu.read_reg16(Reg16::IY);

        let ir = cpu.read_reg16(Reg16::IR);
        cpu.contend_read_no_mreq(ir);
        cpu.contend_read_no_mreq(ir);

        cpu.write_reg16(Reg16::SP, iy);

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OSP|OIY, OSP)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: LD SP, IY", cpu.get_pc() - 1)
    }
}


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

