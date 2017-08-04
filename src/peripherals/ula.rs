use super::Peripheral;
use super::Memory;

use std::rc::Rc;
use std::cell::RefCell;

use sdl2::render::Texture;

static COLOURS: [(u8, u8, u8); 16] = [
    (   0,   0,   0 ),
    (   0,   0, 192 ),
    ( 192,   0,   0 ),
    ( 192,   0, 192 ),
    (   0, 192,   0 ),
    (   0, 192, 192 ),
    ( 192, 192,   0 ),
    ( 192, 192, 192 ),
    (   0,   0,   0 ),
    (   0,   0, 255 ),
    ( 255,   0,   0 ),
    ( 255,   0, 255 ),
    (   0, 255,   0 ),
    (   0, 255, 255 ),
    ( 255, 255,   0 ),
    ( 255, 255, 255 )
];

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

    pub fn display(&self, texture: &mut Texture) {
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for addr in 0x4000..0x5800 {
                let dispx: usize = addr & 0x001F;
                let dispy: usize = ((addr & 0x0700) >> 8 ) |
                                   ((addr & 0x00E0) >> 2 ) |
                                   ((addr & 0x1800) >> 5 ) ;

                let pixels = self.memory.borrow().read_word(addr as u16);

                let attrx = dispx;
                let attry = dispy / 8;
                let attr = attry * 32 + attrx;
                let attrdata = self.memory.borrow().read_word((0x5800 + attr) as u16);
                let ink = attrdata & 0x07;
                let paper = (attrdata & 0x38) >> 3;
                let brightness = if attrdata & 0x40 != 0 { 8 } else { 0 };

                for p in 0..8 {
                    let offset = dispy * pitch + ((dispx * 8) + (7 - p)) * 3;
                    let colour = if pixels & (1 << p) != 0 { ink } else { paper };

                    buffer[offset + 0] = COLOURS[(colour + brightness) as usize].0;
                    buffer[offset + 1] = COLOURS[(colour + brightness) as usize].1;
                    buffer[offset + 2] = COLOURS[(colour + brightness) as usize].2;
                }
            }
        }).unwrap();
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

