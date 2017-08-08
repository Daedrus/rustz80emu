use ::interconnect::*;
use ::peripherals::*;
use ::cpu::*;
use ::debugger::*;
use ::utils::read_bin;

use std::path::Path;

use std::rc::Rc;
use std::cell::RefCell;

extern crate sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum SpectrumKeycode {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Space,
    Enter,
    Caps,
    Symbol,

    NONE,
}

lazy_static! {
    static ref KEYBOARD_MAPPINGS: HashMap<Keycode,(SpectrumKeycode, SpectrumKeycode)> = {
        let mut m = HashMap::new();

        m.insert(Keycode::Escape,    (SpectrumKeycode::Num1,  SpectrumKeycode::Caps));
        m.insert(Keycode::Num1,      (SpectrumKeycode::Num1,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num2,      (SpectrumKeycode::Num2,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num3,      (SpectrumKeycode::Num3,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num4,      (SpectrumKeycode::Num4,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num5,      (SpectrumKeycode::Num5,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num6,      (SpectrumKeycode::Num6,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num7,      (SpectrumKeycode::Num7,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num8,      (SpectrumKeycode::Num8,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num9,      (SpectrumKeycode::Num9,  SpectrumKeycode::NONE));
        m.insert(Keycode::Num0,      (SpectrumKeycode::Num0,  SpectrumKeycode::NONE));
        m.insert(Keycode::Minus,     (SpectrumKeycode::J,     SpectrumKeycode::Symbol));
        m.insert(Keycode::Equals,    (SpectrumKeycode::L,     SpectrumKeycode::Symbol));
        m.insert(Keycode::Backspace, (SpectrumKeycode::Num0,  SpectrumKeycode::Caps));

        m.insert(Keycode::Tab,       (SpectrumKeycode::Caps,  SpectrumKeycode::Symbol));
        m.insert(Keycode::Q,         (SpectrumKeycode::Q,     SpectrumKeycode::NONE));
        m.insert(Keycode::W,         (SpectrumKeycode::W,     SpectrumKeycode::NONE));
        m.insert(Keycode::E,         (SpectrumKeycode::E,     SpectrumKeycode::NONE));
        m.insert(Keycode::R,         (SpectrumKeycode::R,     SpectrumKeycode::NONE));
        m.insert(Keycode::T,         (SpectrumKeycode::T,     SpectrumKeycode::NONE));
        m.insert(Keycode::Y,         (SpectrumKeycode::Y,     SpectrumKeycode::NONE));
        m.insert(Keycode::U,         (SpectrumKeycode::U,     SpectrumKeycode::NONE));
        m.insert(Keycode::I,         (SpectrumKeycode::I,     SpectrumKeycode::NONE));
        m.insert(Keycode::O,         (SpectrumKeycode::O,     SpectrumKeycode::NONE));
        m.insert(Keycode::P,         (SpectrumKeycode::P,     SpectrumKeycode::NONE));

        m.insert(Keycode::CapsLock,  (SpectrumKeycode::Num2,  SpectrumKeycode::Caps));
        m.insert(Keycode::A,         (SpectrumKeycode::A,     SpectrumKeycode::NONE));
        m.insert(Keycode::S,         (SpectrumKeycode::S,     SpectrumKeycode::NONE));
        m.insert(Keycode::D,         (SpectrumKeycode::D,     SpectrumKeycode::NONE));
        m.insert(Keycode::F,         (SpectrumKeycode::F,     SpectrumKeycode::NONE));
        m.insert(Keycode::G,         (SpectrumKeycode::G,     SpectrumKeycode::NONE));
        m.insert(Keycode::H,         (SpectrumKeycode::H,     SpectrumKeycode::NONE));
        m.insert(Keycode::J,         (SpectrumKeycode::J,     SpectrumKeycode::NONE));
        m.insert(Keycode::K,         (SpectrumKeycode::K,     SpectrumKeycode::NONE));
        m.insert(Keycode::L,         (SpectrumKeycode::L,     SpectrumKeycode::NONE));
        m.insert(Keycode::Semicolon, (SpectrumKeycode::O,     SpectrumKeycode::Symbol));
        m.insert(Keycode::Quote,     (SpectrumKeycode::Num7,  SpectrumKeycode::Symbol));
        m.insert(Keycode::Hash,      (SpectrumKeycode::Num3,  SpectrumKeycode::Symbol));
        m.insert(Keycode::Return,    (SpectrumKeycode::Enter, SpectrumKeycode::NONE));

        m.insert(Keycode::LShift,    (SpectrumKeycode::NONE,  SpectrumKeycode::Caps));
        m.insert(Keycode::Z,         (SpectrumKeycode::Z,     SpectrumKeycode::NONE));
        m.insert(Keycode::X,         (SpectrumKeycode::X,     SpectrumKeycode::NONE));
        m.insert(Keycode::C,         (SpectrumKeycode::C,     SpectrumKeycode::NONE));
        m.insert(Keycode::V,         (SpectrumKeycode::V,     SpectrumKeycode::NONE));
        m.insert(Keycode::B,         (SpectrumKeycode::B,     SpectrumKeycode::NONE));
        m.insert(Keycode::N,         (SpectrumKeycode::N,     SpectrumKeycode::NONE));
        m.insert(Keycode::M,         (SpectrumKeycode::M,     SpectrumKeycode::NONE));
        m.insert(Keycode::Comma,     (SpectrumKeycode::N,     SpectrumKeycode::Symbol));
        m.insert(Keycode::Period,    (SpectrumKeycode::M,     SpectrumKeycode::Symbol));
        m.insert(Keycode::Slash,     (SpectrumKeycode::V,     SpectrumKeycode::Symbol));
        m.insert(Keycode::RShift,    (SpectrumKeycode::NONE,  SpectrumKeycode::Caps));

        m.insert(Keycode::LCtrl,     (SpectrumKeycode::NONE,  SpectrumKeycode::Symbol));
        m.insert(Keycode::LAlt,      (SpectrumKeycode::NONE,  SpectrumKeycode::Symbol));
        m.insert(Keycode::Space,     (SpectrumKeycode::Space, SpectrumKeycode::NONE));
        m.insert(Keycode::RAlt,      (SpectrumKeycode::NONE,  SpectrumKeycode::Symbol));
        m.insert(Keycode::RCtrl,     (SpectrumKeycode::NONE,  SpectrumKeycode::Symbol));
        m.insert(Keycode::Mode,      (SpectrumKeycode::NONE,  SpectrumKeycode::Symbol));

        m.insert(Keycode::Left,      (SpectrumKeycode::Num5,  SpectrumKeycode::Caps));
        m.insert(Keycode::Down,      (SpectrumKeycode::Num6,  SpectrumKeycode::Caps));
        m.insert(Keycode::Up,        (SpectrumKeycode::Num7,  SpectrumKeycode::Caps));
        m.insert(Keycode::Right,     (SpectrumKeycode::Num8,  SpectrumKeycode::Caps));
        m
    };
}

pub struct Machine {
    cpu: Rc<RefCell<Cpu>>,
    memory: Rc<RefCell<Memory>>,
    ula: Rc<RefCell<Ula>>,
    debug_on: bool,
}

impl Machine {
    pub fn new(start_in_debug: bool) -> Self {
        let rom0 = read_bin(Path::new("./roms/128-0.rom"));
        let rom1 = read_bin(Path::new("./roms/128-1.rom"));

        let memory = Rc::new(RefCell::new(MemoryBuilder::new()
            .rom0(rom0)
            .rom1(rom1)
            .finalize()));

        let ay = Rc::new(RefCell::new(Ay::new()));
        let ula = Rc::new(RefCell::new(Ula::new(memory.clone())));

        let interconnect = Interconnect::new(
            memory.clone(),
            ay.clone(),
            ula.clone());

        let cpu = Rc::new(RefCell::new(Cpu::new(interconnect)));

        Machine {
            cpu: cpu,
            memory: memory,
            ula: ula,
            debug_on: start_in_debug,
        }
    }

    pub fn run(&self) {
        let mut debugger = Debugger::new(
            self.cpu.clone(),
            self.memory.clone());

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("rustz80emu", 512, 384)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGB24, 256, 192).unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();

        'machine: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => {
                        break 'machine
                    },
                    Event::KeyDown { keycode:Some(k), ..} => {
                        match KEYBOARD_MAPPINGS.get(&k) {
                            Some(&(ref key1, ref key2)) => {
                                self.ula.borrow_mut().key_pressed(key1);
                                self.ula.borrow_mut().key_pressed(key2);
                            }
                            None => {}
                        }
                    },
                    Event::KeyUp { keycode:Some(k), ..} => {
                        match KEYBOARD_MAPPINGS.get(&k) {
                            Some(&(ref key1, ref key2)) => {
                                self.ula.borrow_mut().key_released(key1);
                                self.ula.borrow_mut().key_released(key2);
                            }
                            None => {}
                        }
                    }
                    _ => {}
                }
            }

            if self.debug_on { debugger.pre(); }

            if self.cpu.borrow().tcycles > 70908 {
                self.ula.borrow().display(&mut texture);

                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0, 512, 384))).unwrap();
                canvas.present();
            }

            self.cpu.borrow_mut().handle_interrupts();
            self.cpu.borrow_mut().run_instruction();

            if self.debug_on { debugger.post(); }
        }
    }
}
