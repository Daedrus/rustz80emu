use super::peripherals::*;

use std::rc::Rc;
use std::cell::RefCell;

macro_rules! println_if_trace {
    ($fmt: expr, $( $t: expr ),*) => {{
        #[cfg(feature = "trace-interconnect")]
        println!($fmt, $($t),* );
    }}
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct Interconnect {
    memory: Rc<RefCell<Memory>>,

    ay: Rc<RefCell<Ay>>,

    ula: Rc<RefCell<Ula>>,

    ula_contention: Vec<u8>,
    ula_contention_no_mreq: Vec<u8>,
}

impl Interconnect {
    pub fn new(memory: Rc<RefCell<Memory>>,
               ay : Rc<RefCell<Ay>>,
               ula : Rc<RefCell<Ula>>) -> Self {

        let ula_contention = include_bytes!("peripherals/ulacontention.bin");
        let ula_contention_no_mreq = include_bytes!("peripherals/ulacontentionnomreq.bin");

        Interconnect {
            memory,
            ay,
            ula,
            ula_contention: ula_contention.to_vec(),
            ula_contention_no_mreq: ula_contention_no_mreq.to_vec(),
        }
    }

    fn is_addr_contended(&self, addr: u16) -> bool {
        (addr >= 0x4000 && addr < 0x8000) ||
        (addr >= 0xC000 && (self.memory.borrow().get_c000_bank() % 2 != 0))
    }

    #[inline(always)]
    pub fn contend_read(&self, addr: u16, curr_tcycle: u32, tcycles: u32) -> u32 {
        println_if_trace!("{: >5} MC {:04x}", curr_tcycle, addr);
        let delay = if self.is_addr_contended(addr) {
            self.ula_contention[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + tcycles
    }

    #[inline(always)]
    pub fn contend_read_no_mreq(&self, addr: u16, curr_tcycle: u32) -> u32 {
        println_if_trace!("{: >5} MC {:04x}", curr_tcycle, addr);
        let delay = if self.is_addr_contended(addr) {
            self.ula_contention_no_mreq[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + 1
    }

    #[inline(always)]
    pub fn contend_write_no_mreq(&self, addr: u16, curr_tcycle: u32) -> u32 {
        println_if_trace!("{: >5} MC {:04x}", curr_tcycle, addr);
        let delay = if self.is_addr_contended(addr) {
            self.ula_contention_no_mreq[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + 1
    }

    #[cfg_attr(not(feature = "trace-interconnect"), allow(unused_variables))]
    pub fn read_word(&self, addr: u16, curr_tcycle: u32) -> u8 {
        let val = self.memory.borrow().read_word(addr);
        println_if_trace!("{: >5} MR {:04x} {:02x}", curr_tcycle, addr, val);
        val
    }

    #[cfg_attr(not(feature = "trace-interconnect"), allow(unused_variables))]
    pub fn write_word(&self, addr: u16, val: u8, curr_tcycle: u32) {
        self.memory.borrow_mut().write_word(addr, val);
        println_if_trace!("{: >5} MW {:04x} {:02x}", curr_tcycle, addr, val);
    }

    pub fn contend_port_early(&self, port: u16, curr_tcycle: u32) -> u32 {
        let delay = if self.is_addr_contended(port) {
            println_if_trace!("{: >5} PC {:04x}", curr_tcycle, port);
            self.ula_contention_no_mreq[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + 1
    }

    pub fn contend_port_late(&self, port: u16, curr_tcycle: u32) -> u32 {
        let delay = if (port & 0x0001) == 0 {
            println_if_trace!("{: >5} PC {:04x}", curr_tcycle, port);
            (self.ula_contention_no_mreq[curr_tcycle as usize] as u32) + 2
        } else {
            if self.is_addr_contended(port) {
                let mut delay: u32 = 0;
                println_if_trace!("{: >5} PC {:04x}", curr_tcycle + delay, port);
                delay += (self.ula_contention_no_mreq[(curr_tcycle + delay) as usize] as u32) + 1;
                println_if_trace!("{: >5} PC {:04x}", curr_tcycle + delay, port);
                delay += (self.ula_contention_no_mreq[(curr_tcycle + delay) as usize] as u32) + 1;
                println_if_trace!("{: >5} PC {:04x}", curr_tcycle + delay, port);
                delay += self.ula_contention_no_mreq[(curr_tcycle + delay) as usize] as u32;
                delay
            } else {
                2
            }
        };
        delay + 1
    }

    #[cfg_attr(not(feature = "trace-interconnect"), allow(unused_variables))]
    pub fn read_port(&self, port: u16, curr_tcycle: u32) -> u8 {
        let val = match port {
            port if port & 0x0001 == 0 => self.ula.borrow().read_port(port),
            0x7ffd => self.memory.borrow().read_port(port),
            0xfffd | 0xbffd => self.ay.borrow().read_port(port),
            _ => 0,
        };
        println_if_trace!("{: >5} PR {:04x} {:02x}", curr_tcycle, port, val);
        val
    }

    #[cfg_attr(not(feature = "trace-interconnect"), allow(unused_variables))]
    pub fn write_port(&self, port: u16, val: u8, curr_tcycle: u32) {
        println_if_trace!("{: >5} PW {:04x} {:02x}", curr_tcycle, port, val);
        match port {
            port if port & 0x0001 == 0 => self.ula.borrow_mut().write_port(port, val),
            0x7ffd => self.memory.borrow_mut().write_port(port, val),
            0xfffd | 0xbffd => self.ay.borrow_mut().write_port(port, val),
            _ => (),
        };
    }

    pub fn reset(&self) {
        self.memory.borrow_mut().clear();
    }
}
