use super::cpu::*;
use num::FromPrimitive;

pub trait Instruction {
    fn execute(&self, &mut Cpu);
}

struct Unsupported;

impl Instruction for Unsupported {
    fn execute(&self, cpu: &mut Cpu) {
        println!("{:?}", cpu);

        panic!("Unsupported instruction {:#x} at address {:#06x}", cpu.read_word(cpu.get_pc()), cpu.get_pc());
    }
}


struct AddAN      ;
struct AddAR      { r: Reg8  }
struct AddHlSs    { r: Reg16 }
struct AddAMemIyD ;

impl Instruction for AddAN {
    fn execute(&self, cpu: &mut Cpu) {
        let aval = cpu.read_reg8(Reg8::A);
        let n = cpu.read_word(cpu.get_pc() + 1);

        let addval = aval.wrapping_add(n);
        cpu.write_reg8(Reg8::A, addval);

        if addval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if addval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if ((aval & 0x0F) + (n & 0x0F)) > 0x0f { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        match (aval & 0b10000000   != 0,
               n    & 0b10000000   != 0,
               addval & 0b10000000 != 0) {
            (true, true, false) | (false, false, true) => cpu.set_flag(PARITY_OVERFLOW_FLAG),
            _ => cpu.clear_flag(PARITY_OVERFLOW_FLAG)
        };
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if aval as u16 + n as u16 > 0xFF { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }

        println!("{:#06x}: ADD A, {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}

impl Instruction for AddAR {
    fn execute(&self, cpu: &mut Cpu) {
        let aval = cpu.read_reg8(Reg8::A);
        let rval = cpu.read_reg8(self.r);

        let addval = aval.wrapping_add(rval);
        cpu.write_reg8(Reg8::A, addval);

        if addval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if addval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if ((aval & 0x0F) + (rval & 0x0F)) > 0x0f { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        match (aval   & 0b10000000 != 0,
               rval   & 0b10000000 != 0,
               addval & 0b10000000 != 0) {
            (true, true, false) | (false, false, true) => cpu.set_flag(PARITY_OVERFLOW_FLAG),
            _ => cpu.clear_flag(PARITY_OVERFLOW_FLAG)
        };
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if aval as u16 + rval as u16 > 0xFF { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }

        println!("{:#06x}: ADD A, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for AddHlSs {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);
        let rval = cpu.read_reg16(self.r);

        let addval = hlval.wrapping_add(rval);

        cpu.write_reg16(Reg16::HL, addval);

        if addval & 0x8000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if addval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if ((hlval & 0xfff) + (rval & 0xfff)) > 0xfff { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        match (hlval  & 0x8000 != 0,
               rval   & 0x8000 != 0,
               addval & 0x8000 != 0) {
            (true, true, false) | (false, false, true) => cpu.set_flag(PARITY_OVERFLOW_FLAG),
            _ => cpu.clear_flag(PARITY_OVERFLOW_FLAG)
        };
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if hlval as u32 + rval as u32 > 0xFFFF { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); };

        println!("{:#06x}: ADD HL, {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for AddAMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let aval = cpu.read_reg8(Reg8::A);
        let d = cpu.read_word(curr_pc + 1) as i16;
        let addr = ((cpu.get_iy() as i16) + d) as u16;
        let memval = cpu.read_word(addr);

        let addval = aval.wrapping_add(memval);
        cpu.write_reg8(Reg8::A, addval);

        if addval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if addval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if ((aval & 0x0F) + (memval & 0x0F)) > 0x0f { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        match (aval   & 0b10000000 != 0,
               memval & 0b10000000 != 0,
               addval & 0b10000000 != 0) {
            (true, true, false) | (false, false, true) => cpu.set_flag(PARITY_OVERFLOW_FLAG),
            _ => cpu.clear_flag(PARITY_OVERFLOW_FLAG)
        };
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if aval as u16 + memval as u16 > 0xFF { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: ADD A, (IY-{:#04X})", curr_pc - 1, d);
        } else {
            println!("{:#06x}: ADD A, (IY+{:#04X})", curr_pc - 1, d);
        }
        cpu.inc_pc(2);
    }
}


struct AndR { r: Reg8 }
struct AndN ;

impl Instruction for AndR {
    fn execute(&self, cpu: &mut Cpu) {
        let aval = cpu.read_reg8(Reg8::A);
        let rval = cpu.read_reg8(self.r);
        let andval = aval & rval;

        cpu.write_reg8(Reg8::A, andval);

        cpu.set_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        cpu.clear_flag(CARRY_FLAG);
        if andval.count_ones() % 2 == 0 { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }
        if andval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if andval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }

        println!("{:#06x}: AND {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for AndN {
    fn execute(&self, cpu: &mut Cpu) {
        let n = cpu.read_word(cpu.get_pc() + 1);
        let andval = n & cpu.read_reg8(Reg8::A);

        cpu.write_reg8(Reg8::A, andval);

        cpu.set_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        cpu.clear_flag(CARRY_FLAG);
        if andval.count_ones() % 2 == 0 { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }
        if andval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if andval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }

        println!("{:#06x}: AND {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}


struct BitBMemIyD { b: u8 }

impl Instruction for BitBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc) as i16;
        let addr = ((cpu.get_iy() as i16) + d) as u16;

        let memval = cpu.read_word(addr);

        if memval & (1 << self.b) == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        cpu.set_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: BIT {}, (IY-{:#04X})", curr_pc - 2, self.b, d);
        } else {
            println!("{:#06x}: BIT {}, (IY+{:#04X})", curr_pc - 2, self.b, d);
        }
        cpu.inc_pc(2);
    }
}


struct CallNn;

impl Instruction for CallNn {
    fn execute(&self, cpu: &mut Cpu) {
        let mut curr_pc = cpu.get_pc();
        let nn =  (cpu.read_word(curr_pc + 1) as u16) |
                 ((cpu.read_word(curr_pc + 2) as u16) << 8);
        let curr_sp = cpu.read_reg16(Reg16::SP);

        curr_pc += 3;
        cpu.write_word(curr_sp - 1, ((curr_pc & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (curr_pc & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        println!("{:#06x}: CALL {:#06X}", cpu.get_pc(), nn);
        cpu.set_pc(nn);
    }
}


struct Ccf;

impl Instruction for Ccf {
    fn execute(&self, cpu: &mut Cpu) {
        let cfval = cpu.get_flag(CARRY_FLAG);

        if cfval { cpu.clear_flag(CARRY_FLAG); } else { cpu.set_flag(CARRY_FLAG); }
        if cfval { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        println!("{:#06x}: CCF", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


struct CpR     { r: Reg8 }
struct CpMemHl ;

impl Instruction for CpR {
    fn execute(&self, cpu: &mut Cpu) {
        let rval = cpu.read_reg8(self.r);
        let accval = cpu.read_reg8(Reg8::A);

        cpu.set_flag(ADD_SUBTRACT_FLAG);
        if rval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if rval == accval { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if accval < rval { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }
        if (accval & 0x0F) < (rval & 0x0F) { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        //TODO: Parity flag?

        println!("{:#06x}: CP {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for CpMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let addr = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(addr);
        let accval = cpu.read_reg8(Reg8::A);

        cpu.set_flag(ADD_SUBTRACT_FLAG);
        if memval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if memval == accval { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if accval < memval { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }
        if (accval & 0x0F) < (memval & 0x0F) { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        //TODO: Parity flag?

        println!("{:#06x}: CP (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


struct DecSs  { r: Reg16 }
struct DecR   { r: Reg8  }
struct DecIyD ;

impl Instruction for DecSs {
    fn execute(&self, cpu: &mut Cpu) {
        let oldregval = cpu.read_reg16(self.r);
        cpu.write_reg16(self.r, oldregval - 1);

        println!("{:#06x}: DEC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for DecR {
    fn execute(&self, cpu: &mut Cpu) {
        let decval = cpu.read_reg8(self.r) - 1;
        cpu.write_reg8(self.r, decval);

        cpu.set_flag(ADD_SUBTRACT_FLAG);
        if decval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if decval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if decval & 0b00001111 == 0 { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        if decval == 0x7F { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }

        println!("{:#06x}: DEC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for DecIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc + 1) as i8 as i16;
        let addr = ((cpu.get_iy() as i16) + d) as u16;

        let decval = cpu.read_word(addr).wrapping_sub(1);
        cpu.write_word(addr, decval);

        if decval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if decval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if decval & 0b00001111 == 0 { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        if decval == 0x7F { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }
        cpu.set_flag(ADD_SUBTRACT_FLAG);

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: DEC (IY-{:#04X})", curr_pc - 1, d);
        } else {
            println!("{:#06x}: DEC (IY+{:#04X})", curr_pc - 1, d);
        }
        cpu.inc_pc(2);
    }
}


struct Di;

impl Instruction for Di {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.clear_iff1();
        cpu.clear_iff2();

        println!("{:#06x}: DI", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


struct Djnz;

impl Instruction for Djnz {
    fn execute(&self, cpu: &mut Cpu) {
        let bval = cpu.read_reg8(Reg8::B) - 1;
        cpu.write_reg8(Reg8::B, bval);
        let curr_pc = cpu.get_pc();
        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (curr_pc as i16 + offset as i16) as u16;

        println!("{:#06x}: DJNZ {:#06X}", cpu.get_pc(), target);
        if bval > 0 {
            cpu.set_pc(target);
        } else {
            cpu.inc_pc(2);
        }
    }
}


struct Ei;

impl Instruction for Ei {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_iff1();
        cpu.set_iff2();

        println!("{:#06x}: EI", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


struct ExMemSpHl;
struct ExDeHl;

impl Instruction for ExMemSpHl {
    fn execute(&self, cpu: &mut Cpu) {
        let spval = cpu.read_reg16(Reg16::SP);
        let hlval = cpu.read_reg16(Reg16::HL);
        let (hlhigh, hllow) = (((hlval & 0xFF00) >> 8) as u8,
                               ((hlval & 0x00FF)       as u8));

        let memval =  (cpu.read_word(spval    ) as u16) |
                     ((cpu.read_word(spval + 1) as u16) << 8);

        cpu.write_reg16(Reg16::HL, memval);

        cpu.write_word(spval, hllow);
        cpu.write_word(spval + 1, hlhigh);

        println!("{:#06x}: EX (SP), HL", cpu.get_pc());
        cpu.inc_pc(1);
    }
}

impl Instruction for ExDeHl {
    fn execute(&self, cpu: &mut Cpu) {
        let deval = cpu.read_reg16(Reg16::DE);
        let hlval = cpu.read_reg16(Reg16::HL);

        cpu.write_reg16(Reg16::DE, hlval);
        cpu.write_reg16(Reg16::HL, deval);

        println!("{:#06x}: EX DE, HL", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


struct Exx;

impl Instruction for Exx {
    fn execute(&self, cpu: &mut Cpu) {
        let bcval = cpu.read_reg16(Reg16::BC);
        let deval = cpu.read_reg16(Reg16::DE);
        let hlval = cpu.read_reg16(Reg16::HL);

        let bcaltval = cpu.read_reg16(Reg16::BC_ALT);
        let dealtval = cpu.read_reg16(Reg16::DE_ALT);
        let hlaltval = cpu.read_reg16(Reg16::HL_ALT);

        cpu.write_reg16(Reg16::BC, bcaltval);
        cpu.write_reg16(Reg16::DE, dealtval);
        cpu.write_reg16(Reg16::HL, hlaltval);

        cpu.write_reg16(Reg16::BC_ALT, bcval);
        cpu.write_reg16(Reg16::DE_ALT, deval);
        cpu.write_reg16(Reg16::HL_ALT, hlval);

        println!("{:#06x}: EXX", cpu.get_pc());

        cpu.inc_pc(1);
    }
}


struct Im { mode: u8 }

impl Instruction for Im {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_im(self.mode);

        println!("{:#06x}: IM {}", cpu.get_pc() - 1, self.mode);
        cpu.inc_pc(1);
    }
}


struct IncR  { r: Reg8  }
struct IncSs { r: Reg16 }

impl Instruction for IncR {
    fn execute(&self, cpu: &mut Cpu) {
        let rval = cpu.read_reg8(self.r);
        let incval = rval.wrapping_add(1);
        cpu.write_reg8(self.r, incval);

        if incval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if incval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if incval & 0b00001111 == 0 { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        if rval == 0x7F { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        println!("{:#06x}: INC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for IncSs {
    fn execute(&self, cpu: &mut Cpu) {
        let incval = cpu.read_reg16(self.r) + 1;
        cpu.write_reg16(self.r, incval);

        println!("{:#06x}: INC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}


struct JpMemHl;
struct JpNn   ;

impl Instruction for JpMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);

        println!("{:#06x}: JP (HL)", cpu.get_pc());
        cpu.set_pc(hlval);
    }
}

impl Instruction for JpNn {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        println!("{:#06x}: JP {:#06X}", cpu.get_pc(), nn);
        cpu.set_pc(nn);
    }
}


struct JrZ ;
struct JrNz;
struct JrCE;
struct JrE ;

impl Instruction for JrZ {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (curr_pc as i16 + offset as i16) as u16;

        println!("{:#06x}: JR Z {:#06X}", cpu.get_pc(), target);
        if cpu.get_flag(ZERO_FLAG) {
            cpu.set_pc(target);
        } else {
            cpu.inc_pc(2);
        }
    }
}

impl Instruction for JrNz {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (curr_pc as i16 + offset as i16) as u16;

        println!("{:#06x}: JR NZ {:#06X}", cpu.get_pc(), target);
        if cpu.get_flag(ZERO_FLAG) {
            cpu.inc_pc(2);
        } else {
            cpu.set_pc(target);
        }
    }
}

impl Instruction for JrCE {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (curr_pc as i16 + offset as i16) as u16;

        println!("{:#06x}: JR C, {:#06X}", cpu.get_pc(), target);

        if cpu.get_flag(CARRY_FLAG) {
            cpu.set_pc(target);
        } else {
            cpu.inc_pc(2);
        }
    }
}

impl Instruction for JrE {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (curr_pc as i16 + offset as i16) as u16;

        println!("{:#06x}: JR {:#06X}", cpu.get_pc(), target);
        cpu.set_pc(target);
    }
}


struct LdRN      { r: Reg8  }
struct LdDdNn    { r: Reg16 }
struct LdHlMemNn ;
struct LdMemHlN  ;
struct LdRMemIyD { r: Reg8  }
struct LdMemIyDN ;
struct LdSpHl    ;
struct LdIxNn    ;
struct LdIxMemNn ;
struct LdMemNnIx ;
struct LdMemIxDN ;
struct LdRR      { rt: Reg8, rs: Reg8 }
struct LdMemNnDd { r: Reg16 }
struct LdMemHlR  { r: Reg8  }
struct LdMemNnA  ;
struct LdRMemHl  { r: Reg8  }
struct LdMemNnHl ;
struct LdAMemNn  ;
struct LdIyNn    ;

impl Instruction for LdRN {
    fn execute(&self, cpu: &mut Cpu) {
        let n = cpu.read_word(cpu.get_pc() + 1);
        cpu.write_reg8(self.r, n);

        println!("{:#06x}: LD {:?}, {:#04X}", cpu.get_pc(), self.r, n);
        cpu.inc_pc(2);
    }
}

impl Instruction for LdDdNn {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        cpu.write_reg16(self.r, nn);

        println!("{:#06x}: LD {:?}, {:#06X}", cpu.get_pc(), self.r, nn);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdHlMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let nnmemval = (cpu.read_word(nn    ) as u16) |
                      ((cpu.read_word(nn + 1) as u16) << 8);

        cpu.write_reg16(Reg16::HL, nnmemval);

        println!("{:#06x}: LD HL, ({:#06X})", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdMemHlN {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);

        let n =  cpu.read_word(cpu.get_pc() + 1);

        cpu.write_word(hlval, n);

        println!("{:#06x}: LD (HL), {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}

impl Instruction for LdRMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc + 1);
        let addr = cpu.get_iy() as i16 + d as i16;

        let memval = cpu.read_word(addr as u16);

        cpu.write_reg8(self.r, memval);

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: LD {:?}, (IY-{:#04X})", curr_pc - 1, self.r, d);
        } else {
            println!("{:#06x}: LD {:?}, (IY+{:#04X})", curr_pc - 1, self.r, d);
        }

        cpu.inc_pc(2);
    }
}

impl Instruction for LdMemIyDN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc + 1);
        let n = cpu.read_word(curr_pc + 2);
        let addr = cpu.get_iy() as i16 + d as i16;
        cpu.write_word(addr as u16, n);

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: LD (IY-{:#04X}), {:#04X}", curr_pc - 1, d, n);
        } else {
            println!("{:#06x}: LD (IY+{:#04X}), {:#04X}", curr_pc - 1, d, n);
        }

        cpu.inc_pc(3);
    }
}

impl Instruction for LdSpHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);
        cpu.write_reg16(Reg16::SP, hlval);

        println!("{:#06x}: LD SP, HL", cpu.get_pc());
        cpu.inc_pc(1);
    }
}

impl Instruction for LdIxNn {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.set_ix(nn);

        println!("{:#06x}: LD IX, {:#06X}", cpu.get_pc() - 1, nn);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdIxMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let nnmemval = (cpu.read_word(nn    ) as u16) |
                      ((cpu.read_word(nn + 1) as u16) << 8);

        cpu.set_ix(nnmemval);

        println!("{:#06x}: LD IX, TODO {:#06X}", cpu.get_pc() - 1, nnmemval);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdMemNnIx {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let (ixhigh, ixlow) = (((cpu.get_ix() & 0xFF00) >> 8) as u8,
                               ((cpu.get_ix() & 0x00FF)       as u8));

        cpu.write_word(nn, ixlow);
        cpu.write_word(nn + 1, ixhigh);

        println!("{:#06x}: LD ({:#06X}), IX", cpu.get_pc() - 1, nn);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdMemIxDN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc + 1);
        let n = cpu.read_word(curr_pc + 2);
        let addr = cpu.get_ix() as i16 + d as i16;
        cpu.write_word(addr as u16, n);

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: LD (IX-{:#04X}), {:#04X}", curr_pc - 1, d, n);
        } else {
            println!("{:#06x}: LD (IX+{:#04X}), {:#04X}", curr_pc - 1, d, n);
        }

        cpu.inc_pc(3);
    }
}

impl Instruction for LdRR {
    fn execute(&self, cpu: &mut Cpu) {
        let rsval = cpu.read_reg8(self.rs);
        cpu.write_reg8(self.rt, rsval);
        println!("{:#06x}: LD {:?}, {:?}", cpu.get_pc(), self.rt, self.rs);
        cpu.inc_pc(1);
    }
}

impl Instruction for LdMemNnDd {
    fn execute(&self, cpu: &mut Cpu) {
        let rval = cpu.read_reg16(self.r);
        let (rhigh, rlow) = (((rval & 0xFF00) >> 8) as u8,
                             ((rval & 0x00FF)       as u8));

        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.write_word(nn, rlow);
        cpu.write_word(nn + 1, rhigh);

        println!("{:#06x}: LD ({:#06X}), {:?}", cpu.get_pc() - 1, nn, self.r);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdMemHlR {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);
        let addr = cpu.read_reg16(Reg16::HL);

        cpu.write_word(addr, val);

        println!("{:#06x}: LD (HL), {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for LdMemNnA {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let aval = cpu.read_reg8(Reg8::A);

        cpu.write_word(nn, aval);

        println!("{:#06x}: LD ({:#06X}), A", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdRMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hlmemval = cpu.read_word(cpu.read_reg16(Reg16::HL));
        cpu.write_reg8(self.r, hlmemval);

        println!("{:#06x}: LD {:?}, (HL)", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for LdMemNnHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);
        let (hlhigh, hllow) = (((hlval & 0xFF00) >> 8) as u8,
                               ((hlval & 0x00FF)       as u8));
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.write_word(nn, hllow);
        cpu.write_word(nn + 1, hlhigh);

        println!("{:#06x}: LD ({:#06X}), HL", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdAMemNn {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let memval = cpu.read_word(nn);
        cpu.write_reg8(Reg8::A, memval);

        println!("{:#06x}: LD A, ({:#06X})", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }
}

impl Instruction for LdIyNn {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        cpu.set_iy(nn);

        println!("{:#06x}: LD IY, {:#06X}", cpu.get_pc() - 1, nn);
        cpu.inc_pc(3);
    }
}


struct Lddr;

impl Instruction for Lddr {
    fn execute(&self, cpu: &mut Cpu) {
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

        println!("{:#06x}: LDDR", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }
}


struct Ldir;

impl Instruction for Ldir {
    fn execute(&self, cpu: &mut Cpu) {
        let mut counter = cpu.read_reg16(Reg16::BC);
        while counter > 0 {
            let deval = cpu.read_reg16(Reg16::DE);
            let hlval = cpu.read_reg16(Reg16::HL);

            let memval = cpu.read_word(hlval);
            cpu.write_word(deval, memval);

            cpu.write_reg16(Reg16::DE, deval.wrapping_add(1));
            cpu.write_reg16(Reg16::HL, hlval.wrapping_add(1));

            counter -= 1;
            cpu.write_reg16(Reg16::BC, counter);
        }

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(PARITY_OVERFLOW_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        println!("{:#06x}: LDIR", cpu.get_pc() - 1);
        cpu.inc_pc(1);
    }
}


struct OrR { r: Reg8 }
struct OrN ;

impl Instruction for OrR {
    fn execute(&self, cpu: &mut Cpu) {
        let orval = cpu.read_reg8(self.r) | cpu.read_reg8(Reg8::A);
        cpu.write_reg8(Reg8::A, orval);

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        cpu.clear_flag(CARRY_FLAG);
        if orval.count_ones() % 2 == 0 { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }
        if orval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if orval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }

        println!("{:#06x}: OR {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for OrN {
    fn execute(&self, cpu: &mut Cpu) {
        let n = cpu.read_word(cpu.get_pc() + 1);
        let orval = n | cpu.read_reg8(Reg8::A);
        cpu.write_reg8(Reg8::A, orval);

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        cpu.clear_flag(CARRY_FLAG);
        if orval.count_ones() % 2 == 0 { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }
        if orval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if orval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }

        println!("{:#06x}: OR {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}


struct OutPortCR { r: Reg8 }
struct OutPortNA ;

impl Instruction for OutPortCR {
    fn execute(&self, cpu: &mut Cpu) {
        let rval = cpu.read_reg8(self.r);
        let port = Port::from_u16(cpu.read_reg16(Reg16::BC)).unwrap();

        cpu.write_port(port, rval);

        println!("{:#06x}: OUT (C), {:?}", cpu.get_pc() - 1, self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for OutPortNA {
    fn execute(&self, cpu: &mut Cpu) {
        let n = cpu.read_word(cpu.get_pc() + 1);
        let port = Port::from_u8(n).unwrap();
        let accval = cpu.read_reg8(Reg8::A);

        cpu.write_port(port, accval);

        println!("{:#06x}: OUT ({:#04X}), A", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}


struct PopQq { r: Reg16qq }

impl Instruction for PopQq {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16qq(self.r, ((high as u16) << 8 ) | low as u16);

        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        println!("{:#06x}: POP {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}


struct PushQq { r: Reg16qq }

impl Instruction for PushQq {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);
        let rval = cpu.read_reg16qq(self.r);

        cpu.write_word(curr_sp - 1, ((rval & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (rval & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        println!("{:#06x}: PUSH {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}


struct ResBMemIyD { b: u8 }

impl Instruction for ResBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc) as i16;
        let addr = ((cpu.get_iy() as i16) + d) as u16;

        let memval = cpu.read_word(addr);
        cpu.write_word(addr, memval & !(1 << self.b));

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: RES {}, (IY-{:#04X})", curr_pc - 2, self.b, d);
        } else {
            println!("{:#06x}: RES {}, (IY+{:#04X})", curr_pc - 2, self.b, d);
        }
        cpu.inc_pc(2);
    }
}


struct Ret   ;
struct RetCc { cond: FlagCond }

impl Instruction for Ret {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        println!("{:#06x}: RET", cpu.get_pc());
        cpu.set_pc(((high as u16) << 8 ) | low as u16);
    }
}

impl Instruction for RetCc {
    fn execute(&self, cpu: &mut Cpu) {
        let condval = match self.cond {
            FlagCond::NZ => cpu.get_flag(ZERO_FLAG) == false,
            FlagCond::Z  => cpu.get_flag(ZERO_FLAG) == true,
            FlagCond::NC => cpu.get_flag(CARRY_FLAG) == false,
            FlagCond::C  => cpu.get_flag(CARRY_FLAG) == true,
            FlagCond::PO => cpu.get_flag(PARITY_OVERFLOW_FLAG) == false,
            FlagCond::PE => cpu.get_flag(PARITY_OVERFLOW_FLAG) == true,
            FlagCond::P  => cpu.get_flag(SIGN_FLAG) == false,
            FlagCond::M  => cpu.get_flag(SIGN_FLAG) == true
        };

        println!("{:#06x}: RET {:?}", cpu.get_pc(), self.cond);

        if condval {
            let curr_sp = cpu.read_reg16(Reg16::SP);

            let low = cpu.read_word(curr_sp);
            let high = cpu.read_word(curr_sp + 1);

            cpu.write_reg16(Reg16::SP, curr_sp + 2);

            cpu.set_pc(((high as u16) << 8 ) | low as u16);
        } else {
            cpu.inc_pc(1);
        }
    }
}


struct Rrca;

impl Instruction for Rrca {
    fn execute(&self, cpu: &mut Cpu) {
        let aval = cpu.read_reg8(Reg8::A);

        let rrval = aval.wrapping_shr(1);
        cpu.write_reg8(Reg8::A, rrval);

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if rrval & 0b10000000 != 0 { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }

        println!("{:#06x}: RRCA", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


struct Rst { addr: u8 }

impl Instruction for Rst {
    fn execute(&self, cpu: &mut Cpu) {
        let next_pc = cpu.get_pc() + 1;
        let curr_sp = cpu.read_reg16(Reg16::SP);

        cpu.write_word(curr_sp - 1, ((next_pc & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (next_pc & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        println!("{:#06x}: RST {:#04X}", cpu.get_pc(), self.addr);
        cpu.set_pc(self.addr as u16);
    }
}


struct Scf;

impl Instruction for Scf {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_flag(CARRY_FLAG);
        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        println!("{:#06x}: SCF", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


struct SetBMemIyD { b: u8 }

impl Instruction for SetBMemIyD {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc) as i16;
        let addr = ((cpu.get_iy() as i16) + d) as u16;

        let memval = cpu.read_word(addr);
        cpu.write_word(addr, memval | (1 << self.b));

        let mut d = d as i8;
        if d & 0b10000000 != 0 {
            d = (d ^ 0xFF) + 1;
            println!("{:#06x}: SET {}, (IY-{:#04X})", curr_pc - 2, self.b, d);
        } else {
            println!("{:#06x}: SET {}, (IY+{:#04X})", curr_pc - 2, self.b, d);
        }
        cpu.inc_pc(2);
    }
}


struct SubR { r: Reg8 }
struct SubN ;

impl Instruction for SubN {
    fn execute(&self, cpu: &mut Cpu) {
        let aval = cpu.read_reg8(Reg8::A) as i8;
        let n = cpu.read_word(cpu.get_pc() + 1) as i8;

        let subval = aval.wrapping_sub(n);
        cpu.write_reg8(Reg8::A, subval as u8);

        if subval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if subval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if aval & 0x0F < n & 0x0F { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        match (aval   & 0b10000000 != 0,
               n      & 0b10000000 != 0,
               subval & 0b10000000 != 0) {
            (true, false, false) | (false, true, true) => cpu.set_flag(PARITY_OVERFLOW_FLAG),
            _ => cpu.clear_flag(PARITY_OVERFLOW_FLAG)
        };
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if aval < n { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }

        println!("{:#06x}: SUB {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}

impl Instruction for SubR {
    fn execute(&self, cpu: &mut Cpu) {
        let aval = cpu.read_reg8(Reg8::A) as i8;
        let r = cpu.read_reg8(self.r) as i8;

        let subval = aval.wrapping_sub(r);
        cpu.write_reg8(Reg8::A, subval as u8);

        if subval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if subval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if aval & 0x0F < r & 0x0F { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        match (aval   & 0b10000000 != 0,
               r      & 0b10000000 != 0,
               subval & 0b10000000 != 0) {
            (true, false, false) | (false, true, true) => cpu.set_flag(PARITY_OVERFLOW_FLAG),
            _ => cpu.clear_flag(PARITY_OVERFLOW_FLAG)
        };
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if aval < r { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); }

        println!("{:#06x}: SUB {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}


struct XorR     { r: Reg8 }
struct XorN     ;
struct XorMemHl ;

impl Instruction for XorR {
    fn execute(&self, cpu: &mut Cpu) {
        let xorval = cpu.read_reg8(self.r) ^ cpu.read_reg8(Reg8::A);

        cpu.write_reg8(Reg8::A, xorval);

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        cpu.clear_flag(CARRY_FLAG);
        if xorval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if xorval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if xorval.count_ones() % 2 == 0 { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }

        println!("{:#06x}: XOR {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

impl Instruction for XorN {
    fn execute(&self, cpu: &mut Cpu) {
        let n = cpu.read_word(cpu.get_pc() + 1);
        let xorval = cpu.read_reg8(Reg8::A) ^ n;

        cpu.write_reg8(Reg8::A, xorval);

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        cpu.clear_flag(CARRY_FLAG);
        if xorval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if xorval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if xorval.count_ones() % 2 == 0 { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }

        println!("{:#06x}: XOR {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}

impl Instruction for XorMemHl {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);
        let memval = cpu.read_word(hlval);

        let xorval = cpu.read_reg8(Reg8::A) ^ memval;

        cpu.write_reg8(Reg8::A, xorval);

        cpu.clear_flag(HALF_CARRY_FLAG);
        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        cpu.clear_flag(CARRY_FLAG);
        if xorval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if xorval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if xorval.count_ones() % 2 == 0 { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }

        println!("{:#06x}: XOR (HL)", cpu.get_pc());
        cpu.inc_pc(1);
    }
}


pub const INSTR_TABLE_CB: [&'static Instruction; 256] = [
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

pub const INSTR_TABLE_DD: [&'static Instruction; 256] = [
    /* 0x00 */    /* 0x01 */    /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */    /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */    /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */    /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */    /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &LdIxNn     , &LdMemNnIx  , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */    /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &Unsupported, &LdIxMemNn  , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */    /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdMemIxDN  , &Unsupported,

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

    /* 0x40 */    /* 0x41 */             /* 0x42 */    /* 0x43 */               /* 0x44 */    /* 0x45 */    /* 0x46 */    /* 0x47 */
    &Unsupported, &OutPortCR{r:Reg8::B}, &Unsupported, &LdMemNnDd{r:Reg16::BC}, &Unsupported, &Unsupported, &Im{mode:0} , &Unsupported,

    /* 0x48 */    /* 0x49 */             /* 0x4A */    /* 0x4B */               /* 0x4C */    /* 0x4D */    /* 0x4E */    /* 0x4F */
    &Unsupported, &OutPortCR{r:Reg8::C}, &Unsupported, &Unsupported           , &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x50 */    /* 0x51 */             /* 0x52 */    /* 0x53 */               /* 0x54 */    /* 0x55 */    /* 0x56 */    /* 0x57 */
    &Unsupported, &OutPortCR{r:Reg8::D}, &Unsupported, &LdMemNnDd{r:Reg16::DE}, &Unsupported, &Unsupported, &Im{mode:1} , &Unsupported,

    /* 0x58 */    /* 0x59 */             /* 0x5A */    /* 0x5B */               /* 0x5C */    /* 0x5D */    /* 0x5E */    /* 0x5F */
    &Unsupported, &OutPortCR{r:Reg8::E}, &Unsupported, &Unsupported           , &Unsupported, &Unsupported, &Im{mode:2}, &Unsupported,

    /* 0x60 */    /* 0x61 */             /* 0x62 */    /* 0x63 */               /* 0x64 */    /* 0x65 */    /* 0x66 */    /* 0x67 */
    &Unsupported, &OutPortCR{r:Reg8::H}, &Unsupported, &LdMemNnDd{r:Reg16::HL}, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x68 */    /* 0x69 */             /* 0x6A */    /* 0x6B */               /* 0x6C */    /* 0x6D */    /* 0x6E */    /* 0x6F */
    &Unsupported, &OutPortCR{r:Reg8::L}, &Unsupported, &Unsupported           , &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x70 */    /* 0x71 */             /* 0x72 */    /* 0x73 */               /* 0x74 */    /* 0x75 */    /* 0x76 */    /* 0x77 */
    &Unsupported, &Unsupported         , &Unsupported, &LdMemNnDd{r:Reg16::SP}, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x78 */    /* 0x79 */             /* 0x7A */    /* 0x7B */               /* 0x7C */    /* 0x7D */    /* 0x7E */    /* 0x7F */
    &Unsupported, &OutPortCR{r:Reg8::A}, &Unsupported, &Unsupported,            &Unsupported, &Unsupported, &Unsupported, &Unsupported,

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
    /* 0x00 */    /* 0x01 */    /* 0x02 */    /* 0x03 */    /* 0x04 */    /* 0x05 */    /* 0x06 */    /* 0x07 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x08 */    /* 0x09 */    /* 0x0A */    /* 0x0B */    /* 0x0C */    /* 0x0D */    /* 0x0E */    /* 0x0F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x10 */    /* 0x11 */    /* 0x12 */    /* 0x13 */    /* 0x14 */    /* 0x15 */    /* 0x16 */    /* 0x17 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x18 */    /* 0x19 */    /* 0x1A */    /* 0x1B */    /* 0x1C */    /* 0x1D */    /* 0x1E */    /* 0x1F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x20 */    /* 0x21 */    /* 0x22 */    /* 0x23 */    /* 0x24 */    /* 0x25 */    /* 0x26 */    /* 0x27 */
    &Unsupported, &LdIyNn     , &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x28 */    /* 0x29 */    /* 0x2A */    /* 0x2B */    /* 0x2C */    /* 0x2D */    /* 0x2E */    /* 0x2F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

    /* 0x30 */    /* 0x31 */    /* 0x32 */    /* 0x33 */    /* 0x34 */    /* 0x35 */    /* 0x36 */    /* 0x37 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &DecIyD     , &LdMemIyDN,   &Unsupported,

    /* 0x38 */    /* 0x39 */    /* 0x3A */    /* 0x3B */    /* 0x3C */    /* 0x3D */    /* 0x3E */    /* 0x3F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported,

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

    /* 0x70 */    /* 0x71 */    /* 0x72 */    /* 0x73 */    /* 0x74 */    /* 0x75 */    /* 0x76 */             /* 0x77 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported         , &Unsupported,

    /* 0x78 */    /* 0x79 */    /* 0x7A */    /* 0x7B */    /* 0x7C */    /* 0x7D */    /* 0x7E */             /* 0x7F */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &LdRMemIyD{r:Reg8::A}, &Unsupported,

    /* 0x80 */    /* 0x81 */    /* 0x82 */    /* 0x83 */    /* 0x84 */    /* 0x85 */    /* 0x86 */    /* 0x87 */
    &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &Unsupported, &AddAMemIyD , &Unsupported,

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
    &Unsupported, &LdDdNn{r:Reg16::BC} , &Unsupported, &IncSs{r:Reg16::BC}, &IncR{r:Reg8::B}, &DecR{r:Reg8::B}, &LdRN{r:Reg8::B}, &Unsupported,

    /* 0x08 */    /* 0x09 */             /* 0x0A */    /* 0x0B */           /* 0x0C */        /* 0x0D */        /* 0x0E */        /* 0x0F */
    &Unsupported, &AddHlSs{r:Reg16::BC}, &Unsupported, &DecSs{r:Reg16::BC}, &IncR{r:Reg8::C}, &DecR{r:Reg8::C}, &LdRN{r:Reg8::C}, &Rrca       ,

    /* 0x10 */    /* 0x11 */             /* 0x12 */    /* 0x13 */           /* 0x14 */        /* 0x15 */        /* 0x16 */        /* 0x17 */
    &Djnz       , &LdDdNn{r:Reg16::DE} , &Unsupported, &IncSs{r:Reg16::DE}, &IncR{r:Reg8::D}, &DecR{r:Reg8::D}, &LdRN{r:Reg8::D}, &Unsupported,

    /* 0x18 */    /* 0x19 */             /* 0x1A */    /* 0x1B */           /* 0x1C */        /* 0x1D */        /* 0x1E */        /* 0x1F */
    &JrE        , &AddHlSs{r:Reg16::DE}, &Unsupported, &DecSs{r:Reg16::DE}, &IncR{r:Reg8::E}, &DecR{r:Reg8::E}, &LdRN{r:Reg8::E}, &Unsupported,

    /* 0x20 */    /* 0x21 */             /* 0x22 */    /* 0x23 */           /* 0x24 */        /* 0x25 */        /* 0x26 */        /* 0x27 */
    &JrNz       , &LdDdNn{r:Reg16::HL} , &LdMemNnHl  , &IncSs{r:Reg16::HL}, &IncR{r:Reg8::H}, &DecR{r:Reg8::H}, &LdRN{r:Reg8::H}, &Unsupported,

    /* 0x28 */    /* 0x29 */             /* 0x2A */    /* 0x2B */           /* 0x2C */        /* 0x2D */        /* 0x2E */        /* 0x2F */
    &JrZ        , &AddHlSs{r:Reg16::HL}, &LdHlMemNn  , &DecSs{r:Reg16::HL}, &IncR{r:Reg8::L}, &DecR{r:Reg8::L}, &LdRN{r:Reg8::L}, &Unsupported,

    /* 0x30 */    /* 0x31 */             /* 0x32 */    /* 0x33 */           /* 0x34 */        /* 0x35 */        /* 0x36 */        /* 0x37 */
    &Unsupported, &LdDdNn{r:Reg16::SP} , &LdMemNnA   , &IncSs{r:Reg16::SP}, &Unsupported    , &Unsupported    , &LdMemHlN       , &Scf        ,

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
    &AddAR{r:Reg8::B}, &AddAR{r:Reg8::C}, &AddAR{r:Reg8::D}, &AddAR{r:Reg8::E}, &AddAR{r:Reg8::H}, &AddAR{r:Reg8::L}, &Unsupported, &AddAR{r:Reg8::A},

    /* 0x88 */         /* 0x89 */         /* 0x8A */         /* 0x8B */         /* 0x8C */         /* 0x8D */         /* 0x8E */    /* 0x8F */
    &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported, &Unsupported     ,

    /* 0x90 */         /* 0x91 */         /* 0x92 */         /* 0x93 */         /* 0x94 */         /* 0x95 */         /* 0x96 */    /* 0x97 */
    &SubR{r:Reg8::B} , &SubR{r:Reg8::C} , &SubR{r:Reg8::D} , &SubR{r:Reg8::E} , &SubR{r:Reg8::H} , &SubR{r:Reg8::L} , &Unsupported, &SubR{r:Reg8::A} ,

    /* 0x98 */         /* 0x99 */         /* 0x9A */         /* 0x9B */         /* 0x9C */         /* 0x9D */         /* 0x9E */    /* 0x9F */
    &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported     , &Unsupported, &Unsupported     ,

    /* 0xA0 */         /* 0xA1 */         /* 0xA2 */         /* 0xA3 */         /* 0xA4 */         /* 0xA5 */         /* 0xA6 */    /* 0xA7 */
    &AndR{r:Reg8::B} , &AndR{r:Reg8::C} , &AndR{r:Reg8::D} , &AndR{r:Reg8::E} , &AndR{r:Reg8::H} , &AndR{r:Reg8::L} , &Unsupported, &AndR{r:Reg8::A} ,

    /* 0xA8 */         /* 0xA9 */         /* 0xAA */         /* 0xAB */         /* 0xAC */         /* 0xAD */         /* 0xAE */    /* 0xAF */
    &XorR{r:Reg8::B} , &XorR{r:Reg8::C} , &XorR{r:Reg8::D} , &XorR{r:Reg8::E} , &XorR{r:Reg8::H} , &XorR{r:Reg8::L} , &XorMemHl   , &XorR{r:Reg8::A} ,

    /* 0xB0 */         /* 0xB1 */         /* 0xB2 */         /* 0xB3 */         /* 0xB4 */         /* 0xB5 */         /* 0xB6 */    /* 0xB7 */
    &OrR{r:Reg8::B}  , &OrR{r:Reg8::C}  , &OrR{r:Reg8::D}  , &OrR{r:Reg8::E}  , &OrR{r:Reg8::H}  , &OrR{r:Reg8::L}  , &Unsupported, &OrR{r:Reg8::A}  ,

    /* 0xB8 */         /* 0xB9 */         /* 0xBA */         /* 0xBB */         /* 0xBC */         /* 0xBD */         /* 0xBE */    /* 0xBF */
    &CpR{r:Reg8::B}  , &CpR{r:Reg8::C}  , &CpR{r:Reg8::D}  , &CpR{r:Reg8::E}  , &CpR{r:Reg8::H}  , &CpR{r:Reg8::L}  , &CpMemHl    , &CpR{r:Reg8::A}  ,

    /* 0xC0 */                 /* 0xC1 */             /* 0xC2 */    /* 0xC3 */    /* 0xC4 */    /* 0xC5 */              /* 0xC6 */    /* 0xC7 */
    &RetCc{cond:FlagCond::NZ}, &PopQq{r:Reg16qq::BC}, &Unsupported, &JpNn       , &Unsupported, &PushQq{r:Reg16qq::BC}, &AddAN      , &Rst{addr:0x00},

    /* 0xC8 */                 /* 0xC9 */             /* 0xCA */    /* 0xCB */    /* 0xCC */    /* 0xCD */              /* 0xCE */    /* 0xCF */
    &RetCc{cond:FlagCond::Z} , &Ret                 , &Unsupported, &Unsupported, &Unsupported, &CallNn               , &Unsupported, &Rst{addr:0x08},

    /* 0xD0 */                 /* 0xD1 */             /* 0xD2 */    /* 0xD3 */    /* 0xD4 */    /* 0xD5 */              /* 0xD6 */    /* 0xD7 */
    &RetCc{cond:FlagCond::NC}, &PopQq{r:Reg16qq::DE}, &Unsupported, &OutPortNA  , &Unsupported, &PushQq{r:Reg16qq::DE}, &SubN       , &Rst{addr:0x10},

    /* 0xD8 */                 /* 0xD9 */             /* 0xDA */    /* 0xDB */    /* 0xDC */    /* 0xDD */              /* 0xDE */    /* 0xDF */
    &RetCc{cond:FlagCond::C} , &Exx                 , &Unsupported, &Unsupported, &Unsupported, &Unsupported          , &Unsupported, &Rst{addr:0x18},

    /* 0xE0 */                 /* 0xE1 */             /* 0xE2 */    /* 0xE3 */    /* 0xE4 */    /* 0xE5 */              /* 0xE6 */    /* 0xE7 */
    &RetCc{cond:FlagCond::PO}, &PopQq{r:Reg16qq::HL}, &Unsupported, &ExMemSpHl  , &Unsupported, &PushQq{r:Reg16qq::HL}, &AndN       , &Rst{addr:0x20},

    /* 0xE8 */                 /* 0xE9 */             /* 0xEA */    /* 0xEB */    /* 0xEC */    /* 0xED */              /* 0xEE */    /* 0xEF */
    &RetCc{cond:FlagCond::PE}, &JpMemHl             , &Unsupported, &ExDeHl     , &Unsupported, &Unsupported          , &XorN       , &Rst{addr:0x28},

    /* 0xF0 */                 /* 0xF1 */             /* 0xF2 */    /* 0xF3 */    /* 0xF4 */    /* 0xF5 */              /* 0xF6 */    /* 0xF7 */
    &RetCc{cond:FlagCond::P} , &PopQq{r:Reg16qq::AF}, &Unsupported, &Di         , &Unsupported, &PushQq{r:Reg16qq::AF}, &OrN        , &Rst{addr:0x30},

    /* 0xF8 */                 /* 0xF9 */             /* 0xFA */    /* 0xFB */    /* 0xFC */    /* 0xFD */              /* 0xFE */    /* 0xFF */
    &RetCc{cond:FlagCond::M} , &LdSpHl              , &Unsupported, &Ei         , &Unsupported, &Unsupported          , &Unsupported, &Rst{addr:0x38}
];

