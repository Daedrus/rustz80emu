use super::instructions::{Instruction, update_flags_logical, update_flags_bit, update_xyflags_bit};
use super::cpu::*;
use ::debugger::output_registers::*;
use ::peripherals::Memory;


struct RlcMemIyDR { r: Reg8 }
struct RlcMemIyD  ;

impl Instruction for RlcMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_left(1);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RLC (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for RlcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_left(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RLC (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct RrcMemIyDR { r: Reg8 }
struct RrcMemIyD  ;

impl Instruction for RrcMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_right(1);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RRC (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for RrcMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval.rotate_right(1);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RRC (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct RlMemIyDR  { r: Reg8 }
struct RlMemIyD   ;

impl Instruction for RlMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RL (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for RlMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RL (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct RrMemIyDR  { r: Reg8 }
struct RrMemIyD   ;

impl Instruction for RrMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RR (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for RrMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let mut res = memval.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RR (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct SlaMemIyDR { r: Reg8 }
struct SlaMemIyD  ;

impl Instruction for SlaMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SLA (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for SlaMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SLA (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct SraMemIyDR { r: Reg8 }
struct SraMemIyD  ;

impl Instruction for SraMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1 | (memval & 0x80);

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SRA (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for SraMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1 | (memval & 0x80);

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SRA (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct SllMemIyDR { r: Reg8 }
struct SllMemIyD  ;

impl Instruction for SllMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1 | 0x1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SLL (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for SllMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval << 1 | 0x1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x80 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SLL (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct SrlMemIyDR { r: Reg8 }
struct SrlMemIyD  ;

impl Instruction for SrlMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1;

        cpu.write_reg8(self.r, res);
        cpu.write_word(addr, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SRL (IY{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r)
    }
}

impl Instruction for SrlMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);
        let res = memval >> 1;

        cpu.write_word(addr, res);
        cpu.write_reg16(Reg16::WZ, addr);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                      );
        cpu.cond_flag  ( CARRY_FLAG      , memval & 0x01 != 0 );

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY|OWZ, OF|OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SRL (IY{:+#04X})", cpu.get_pc() - 2, offset)
    }
}


struct BitBMemIyD { b: u8 }

impl Instruction for BitBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        update_flags_bit(cpu, self.b, memval & (1 << self.b) != 0);
        update_xyflags_bit(cpu);

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIY, OF)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: BIT {}, (IY{:+#04X})", cpu.get_pc() - 2, self.b, offset)
    }
}


struct ResBMemIyDR { b: u8, r: Reg8 }
struct ResBMemIyD  { b: u8 }

impl Instruction for ResBMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval & !(1 << self.b));
        cpu.write_word(addr, memval & !(1 << self.b));

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RES {}, (IY{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, offset, self.r)
    }
}

impl Instruction for ResBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval & !(1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: RES {}, (IY{:+#04X})", cpu.get_pc() - 2, self.b, offset)
    }
}


struct SetBMemIyDR { b: u8, r: Reg8 }
struct SetBMemIyD  { b: u8 }

impl Instruction for SetBMemIyDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval | (1 << self.b));
        cpu.write_word(addr, memval | (1 << self.b));

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SET {}, (IY{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, offset, self.r)
    }
}

impl Instruction for SetBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval | (1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIY|OWZ, OWZ)
    }

    fn get_string(&self, cpu: &Cpu, memory: &Memory) -> String {
        let offset = memory.read_word(cpu.get_pc()) as i16;
        format!("{:#06x}: SET {}, (IY{:+#04X})", cpu.get_pc() - 2, self.b, offset)
    }
}


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

