mod cpu;

fn main() {
    println!("Hello, world!");

    let mut cpu = cpu::Cpu::default();
    println!("{:?}", cpu);
}
