fn main() {
    println!("Hello, world!");
}

pub struct Emulator {
    /// Memory for the emulator
    pub mmu: usize,
}

impl Emulator {
    pub fn new(size: usize) -> Self {
        Self { mmu: size }
    }
}
