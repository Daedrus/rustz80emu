use super::Peripheral;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Ay {
    value: u8,
}

impl Ay {
    pub fn new() -> Self {
        Ay { value: 0 }
    }
}

impl Peripheral for Ay {
    fn read_port(&self, _: u16) -> u8 {
        0xff
    }

    fn write_port(&mut self, _: u16, val: u8) {
        self.value = val;
    }
}

