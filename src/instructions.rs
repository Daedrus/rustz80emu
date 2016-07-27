use super::cpu::*;
use num::FromPrimitive;

pub trait Instruction {
    fn execute(&self, &mut Cpu);
}

struct Instruction_UNSUPPORTED;

impl Instruction for Instruction_UNSUPPORTED {
    fn execute(&self, cpu: &mut Cpu) {
        println!("{:?}", cpu);

        panic!("Unsupported instruction {:#x}", cpu.read_word(cpu.get_pc()));
    }
}

struct Instruction_DEC_SS {
    regpair: Reg16
}

impl Instruction for Instruction_DEC_SS {
    fn execute(&self, cpu: &mut Cpu) {
        let oldregval = cpu.read_reg16(self.regpair);
        cpu.write_reg16(self.regpair, oldregval - 1);

        println!("{:#06x}: DEC {:?}", cpu.get_pc(), self.regpair);
        cpu.inc_pc(1);
    }
}

struct Instruction_LD_R_N {
    r: Reg8
}

impl Instruction for Instruction_LD_R_N {
    fn execute(&self, cpu: &mut Cpu) {
        let n = cpu.read_word(cpu.get_pc() + 1);
        cpu.write_reg8(self.r, n);

        println!("{:#06x}: LD {:?}, {:#04X}", cpu.get_pc(), self.r, n);
        cpu.inc_pc(2);
    }
}

struct Instruction_LD_DD_NN {
    regpair: Reg16
}

impl Instruction for Instruction_LD_DD_NN {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        cpu.write_reg16(self.regpair, nn);

        println!("{:#06x}: LD {:?}, {:#06X}", cpu.get_pc(), self.regpair, nn);
        cpu.inc_pc(3);
    }
}

struct Instruction_LD_HL_NN;

impl Instruction for Instruction_LD_HL_NN {
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

struct Instruction_LD_HL_N;

impl Instruction for Instruction_LD_HL_N {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);

        let n =  cpu.read_word(cpu.get_pc() + 1);

        cpu.write_word(hlval, n);

        println!("{:#06x}: LD (HL), {:#04X}", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}

struct Instruction_LD_SP_HL;

impl Instruction for Instruction_LD_SP_HL {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);
        cpu.write_reg16(Reg16::SP, hlval);

        println!("{:#06x}: LD SP, HL", cpu.get_pc());
        cpu.inc_pc(1);
    }
}

struct Instruction_LD_IX_NN;

impl Instruction for Instruction_LD_IX_NN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let d = cpu.read_word(curr_pc + 2);
        let n = cpu.read_word(curr_pc + 3);
        let nn = (d as u16) | ((n as u16) << 8);

        match cpu.read_word(curr_pc + 1) {
            0b00100001 => {
                cpu.set_ix(nn);
                println!("{:#06x}: LD IX, {:#06X}", cpu.get_pc(), nn);
            }
            0b00101010 => {
                let nnmemval = (cpu.read_word(nn    ) as u16) |
                              ((cpu.read_word(nn + 1) as u16) << 8);

                cpu.set_ix(nnmemval);

                println!("{:#06x}: LD IX, {:#06X}", cpu.get_pc(), nnmemval);
            },
            0b00100010 => {
                let (ixhigh, ixlow) = (((cpu.get_ix() & 0xFF00) >> 8) as u8, 
                                       ((cpu.get_ix() & 0x00FF)       as u8));

                cpu.write_word(nn, ixlow);
                cpu.write_word(nn + 1, ixhigh);

                println!("{:#06x}: LD ({:#06X}), IX", cpu.get_pc(), nn);
            },
            0b00110110 => {
                let addr = cpu.get_ix() as i16 + d as i16;
                cpu.write_word(addr as u16, n);

                println!("{:#06x}: LD (IX+{:#04X}), {:#04X}", cpu.get_pc(), d, n);
            }
            _ => unreachable!()
        }

        cpu.inc_pc(4);
    }
}

struct Instruction_LD_R_R {
    rt: Reg8,
    rs: Reg8
}

impl Instruction for Instruction_LD_R_R {
    fn execute(&self, cpu: &mut Cpu) {
        let rsval = cpu.read_reg8(self.rs);
        cpu.write_reg8(self.rt, rsval);
        println!("{:#06x}: LD {:?}, {:?}", cpu.get_pc(), self.rt, self.rs);
        cpu.inc_pc(1);
    }
}

struct Instruction_OR_R {
    r: Reg8
}

