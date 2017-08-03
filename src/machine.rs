use ::interconnect::*;
use ::peripherals::*;
use ::cpu::*;
use ::debugger::*;
use ::utils::read_bin;

use std::path::Path;

use std::rc::Rc;
use std::cell::RefCell;


pub struct Machine {
    cpu: Rc<RefCell<Cpu>>,
    memory: Rc<RefCell<Memory>>,
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
            debug_on: start_in_debug,
        }
    }

    pub fn run(&self) {
        let mut debugger = Debugger::new(
            self.cpu.clone(),
            self.memory.clone());

        loop {
            if self.debug_on { debugger.pre(); }

            self.cpu.borrow_mut().handle_interrupts();
            self.cpu.borrow_mut().run_instruction();

            if self.debug_on { debugger.post(); }
        }
    }
}
