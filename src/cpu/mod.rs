mod cpu;
mod instructions;
mod instructions_fdcb;
mod instructions_ddcb;
mod instructions_cb;
mod instructions_ed;
mod instructions_dd;
mod instructions_fd;

pub use cpu::cpu::*;
pub use cpu::instructions::{Instruction, INSTR_TABLE};
pub use cpu::instructions_fdcb::INSTR_TABLE_FDCB;
pub use cpu::instructions_ddcb::INSTR_TABLE_DDCB;
pub use cpu::instructions_cb::INSTR_TABLE_CB;
pub use cpu::instructions_ed::INSTR_TABLE_ED;
pub use cpu::instructions_dd::INSTR_TABLE_DD;
pub use cpu::instructions_fd::INSTR_TABLE_FD;
