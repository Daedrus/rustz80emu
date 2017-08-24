use super::Peripheral;
use super::Memory;
use machine::SpectrumKeycode;

use std::rc::Rc;
use std::cell::RefCell;

use sdl2::render::Texture;

use std::collections::HashMap;


lazy_static! {
    static ref KEYBOARD_PORTS: HashMap<SpectrumKeycode,(u8, u8)> = {
        let mut m = HashMap::new();

        m.insert(SpectrumKeycode::Num1,   (3, 0x01));
        m.insert(SpectrumKeycode::Num2,   (3, 0x02));
        m.insert(SpectrumKeycode::Num3,   (3, 0x04));
        m.insert(SpectrumKeycode::Num4,   (3, 0x08));
        m.insert(SpectrumKeycode::Num5,   (3, 0x10));
        m.insert(SpectrumKeycode::Num6,   (4, 0x10));
        m.insert(SpectrumKeycode::Num7,   (4, 0x08));
        m.insert(SpectrumKeycode::Num8,   (4, 0x04));
        m.insert(SpectrumKeycode::Num9,   (4, 0x02));
        m.insert(SpectrumKeycode::Num0,   (4, 0x01));

        m.insert(SpectrumKeycode::Q,      (2, 0x01));
        m.insert(SpectrumKeycode::W,      (2, 0x02));
        m.insert(SpectrumKeycode::E,      (2, 0x04));
        m.insert(SpectrumKeycode::R,      (2, 0x08));
        m.insert(SpectrumKeycode::T,      (2, 0x10));
        m.insert(SpectrumKeycode::Y,      (5, 0x10));
        m.insert(SpectrumKeycode::U,      (5, 0x08));
        m.insert(SpectrumKeycode::I,      (5, 0x04));
        m.insert(SpectrumKeycode::O,      (5, 0x02));
        m.insert(SpectrumKeycode::P,      (5, 0x01));

        m.insert(SpectrumKeycode::A,      (1, 0x01));
        m.insert(SpectrumKeycode::S,      (1, 0x02));
        m.insert(SpectrumKeycode::D,      (1, 0x04));
        m.insert(SpectrumKeycode::F,      (1, 0x08));
        m.insert(SpectrumKeycode::G,      (1, 0x10));
        m.insert(SpectrumKeycode::H,      (6, 0x10));
        m.insert(SpectrumKeycode::J,      (6, 0x08));
        m.insert(SpectrumKeycode::K,      (6, 0x04));
        m.insert(SpectrumKeycode::L,      (6, 0x02));
        m.insert(SpectrumKeycode::Enter,  (6, 0x01));

        m.insert(SpectrumKeycode::Caps,   (0, 0x01));
        m.insert(SpectrumKeycode::Z,      (0, 0x02));
        m.insert(SpectrumKeycode::X,      (0, 0x04));
        m.insert(SpectrumKeycode::C,      (0, 0x08));
        m.insert(SpectrumKeycode::V,      (0, 0x10));
        m.insert(SpectrumKeycode::B,      (7, 0x10));
        m.insert(SpectrumKeycode::N,      (7, 0x08));
        m.insert(SpectrumKeycode::M,      (7, 0x04));
        m.insert(SpectrumKeycode::Symbol, (7, 0x02));
        m.insert(SpectrumKeycode::Space,  (7, 0x01));

        m.insert(SpectrumKeycode::NONE,   (0, 0x00));

        m
    };
}

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

    keyboard_ports: [u8; 8],
}

impl Ula {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Self {
        Ula { value: 0,
              memory,
              keyboard_ports: [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
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

    pub fn key_pressed(&mut self, keycode: &SpectrumKeycode) {
        match KEYBOARD_PORTS.get(keycode) {
            Some(&(ref port, ref value)) => {
                self.keyboard_ports[*port as usize] &= !*value;
            }
            None => {}
        }
    }

    pub fn key_released(&mut self, keycode: &SpectrumKeycode) {
        match KEYBOARD_PORTS.get(keycode) {
            Some(&(ref port, ref value)) => {
                self.keyboard_ports[*port as usize] |= *value;
            }
            None => {}
        }
    }
}

impl Peripheral for Ula {
    fn read_port(&self, port: u16) -> u8 {
        let mut data = 0xff;
        let mut porth: u8 = (port >> 8) as u8;
        for i in 0..8 {
            if porth & 0x01 == 0x00 {
                data = data & self.keyboard_ports[i];
            }
            porth = porth >> 1;
        }
        data
    }

    fn write_port(&mut self, _: u16, val: u8) {
        if val & 0x10 != 0 {
            self.value = 0xff;
        } else {
            self.value = 0xbf;
        }
    }
}

