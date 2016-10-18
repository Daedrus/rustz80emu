use std::error::Error;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use super::cpu::*;

pub fn save_cpu(cpu: &Cpu, path: &str) {
    use std::fs::File;
    use std::io::Write;
    let encoded: Vec<u8> = encode(cpu, SizeLimit::Infinite).unwrap();
    let mut file = File::create(path).unwrap();
    match file.write_all(&encoded) {
        Err(e) => {
            panic!("Couldn't write to {} : {}", path, e.description());
        }
        Ok(_) => (),
    }
}

pub fn load_cpu(path: &str) -> Cpu {
    use std::fs::File;
    use std::io::Read;
    let mut encoded = Vec::new();
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut encoded).unwrap();
    decode(&encoded[..]).unwrap()
}
