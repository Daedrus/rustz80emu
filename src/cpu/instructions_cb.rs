use super::instructions::{Instruction, update_flags_logical, update_flags_bit, update_xyflags_bit};
use super::cpu::*;
use ::debugger::output_registers::*;
use ::peripherals::Memory;


struct RlcR       { r: Reg8 }
struct RlcMemHl   ;

impl Instruction for RlcR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r.rotate_left(1);

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG           );
        cpu.cond_flag  ( CARRY_FLAG, r & 0x80 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RLC {:?}", cpu.get_pc() - 1, self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RLC (HL)", cpu.get_pc() - 1)
    }
}


struct RrcR       { r: Reg8 }
struct RrcMemHl   ;

impl Instruction for RrcR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r.rotate_right(1);

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG           );
        cpu.cond_flag  ( CARRY_FLAG, r & 0x01 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RRC {:?}", cpu.get_pc() - 1, self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RRC (HL)", cpu.get_pc() - 1)
    }
}


struct RlR        { r: Reg8 }
struct RlMemHl    ;

impl Instruction for RlR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let mut res = r.rotate_left(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x01; } else { res &= 0xFE; }

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x80 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RL {:?}", cpu.get_pc() - 1, self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RL (HL)", cpu.get_pc() - 1)
    }
}


struct RrR        { r: Reg8 }
struct RrMemHl    ;

impl Instruction for RrR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let mut res = r.rotate_right(1);
        if cpu.get_flag(CARRY_FLAG) { res |= 0x80; } else { res &= 0x7F; }

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG           );
        cpu.cond_flag  ( CARRY_FLAG, r & 0x01 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RR {:?}", cpu.get_pc() - 1, self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RR (HL)", cpu.get_pc() - 1)
    }
}


struct SlaR       { r: Reg8 }
struct SlaMemHl   ;

impl Instruction for SlaR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r << 1;

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x80 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SLA {:?}", cpu.get_pc(), self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SLA (HL)", cpu.get_pc() - 1)
    }
}


struct SraR       { r: Reg8 }
struct SraMemHl   ;

impl Instruction for SraR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r >> 1 | (r & 0x80);

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x01 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SRA {:?}", cpu.get_pc(), self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SRA (HL)", cpu.get_pc() - 1)
    }
}


struct SllR       { r: Reg8 }
struct SllMemHl   ;

impl Instruction for SllR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r << 1 | 0x1;

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x80 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SLL {:?}", cpu.get_pc(), self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SLL (HL)", cpu.get_pc() - 1)
    }
}


struct SrlR       { r: Reg8 }
struct SrlMemHl   ;

impl Instruction for SrlR {
    fn execute(&self, cpu: &mut Cpu) {
        let r = cpu.read_reg8(self.r);

        let res = r >> 1;

        cpu.write_reg8(self.r, res);

        update_flags_logical(cpu, res);
        cpu.clear_flag ( HALF_CARRY_FLAG                 );
        cpu.cond_flag  ( CARRY_FLAG      , r & 0x01 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SRL {:?}", cpu.get_pc(), self.r)
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

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SRL (HL)", cpu.get_pc() - 1)
    }
}


struct BitBR      { b: u8, r: Reg8 }
struct BitBMemHl  { b: u8 }

impl Instruction for BitBR {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);

        update_flags_bit(cpu, self.b, val & (1 << self.b) != 0);
        cpu.cond_flag ( X_FLAG , val & 0x08 != 0 );
        cpu.cond_flag ( Y_FLAG , val & 0x20 != 0 );

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: BIT {}, {:?}", cpu.get_pc() - 1, self.b, self.r)
    }
}

impl Instruction for BitBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        update_flags_bit(cpu, self.b, memval & (1 << self.b) != 0);
        update_xyflags_bit(cpu);

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OH|OL, OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: BIT {}, (HL)", cpu.get_pc() - 1, self.b)
    }
}



struct ResBR       { b: u8, r: Reg8 }
struct ResBMemHl   { b: u8 }

impl Instruction for ResBR {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);

        cpu.write_reg8(self.r, val & !(1 << self.b));

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RES {}, {:?}", cpu.get_pc() - 1, self.b, self.r)
    }
}

impl Instruction for ResBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        cpu.write_word(hl, memval & !(1 << self.b));

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, ONONE)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: RES {}, (HL)", cpu.get_pc() - 1, self.b)
    }
}


struct SetBR       { b: u8, r: Reg8 }
struct SetBMemHl   { b: u8 }

impl Instruction for SetBR {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);

        cpu.write_reg8(self.r, val | (1 << self.b));

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OutputRegisters::from(self.r), OF)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SET {}, {:?}", cpu.get_pc() - 1, self.b, self.r)
    }
}

impl Instruction for SetBMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hl     = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hl);

        cpu.contend_read_no_mreq(hl);

        cpu.write_word(hl, memval | (1 << self.b));

        cpu.inc_pc(1);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OH|OL, ONONE)
    }

    fn get_string(&self, cpu: &Cpu, _memory: &Memory) -> String {
        format!("{:#06x}: SET {}, (HL)", cpu.get_pc() - 1, self.b)
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

