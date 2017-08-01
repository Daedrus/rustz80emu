use super::Peripheral;
use super::Memory;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Ula {
    value: u8,

    memory: Rc<RefCell<Memory>>,
}

impl Ula {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Self {
        Ula { value: 0,
              memory: memory,
        }
    }
}

impl Peripheral for Ula {
    fn read_port(&self, _: u16) -> u8 {
        self.value
    }

    fn write_port(&mut self, _: u16, val: u8) {
        if val & 0x10 != 0 {
            self.value = 0xff;
        } else {
            self.value = 0xbf;
        }
    }
}

