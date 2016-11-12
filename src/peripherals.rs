pub trait Peripheral {
    fn read_port(&self, port: u16) -> u8;
    fn write_port(&mut self, port: u16, val: u8);
}


#[derive(RustcEncodable, RustcDecodable)]
pub struct Ula {
    pub value: u8
}

impl Peripheral for Ula {
    fn read_port(&self, _: u16) -> u8 {
        self.value
    }

    fn write_port(&mut self, _ : u16, val: u8) {
        if val & 0x10 != 0 {
            self.value = 0xff;
        } else {
            self.value = 0xbf;
        }
    }
}


#[derive(RustcEncodable, RustcDecodable)]
pub struct Ay {
    pub value: u8
}

impl Peripheral for Ay {
    fn read_port(&self, _: u16) -> u8 {
        0
    }

    fn write_port(&mut self, _: u16, val: u8) {
        self.value = 0;
    }
}
