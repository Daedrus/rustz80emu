use super::instructions::{Instruction, update_flags_logical, update_flags_bit, update_xyflags_bit};
use super::cpu::*;
use ::debugger::*;

struct RlcMemIxDR { r: Reg8 }
struct RlcMemIxD  ;

impl Instruction for RlcMemIxDR {
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

        //TODO info!("{:#06x}: RLC (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RlcMemIxD {
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

        //TODO info!("{:#06x}: RLC (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}


struct RrcMemIxDR { r: Reg8 }
struct RrcMemIxD  ;

impl Instruction for RrcMemIxDR {
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

        //TODO info!("{:#06x}: RRC (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RrcMemIxD {
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

        //TODO info!("{:#06x}: RRC (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}


struct RlMemIxDR  { r: Reg8 }
struct RlMemIxD   ;

impl Instruction for RlMemIxDR {
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

        //TODO info!("{:#06x}: RL (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RlMemIxD {
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

        //TODO info!("{:#06x}: RL (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}


struct RrMemIxDR  { r: Reg8 }
struct RrMemIxD   ;

impl Instruction for RrMemIxDR {
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

        //TODO info!("{:#06x}: RR (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for RrMemIxD {
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

        //TODO info!("{:#06x}: RR (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}


struct SlaMemIxDR { r: Reg8 }
struct SlaMemIxD  ;

impl Instruction for SlaMemIxDR {
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

        //TODO info!("{:#06x}: SLA (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SlaMemIxD {
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

        //TODO info!("{:#06x}: SLA (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OA|OF|OIX|OWZ, OF|OWZ)
    }
}


struct SraMemIxDR { r: Reg8 }
struct SraMemIxD  ;

impl Instruction for SraMemIxDR {
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

        //TODO info!("{:#06x}: SRA (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SraMemIxD {
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

        //TODO info!("{:#06x}: SRA (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}


struct SllMemIxDR { r: Reg8 }
struct SllMemIxD  ;

impl Instruction for SllMemIxDR {
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

        //TODO info!("{:#06x}: SLL (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SllMemIxD {
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

        //TODO info!("{:#06x}: SLL (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}


struct SrlMemIxDR { r: Reg8 }
struct SrlMemIxD  ;

impl Instruction for SrlMemIxDR {
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

        //TODO info!("{:#06x}: SRL (IX{:+#04X}), {:?}", cpu.get_pc() - 2, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OutputRegisters::from(self.r), OF|OutputRegisters::from(self.r))
    }
}

impl Instruction for SrlMemIxD {
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

        //TODO info!("{:#06x}: SRL (IX{:+#04X})", cpu.get_pc() - 2, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX|OWZ, OF|OWZ)
    }
}


struct BitBMemIxD { b: u8 }

impl Instruction for BitBMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        update_flags_bit(cpu, self.b, memval & (1 << self.b) != 0);
        update_xyflags_bit(cpu);

        //TODO
        //info!("{:#06x}: BIT {}, (IX{:+#04X})", cpu.get_pc() - 2, self.b, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OF|OIX, OF)
    }
}


struct ResBMemIxDR { b: u8, r: Reg8 }
struct ResBMemIxD  { b: u8 }

impl Instruction for ResBMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval & !(1 << self.b));
        cpu.write_word(addr, memval & !(1 << self.b));

        //TODO info!("{:#06x}: RES {}, (IX{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}

impl Instruction for ResBMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval & !(1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        //TODO
        //info!("{:#06x}: RES {}, (IX{:+#04X})", cpu.get_pc() - 2, self.b, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OWZ, OWZ)
    }
}


struct SetBMemIxDR { b: u8, r: Reg8 }
struct SetBMemIxD  { b: u8 }

impl Instruction for SetBMemIxDR {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_reg8(self.r, memval | (1 << self.b));
        cpu.write_word(addr, memval | (1 << self.b));

        //TODO info!("{:#06x}: SET {}, (IX{:+#04X}), {:?}", cpu.get_pc() - 2, self.b, offset, self.r);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OutputRegisters::from(self.r), OutputRegisters::from(self.r))
    }
}

impl Instruction for SetBMemIxD {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::WZ);
        let memval = cpu.read_word(addr);

        cpu.contend_read_no_mreq(addr);

        cpu.write_word(addr, memval | (1 << self.b));
        cpu.write_reg16(Reg16::WZ, addr);

        //TODO info!("{:#06x}: SET {}, (IX{:+#04X})", cpu.get_pc() - 2, self.b, offset);
        cpu.inc_pc(2);
    }

    fn get_accessed_regs(&self) -> (OutputRegisters, OutputRegisters) {
        (OIX|OWZ, OWZ)
    }
}


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
