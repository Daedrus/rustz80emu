use super::memory::*;
use super::peripherals::*;

use std::rc::Rc;
use std::cell::RefCell;

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

        let ula_contention = include_bytes!("ulacontention.bin");
        let ula_contention_no_mreq = include_bytes!("ulacontention.bin");

        Interconnect {
            memory: memory,
            ay: ay,
            ula: ula,
            ula_contention: ula_contention.to_vec(),
            ula_contention_no_mreq: ula_contention_no_mreq.to_vec(),
        }
    }

    fn is_addr_contended(&self, addr: u16) -> bool {
        (addr >= 0x4000 && addr < 0x8000) ||
        (addr >= 0xC000 && (self.memory.borrow().get_c000_bank() % 2 != 0))
    }

    #[inline(always)]
    pub fn contend_read(&mut self, addr: u16, curr_tcycle: u32, tcycles: u32) -> u32 {
        //println!("{: >5} MC {:04x}", curr_tcycle, addr);
        let delay = if self.is_addr_contended(addr) {
            self.ula_contention[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + tcycles
    }

    #[inline(always)]
    pub fn contend_read_no_mreq(&mut self, addr: u16, curr_tcycle: u32) -> u32 {
        //println!("{: >5} MC {:04x}", curr_tcycle, addr);
        let delay = if self.is_addr_contended(addr) {
            self.ula_contention_no_mreq[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + 1
    }

    #[inline(always)]
    pub fn contend_write_no_mreq(&mut self, addr: u16, curr_tcycle: u32) -> u32 {
        //println!("{: >5} MC {:04x}", curr_tcycle, addr);
        let delay = if self.is_addr_contended(addr) {
            self.ula_contention_no_mreq[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + 1
    }

    pub fn read_word(&self, addr: u16, _curr_tcycle: u32) -> u8 {
        let val = self.memory.borrow().read_word(addr);
        //println!("{: >5} MR {:04x} {:02x}", _curr_tcycle, addr, val);
        val
    }

    pub fn write_word(&mut self, addr: u16, val: u8, _curr_tcycle: u32) {
        self.memory.borrow_mut().write_word(addr, val);
        //println!("{: >5} MW {:04x} {:02x}", _curr_tcycle, addr, val);
    }

    pub fn contend_port_early(&mut self, port: u16, curr_tcycle: u32) -> u32 {
        let delay = if self.is_addr_contended(port) {
            //println!("{: >5} PC {:04x}", curr_tcycle, port);
            self.ula_contention_no_mreq[curr_tcycle as usize] as u32
        } else {
            0
        };
        delay + 1
    }

    pub fn contend_port_late(&mut self, port: u16, curr_tcycle: u32) -> u32 {
        let delay = if (port & 0x0001) == 0 {
            //println!("{: >5} PC {:04x}", curr_tcycle, port);
            (self.ula_contention_no_mreq[curr_tcycle as usize] as u32) + 2
        } else {
            if self.is_addr_contended(port) {
                let mut delay: u32 = 0;
                //println!("{: >5} PC {:04x}", curr_tcycle + delay, port);
                delay += (self.ula_contention_no_mreq[(curr_tcycle + delay) as usize] as u32) + 1;
                //println!("{: >5} PC {:04x}", curr_tcycle + delay, port);
                delay += (self.ula_contention_no_mreq[(curr_tcycle + delay) as usize] as u32) + 1;
                //println!("{: >5} PC {:04x}", curr_tcycle + delay, port);
                delay += self.ula_contention_no_mreq[(curr_tcycle + delay) as usize] as u32;
                delay
            } else {
                2
            }
        };
        delay + 1
    }

    pub fn read_port(&mut self, port: u16, _curr_tcycle: u32) -> u8 {
        let val = match port {
            port if port & 0x0001 == 0 => self.ula.borrow().read_port(port),
            0x7ffd => self.memory.borrow().read_port(port),
            0xfffd | 0xbffd => self.ay.borrow().read_port(port),
            _ => 0,
        };
        //println!("{: >5} PR {:04x} {:02x}", _curr_tcycle, port, val);
        val
    }

    pub fn write_port(&mut self, port: u16, val: u8, _curr_tcycle: u32) {
        //println!("{: >5} PW {:04x} {:02x}", _curr_tcycle, port, val);
        match port {
            port if port & 0x0001 == 0 => self.ula.borrow_mut().write_port(port, val),
            0x7ffd => self.memory.borrow_mut().write_port(port, val),
            0xfffd | 0xbffd => self.ay.borrow_mut().write_port(port, val),
            _ => (),
        };
    }

    pub fn reset(&mut self) {
        self.memory.borrow_mut().clear();
    }
}