impl Instruction for Instruction_OR_R {
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

struct Instruction_DI;

impl Instruction for Instruction_DI {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.clear_iff1();
        cpu.clear_iff2();

        println!("{:#06x}: DI", cpu.get_pc());
        cpu.inc_pc(1);
    }
}

struct Instruction_EI;

impl Instruction for Instruction_EI {
    fn execute(&self, cpu: &mut Cpu) {
        cpu.set_iff1();
        cpu.set_iff2();

        println!("{:#06x}: EI", cpu.get_pc());
        cpu.inc_pc(1);
    }
}

struct Instruction_JR_NZ;

impl Instruction for Instruction_JR_NZ {
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

struct Instruction_JP_NN;

impl Instruction for Instruction_JP_NN {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);

        println!("{:#06x}: JP {:#06X}", cpu.get_pc(), nn);
        cpu.set_pc(nn);
    }
}

struct Instruction_EX_SP_HL;

impl Instruction for Instruction_EX_SP_HL {
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

struct Instruction_EX_DE_HL;

impl Instruction for Instruction_EX_DE_HL {
    fn execute(&self, cpu: &mut Cpu) {
        let deval = cpu.read_reg16(Reg16::DE);
        let hlval = cpu.read_reg16(Reg16::HL);

        cpu.write_reg16(Reg16::DE, hlval);
        cpu.write_reg16(Reg16::HL, deval);

        println!("{:#06x}: EX DE, HL", cpu.get_pc());
        cpu.inc_pc(1);
    }
}

struct Instruction_EXX;

impl Instruction for Instruction_EXX {
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

struct Instruction_DEC_R {
    r: Reg8
}

impl Instruction for Instruction_DEC_R {
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

struct Instruction_INC_R {
    r: Reg8
}

impl Instruction for Instruction_INC_R {
    fn execute(&self, cpu: &mut Cpu) {
        let incval = cpu.read_reg8(self.r) + 1;
        cpu.write_reg8(self.r, incval);

        if incval & 0b10000000 != 0 { cpu.set_flag(SIGN_FLAG); } else { cpu.clear_flag(SIGN_FLAG); }
        if incval == 0 { cpu.set_flag(ZERO_FLAG); } else { cpu.clear_flag(ZERO_FLAG); }
        if incval & 0b00001111 == 0 { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }
        if (incval - 1) == 0x7F { cpu.set_flag(PARITY_OVERFLOW_FLAG); } else { cpu.clear_flag(PARITY_OVERFLOW_FLAG); }
        cpu.clear_flag(ADD_SUBTRACT_FLAG);

        println!("{:#06x}: INC {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

struct Instruction_INC_SS {
    regpair: Reg16
}

impl Instruction for Instruction_INC_SS {
    fn execute(&self, cpu: &mut Cpu) {
        let incval = cpu.read_reg16(self.regpair) + 1;
        cpu.write_reg16(self.regpair, incval);

        println!("{:#06x}: INC {:?}", cpu.get_pc(), self.regpair);
        cpu.inc_pc(1);
    }
}

struct Instruction_OUT_N_A;

impl Instruction for Instruction_OUT_N_A {
    fn execute(&self, cpu: &mut Cpu) {
        let n = cpu.read_word(cpu.get_pc() + 1);
        let port = Port::from_u8(n).unwrap();
        let accval = cpu.read_reg8(Reg8::A);

        cpu.write_port(port, accval);

        println!("{:#06x}: OUT ({:#04X}), A", cpu.get_pc(), n);
        cpu.inc_pc(2);
    }
}

struct Instruction_OUT_C_R;

impl Instruction for Instruction_OUT_C_R {
    fn execute(&self, cpu: &mut Cpu) {
        match cpu.read_word(cpu.get_pc() + 1) {
            0b10110000 => {
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

                println!("{:#06x}: LDIR", cpu.get_pc());
                cpu.inc_pc(2);
            },
            0b10111000 => {
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

                println!("{:#06x}: LDDR", cpu.get_pc());
                cpu.inc_pc(2);
            },
            0b01000110 => {
                cpu.set_im(0);
                println!("{:#06x}: IM 0", cpu.get_pc());
                cpu.inc_pc(2);
            },
            0b01010110 => {
                cpu.set_im(1);
                println!("{:#06x}: IM 1", cpu.get_pc());
                cpu.inc_pc(2);
            },
            0b01011110 => {
                cpu.set_im(2);
                println!("{:#06x}: IM 2", cpu.get_pc());
                cpu.inc_pc(2);
            },
            opcode if opcode & 0b11000111 == 0b01000001 => {
                let r = Reg8::from_u8((opcode & 0b00111000) >> 3).unwrap();
                let rval = cpu.read_reg8(r);
                let port = Port::from_u16(cpu.read_reg16(Reg16::BC)).unwrap();

                cpu.write_port(port, rval);

                println!("{:#06x}: OUT (C), {:?}", cpu.get_pc(), r);
                cpu.inc_pc(2);
            },
            opcode if opcode & 0b11001111 == 0b01000011 => {
                let regpair = Reg16::from_u8((opcode & 0b00110000) >> 4).unwrap();
                let rval = cpu.read_reg16(regpair);
                let (rhigh, rlow) = (((rval & 0xFF00) >> 8) as u8,
                                     ((rval & 0x00FF)       as u8));

                let nn =  (cpu.read_word(cpu.get_pc() + 2) as u16) |
                         ((cpu.read_word(cpu.get_pc() + 3) as u16) << 8);

                cpu.write_word(nn, rlow);
                cpu.write_word(nn + 1, rhigh);

                println!("{:#06x}: LD ({:#06X}), {:?}", cpu.get_pc(), nn, regpair);
                cpu.inc_pc(4);
            },
            _ => unreachable!()
        }
    }
}

struct Instruction_LD_HL_R {
    r: Reg8
}

impl Instruction for Instruction_LD_HL_R {
    fn execute(&self, cpu: &mut Cpu) {
        let val = cpu.read_reg8(self.r);
        let addr = cpu.read_reg16(Reg16::HL);

        cpu.write_word(addr, val);

        println!("{:#06x}: LD (HL), {:?}", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

struct Instruction_CP_HL;

impl Instruction for Instruction_CP_HL {
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

struct Instruction_XOR_R {
    r: Reg8
}

impl Instruction for Instruction_XOR_R {
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

struct Instruction_XOR_N;

impl Instruction for Instruction_XOR_N {
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

struct Instruction_DJNZ;

impl Instruction for Instruction_DJNZ {
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

struct Instruction_LD_NN_A;

impl Instruction for Instruction_LD_NN_A {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let aval = cpu.read_reg8(Reg8::A);

        cpu.write_word(nn, aval);

        println!("{:#06x}: LD ({:#06X}), A", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }
}

struct Instruction_JR_E;

impl Instruction for Instruction_JR_E {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        let offset = cpu.read_word(curr_pc + 1) as i8 + 2;
        let target = (curr_pc as i16 + offset as i16) as u16;

        println!("{:#06x}: JR {:#06X}", cpu.get_pc(), target);
        cpu.set_pc(target);
    }
}

struct Instruction_RST {
    addr: u8
}

impl Instruction for Instruction_RST {
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

struct Instruction_CALL_NN;

impl Instruction for Instruction_CALL_NN {
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

struct Instruction_RET;

impl Instruction for Instruction_RET {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        println!("{:#06x}: RET", cpu.get_pc());

        cpu.set_pc(((high as u16) << 8 ) | low as u16);
    }
}

struct Instruction_PUSH_QQ {
    regpair: Reg16qq
}

impl Instruction for Instruction_PUSH_QQ {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);
        let regpairval = cpu.read_reg16qq(self.regpair);

        cpu.write_word(curr_sp - 1, ((regpairval & 0xFF00) >> 8) as u8);
        cpu.write_word(curr_sp - 2,  (regpairval & 0x00FF)       as u8);

        cpu.write_reg16(Reg16::SP, curr_sp - 2);

        println!("{:#06x}: PUSH {:?}", cpu.get_pc(), self.regpair);
        cpu.inc_pc(1);
    }
}

struct Instruction_POP_QQ {
    regpair: Reg16qq
}

impl Instruction for Instruction_POP_QQ {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_sp = cpu.read_reg16(Reg16::SP);

        let low = cpu.read_word(curr_sp);
        let high = cpu.read_word(curr_sp + 1);

        cpu.write_reg16qq(self.regpair, ((high as u16) << 8 ) | low as u16);

        cpu.write_reg16(Reg16::SP, curr_sp + 2);

        println!("{:#06x}: POP {:?}", cpu.get_pc(), self.regpair);
        cpu.inc_pc(1);
    }
}

struct Instruction_ADD_HL_SS {
    regpair: Reg16
}

impl Instruction for Instruction_ADD_HL_SS {
    fn execute(&self, cpu: &mut Cpu) {
        let hlval = cpu.read_reg16(Reg16::HL);
        let regpairval = cpu.read_reg16(self.regpair);

        let sum = hlval as u32 + regpairval as u32;

        cpu.write_reg16(Reg16::HL, sum as u16);

        cpu.clear_flag(ADD_SUBTRACT_FLAG);
        if sum & 0x10000 != 0 { cpu.set_flag(CARRY_FLAG); } else { cpu.clear_flag(CARRY_FLAG); };
        if ((hlval & 0xfff) + (regpairval & 0xfff)) & 0x1000 != 0 { cpu.set_flag(HALF_CARRY_FLAG); } else { cpu.clear_flag(HALF_CARRY_FLAG); }

        println!("{:#06x}: ADD HL, {:?}", cpu.get_pc(), self.regpair);
        cpu.inc_pc(1);
    }
}

struct Instruction_LD_R_HL {
    r: Reg8
}

impl Instruction for Instruction_LD_R_HL {
    fn execute(&self, cpu: &mut Cpu) {
        let hlmemval = cpu.read_word(cpu.read_reg16(Reg16::HL));
        cpu.write_reg8(self.r, hlmemval);

        println!("{:#06x}: LD {:?}, (HL)", cpu.get_pc(), self.r);
        cpu.inc_pc(1);
    }
}

struct Instruction_LD_NN_HL;

impl Instruction for Instruction_LD_NN_HL {
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

struct Instruction_LD_A_NN;

impl Instruction for Instruction_LD_A_NN {
    fn execute(&self, cpu: &mut Cpu) {
        let nn =  (cpu.read_word(cpu.get_pc() + 1) as u16) |
                 ((cpu.read_word(cpu.get_pc() + 2) as u16) << 8);
        let memval = cpu.read_word(nn);
        cpu.write_reg8(Reg8::A, memval);

        println!("{:#06x}: LD A, ({:#06X})", cpu.get_pc(), nn);
        cpu.inc_pc(3);
    }
}

struct Instruction_LD_IY_NN;

impl Instruction for Instruction_LD_IY_NN {
    fn execute(&self, cpu: &mut Cpu) {
        let curr_pc = cpu.get_pc();
        match cpu.read_word(curr_pc + 1) {
            0b00100001 => {
                let nn =  (cpu.read_word(curr_pc + 2) as u16) |
                         ((cpu.read_word(curr_pc + 3) as u16) << 8);
                cpu.set_iy(nn);

                println!("{:#06x}: LD IY, {:#06X}", curr_pc, nn);
                cpu.inc_pc(4);
            },
            0b11001011 => {
                let d = cpu.read_word(curr_pc + 2) as i16;
                let addr = ((cpu.get_iy() as i16) + d) as u16;

                let b = (cpu.read_word(curr_pc + 3) & 0b00111000) >> 3;

                let memval = cpu.read_word(addr);
                cpu.write_word(addr, memval | (1 << b));

                println!("{:#06x}: SET {}, (IY+{:#04X})", curr_pc, b, d);
                cpu.inc_pc(4);
            },
            _ => unreachable!()
        }
    }
}

struct Instruction_AND_N;

impl Instruction for Instruction_AND_N {
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


pub const INSTR_TABLE: [&'static Instruction; 256] = [
    &Instruction_UNSUPPORTED, /* 0b00000000 */
    &Instruction_LD_DD_NN {   /* 0b00000001 */
        regpair: Reg16::BC
    },
    &Instruction_UNSUPPORTED, /* 0b00000010 */
    &Instruction_INC_SS {     /* 0b00000011 */
        regpair: Reg16::BC
    },
    &Instruction_INC_R {      /* 0b00000100 */
        r: Reg8::B
    },
    &Instruction_DEC_R {      /* 0b00000101 */
        r: Reg8::B
    },
    &Instruction_LD_R_N {     /* 0b00000110 */
        r: Reg8::B
    },
    &Instruction_UNSUPPORTED, /* 0b00000111 */
    &Instruction_UNSUPPORTED, /* 0b00001000 */
    &Instruction_ADD_HL_SS {  /* 0b00001001 */
        regpair: Reg16::BC
    },
    &Instruction_UNSUPPORTED, /* 0b00001010 */
    &Instruction_DEC_SS {     /* 0b00001011 */
        regpair: Reg16::BC
    },
    &Instruction_INC_R {      /* 0b00001100 */
        r: Reg8::C
    },
    &Instruction_DEC_R {      /* 0b00001101 */
        r: Reg8::C
    },
    &Instruction_LD_R_N {     /* 0b00001110 */
        r: Reg8::C
    },
    &Instruction_UNSUPPORTED, /* 0b00001111 */
    &Instruction_DJNZ       , /* 0b00010000 */
    &Instruction_LD_DD_NN {   /* 0b00010001 */
        regpair: Reg16::DE
    },
    &Instruction_UNSUPPORTED, /* 0b00010010 */
    &Instruction_INC_SS {     /* 0b00010011 */
        regpair: Reg16::DE
    },
    &Instruction_INC_R {      /* 0b00010100 */
        r: Reg8::D
    },
    &Instruction_DEC_R {      /* 0b00010101 */
        r: Reg8::D
    },
    &Instruction_LD_R_N {     /* 0b00010110 */
        r: Reg8::D
    },
    &Instruction_UNSUPPORTED, /* 0b00010111 */
    &Instruction_JR_E       , /* 0b00011000 */
    &Instruction_ADD_HL_SS {  /* 0b00011001 */
        regpair: Reg16::DE
    },
    &Instruction_UNSUPPORTED, /* 0b00011010 */
    &Instruction_DEC_SS {     /* 0b00011011 */
        regpair: Reg16::DE
    },
    &Instruction_INC_R {      /* 0b00011100 */
        r: Reg8::E
    },
    &Instruction_DEC_R {      /* 0b00011101 */
        r: Reg8::E
    },
    &Instruction_LD_R_N {     /* 0b00011110 */
        r: Reg8::E
    },
    &Instruction_UNSUPPORTED, /* 0b00011111 */
    &Instruction_JR_NZ      , /* 0b00100000 */
    &Instruction_LD_DD_NN {   /* 0b00100001 */
        regpair: Reg16::HL
    },
    &Instruction_LD_NN_HL   , /* 0b00100010 */
    &Instruction_INC_SS {     /* 0b00100011 */
        regpair: Reg16::HL
    },
    &Instruction_INC_R {      /* 0b00100100 */
        r: Reg8::H
    },
    &Instruction_DEC_R {      /* 0b00100101 */
        r: Reg8::H
    },
    &Instruction_LD_R_N {     /* 0b00100110 */
        r: Reg8::H
    },
    &Instruction_UNSUPPORTED, /* 0b00100111 */
    &Instruction_UNSUPPORTED, /* 0b00101000 */
    &Instruction_ADD_HL_SS {  /* 0b00101001 */
        regpair: Reg16::HL
    },
    &Instruction_LD_HL_NN   , /* 0b00101010 */
    &Instruction_DEC_SS {     /* 0b00101011 */
        regpair: Reg16::HL
    },
    &Instruction_INC_R {      /* 0b00101100 */
        r: Reg8::L
    },
    &Instruction_DEC_R {      /* 0b00101101 */
        r: Reg8::L
    },
    &Instruction_LD_R_N {     /* 0b00101110 */
        r: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b00101111 */
    &Instruction_UNSUPPORTED, /* 0b00110000 */
    &Instruction_LD_DD_NN {   /* 0b00110001 */
        regpair: Reg16::SP
    },
    &Instruction_LD_NN_A    , /* 0b00110010 */
    &Instruction_INC_SS {     /* 0b00110011 */
        regpair: Reg16::SP
    },
    &Instruction_UNSUPPORTED, /* 0b00110100 */
    &Instruction_UNSUPPORTED, /* 0b00110101 */
    &Instruction_LD_HL_N    , /* 0b00110110 */
    &Instruction_UNSUPPORTED, /* 0b00110111 */
    &Instruction_UNSUPPORTED, /* 0b00111000 */
    &Instruction_ADD_HL_SS {  /* 0b00111001 */
        regpair: Reg16::SP
    },
    &Instruction_LD_A_NN    , /* 0b00111010 */
    &Instruction_DEC_SS {     /* 0b00111011 */
        regpair: Reg16::SP
    },
    &Instruction_INC_R {      /* 0b00111100 */
        r: Reg8::A
    },
    &Instruction_DEC_R {      /* 0b00111101 */
        r: Reg8::A
    },
    &Instruction_LD_R_N {     /* 0b00111110 */
        r: Reg8::A
    },
    &Instruction_UNSUPPORTED, /* 0b00111111 */
    &Instruction_LD_R_R {     /* 0b01000000 *//*TODO: Valid?*/
        rt: Reg8::B,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01000001 */
        rt: Reg8::B,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01000010 */
        rt: Reg8::B,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01000011 */
        rt: Reg8::B,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01000100 */
        rt: Reg8::B,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01000101 */
        rt: Reg8::B,
        rs: Reg8::L
    },
    &Instruction_LD_R_HL {    /* 0b01000110 */
        r: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01000111 */
        rt: Reg8::B,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01001000 */
        rt: Reg8::C,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01001001 *//*TODO: Valid?*/
        rt: Reg8::C,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01001010 */
        rt: Reg8::C,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01001011 */
        rt: Reg8::C,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01001100 */
        rt: Reg8::C,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01001101 */
        rt: Reg8::C,
        rs: Reg8::L
    },
    &Instruction_LD_R_HL {    /* 0b01001110 */
        r: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01001111 */
        rt: Reg8::C,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01010000 */
        rt: Reg8::D,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01010001 */
        rt: Reg8::D,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01010010 *//*TODO: Valid?*/
        rt: Reg8::D,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01010011 */
        rt: Reg8::D,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01010100 */
        rt: Reg8::D,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01010101 */
        rt: Reg8::D,
        rs: Reg8::L
    },
    &Instruction_LD_R_HL {    /* 0b01010110 */
        r: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01010111 */
        rt: Reg8::D,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01011000 */
        rt: Reg8::E,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01011001 */
        rt: Reg8::E,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01011010 */
        rt: Reg8::E,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01011011 *//*TODO: Valid?*/
        rt: Reg8::E,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01011100 */
        rt: Reg8::E,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01011101 */
        rt: Reg8::E,
        rs: Reg8::L
    },
    &Instruction_LD_R_HL {    /* 0b01011110 */
        r: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01011111 */
        rt: Reg8::E,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01100000 */
        rt: Reg8::H,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01100001 */
        rt: Reg8::H,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01100010 */
        rt: Reg8::H,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01100011 */
        rt: Reg8::H,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01100100 *//*TODO: Valid?*/
        rt: Reg8::H,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01100101 */
        rt: Reg8::H,
        rs: Reg8::L
    },
    &Instruction_LD_R_HL {    /* 0b01100110 */
        r: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01100111 */
        rt: Reg8::H,
        rs: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01101000 */
        rt: Reg8::L,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01101001 */
        rt: Reg8::L,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01101010 */
        rt: Reg8::L,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01101011 */
        rt: Reg8::L,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01101100 */
        rt: Reg8::L,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01101101 *//*TODO: Valid?*/
        rt: Reg8::L,
        rs: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01101110 */
    &Instruction_LD_R_R {     /* 0b01101111 */
        rt: Reg8::L,
        rs: Reg8::A
    },
    &Instruction_LD_HL_R {    /* 0b01110000 */
        r: Reg8::B
    },
    &Instruction_LD_HL_R {    /* 0b01110001 */
        r: Reg8::C
    },
    &Instruction_LD_HL_R {    /* 0b01110010 */
        r: Reg8::D
    },
    &Instruction_LD_HL_R {    /* 0b01110011 */
        r: Reg8::E
    },
    &Instruction_LD_HL_R {    /* 0b01110100 */
        r: Reg8::H
    },
    &Instruction_LD_HL_R {    /* 0b01110101 */
        r: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b01110110 */
    &Instruction_LD_HL_R {    /* 0b01110111 */
        r: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01111000 */
        rt: Reg8::A,
        rs: Reg8::B
    },
    &Instruction_LD_R_R {     /* 0b01111001 */
        rt: Reg8::A,
        rs: Reg8::C
    },
    &Instruction_LD_R_R {     /* 0b01111010 */
        rt: Reg8::A,
        rs: Reg8::D
    },
    &Instruction_LD_R_R {     /* 0b01111011 */
        rt: Reg8::A,
        rs: Reg8::E
    },
    &Instruction_LD_R_R {     /* 0b01111100 */
        rt: Reg8::A,
        rs: Reg8::H
    },
    &Instruction_LD_R_R {     /* 0b01111101 */
        rt: Reg8::A,
        rs: Reg8::L
    },
    &Instruction_LD_R_HL {    /* 0b01111110 */
        r: Reg8::A
    },
    &Instruction_LD_R_R {     /* 0b01111111 *//*TODO: Valid?*/
        rt: Reg8::A,
        rs: Reg8::A
    },
    &Instruction_UNSUPPORTED, /* 0b10000000 */
    &Instruction_UNSUPPORTED, /* 0b10000001 */
    &Instruction_UNSUPPORTED, /* 0b10000010 */
    &Instruction_UNSUPPORTED, /* 0b10000011 */
    &Instruction_UNSUPPORTED, /* 0b10000100 */
    &Instruction_UNSUPPORTED, /* 0b10000101 */
    &Instruction_UNSUPPORTED, /* 0b10000110 */
    &Instruction_UNSUPPORTED, /* 0b10000111 */
    &Instruction_UNSUPPORTED, /* 0b10001000 */
    &Instruction_UNSUPPORTED, /* 0b10001001 */
    &Instruction_UNSUPPORTED, /* 0b10001010 */
    &Instruction_UNSUPPORTED, /* 0b10001011 */
    &Instruction_UNSUPPORTED, /* 0b10001100 */
    &Instruction_UNSUPPORTED, /* 0b10001101 */
    &Instruction_UNSUPPORTED, /* 0b10001110 */
    &Instruction_UNSUPPORTED, /* 0b10001111 */
    &Instruction_UNSUPPORTED, /* 0b10010000 */
    &Instruction_UNSUPPORTED, /* 0b10010001 */
    &Instruction_UNSUPPORTED, /* 0b10010010 */
    &Instruction_UNSUPPORTED, /* 0b10010011 */
    &Instruction_UNSUPPORTED, /* 0b10010100 */
    &Instruction_UNSUPPORTED, /* 0b10010101 */
    &Instruction_UNSUPPORTED, /* 0b10010110 */
    &Instruction_UNSUPPORTED, /* 0b10010111 */
    &Instruction_UNSUPPORTED, /* 0b10011000 */
    &Instruction_UNSUPPORTED, /* 0b10011001 */
    &Instruction_UNSUPPORTED, /* 0b10011010 */
    &Instruction_UNSUPPORTED, /* 0b10011011 */
    &Instruction_UNSUPPORTED, /* 0b10011100 */
    &Instruction_UNSUPPORTED, /* 0b10011101 */
    &Instruction_UNSUPPORTED, /* 0b10011110 */
    &Instruction_UNSUPPORTED, /* 0b10011111 */
    &Instruction_UNSUPPORTED, /* 0b10100000 */
    &Instruction_UNSUPPORTED, /* 0b10100001 */
    &Instruction_UNSUPPORTED, /* 0b10100010 */
    &Instruction_UNSUPPORTED, /* 0b10100011 */
    &Instruction_UNSUPPORTED, /* 0b10100100 */
    &Instruction_UNSUPPORTED, /* 0b10100101 */
    &Instruction_UNSUPPORTED, /* 0b10100110 */
    &Instruction_UNSUPPORTED, /* 0b10100111 */
    &Instruction_XOR_R {      /* 0b10101000 */
        r: Reg8::B
    },
    &Instruction_XOR_R {      /* 0b10101001 */
        r: Reg8::C
    },
    &Instruction_XOR_R {      /* 0b10101010 */
        r: Reg8::D
    },
    &Instruction_XOR_R {      /* 0b10101011 */
        r: Reg8::E
    },
    &Instruction_XOR_R {      /* 0b10101100 */
        r: Reg8::H
    },
    &Instruction_XOR_R {      /* 0b10101101 */
        r: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b10101110 */
    &Instruction_XOR_R {      /* 0b10101111 */
        r: Reg8::A
    },
    &Instruction_OR_R {       /* 0b10110000 */
        r: Reg8::B
    },
    &Instruction_OR_R {       /* 0b10110001 */
        r: Reg8::C
    },
    &Instruction_OR_R {       /* 0b10110010 */
        r: Reg8::D
    },
    &Instruction_OR_R {       /* 0b10110011 */
        r: Reg8::E
    },
    &Instruction_OR_R {       /* 0b10110100 */
        r: Reg8::H
    },
    &Instruction_OR_R {       /* 0b10110101 */
        r: Reg8::L
    },
    &Instruction_UNSUPPORTED, /* 0b10110110 */
    &Instruction_OR_R {       /* 0b10110111 */
        r: Reg8::A
    },
    &Instruction_UNSUPPORTED, /* 0b10111000 */
    &Instruction_UNSUPPORTED, /* 0b10111001 */
    &Instruction_UNSUPPORTED, /* 0b10111010 */
    &Instruction_UNSUPPORTED, /* 0b10111011 */
    &Instruction_UNSUPPORTED, /* 0b10111100 */
    &Instruction_UNSUPPORTED, /* 0b10111101 */
    &Instruction_CP_HL      , /* 0b10111110 */
    &Instruction_UNSUPPORTED, /* 0b10111111 */
    &Instruction_UNSUPPORTED, /* 0b11000000 */
    &Instruction_POP_QQ {     /* 0b11000001 */
        regpair: Reg16qq::BC
    },
    &Instruction_UNSUPPORTED, /* 0b11000010 */
    &Instruction_JP_NN      , /* 0b11000011 */
    &Instruction_UNSUPPORTED, /* 0b11000100 */
    &Instruction_PUSH_QQ {    /* 0b11000101 */
        regpair: Reg16qq::BC
    },
    &Instruction_UNSUPPORTED, /* 0b11000110 */
    &Instruction_RST {        /* 0b11000111 */
        addr: 0x00
    },
    &Instruction_UNSUPPORTED, /* 0b11001000 */
    &Instruction_RET        , /* 0b11001001 */
    &Instruction_UNSUPPORTED, /* 0b11001010 */
    &Instruction_UNSUPPORTED, /* 0b11001011 */
    &Instruction_UNSUPPORTED, /* 0b11001100 */
    &Instruction_CALL_NN    , /* 0b11001101 */
    &Instruction_UNSUPPORTED, /* 0b11001110 */
    &Instruction_RST {        /* 0b11001111 */
        addr: 0x08
    },
    &Instruction_UNSUPPORTED, /* 0b11010000 */
    &Instruction_POP_QQ {     /* 0b11010001 */
        regpair: Reg16qq::DE
    },
    &Instruction_UNSUPPORTED, /* 0b11010010 */
    &Instruction_OUT_N_A    , /* 0b11010011 */
    &Instruction_UNSUPPORTED, /* 0b11010100 */
    &Instruction_PUSH_QQ {    /* 0b11010101 */
        regpair: Reg16qq::DE
    },
    &Instruction_UNSUPPORTED, /* 0b11010110 */
    &Instruction_RST {        /* 0b11010111 */
        addr: 0x10
    },
    &Instruction_UNSUPPORTED, /* 0b11011000 */
    &Instruction_EXX        , /* 0b11011001 */
    &Instruction_UNSUPPORTED, /* 0b11011010 */
    &Instruction_UNSUPPORTED, /* 0b11011011 */
    &Instruction_UNSUPPORTED, /* 0b11011100 */
    &Instruction_LD_IX_NN   , /* 0b11011101 */
    &Instruction_UNSUPPORTED, /* 0b11011110 */
    &Instruction_RST {        /* 0b11011111 */
        addr: 0x18
    },
    &Instruction_UNSUPPORTED, /* 0b11100000 */
    &Instruction_POP_QQ {     /* 0b11100001 */
        regpair: Reg16qq::HL
    },
    &Instruction_UNSUPPORTED, /* 0b11100010 */
    &Instruction_EX_SP_HL   , /* 0b11100011 */
    &Instruction_UNSUPPORTED, /* 0b11100100 */
    &Instruction_PUSH_QQ {    /* 0b11100101 */
        regpair: Reg16qq::HL
    },
    &Instruction_AND_N      , /* 0b11100110 */
    &Instruction_RST {        /* 0b11100111 */
        addr: 0x20
    },
    &Instruction_UNSUPPORTED, /* 0b11101000 */
    &Instruction_UNSUPPORTED, /* 0b11101001 */
    &Instruction_UNSUPPORTED, /* 0b11101010 */
    &Instruction_EX_DE_HL   , /* 0b11101011 */
    &Instruction_UNSUPPORTED, /* 0b11101100 */
    &Instruction_OUT_C_R    , /* 0b11101101 */
    &Instruction_XOR_N      , /* 0b11101110 */
    &Instruction_RST {        /* 0b11101111 */
        addr: 0x28
    },
    &Instruction_UNSUPPORTED, /* 0b11110000 */
    &Instruction_POP_QQ {     /* 0b11110001 */
        regpair: Reg16qq::AF
    },
    &Instruction_UNSUPPORTED, /* 0b11110010 */
    &Instruction_DI,          /* 0b11110011 */
    &Instruction_UNSUPPORTED, /* 0b11110100 */
    &Instruction_PUSH_QQ {    /* 0b11110101 */
        regpair: Reg16qq::AF
    },
    &Instruction_UNSUPPORTED, /* 0b11110110 */
    &Instruction_RST {        /* 0b11110111 */
        addr: 0x30
    },
    &Instruction_UNSUPPORTED, /* 0b11111000 */
    &Instruction_LD_SP_HL   , /* 0b11111001 */
    &Instruction_UNSUPPORTED, /* 0b11111010 */
    &Instruction_EI         , /* 0b11111011 */
    &Instruction_UNSUPPORTED, /* 0b11111100 */
    &Instruction_LD_IY_NN   , /* 0b11111101 */
    &Instruction_UNSUPPORTED, /* 0b11111110 */
    &Instruction_RST {        /* 0b11111111 */
        addr: 0x38
    }
];

