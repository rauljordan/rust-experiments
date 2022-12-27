#[repr(transparent)]
#[derive(Clone)]
pub struct Perm(u8);

pub struct Emulator {
    /// Memory for the emulator
    mem: Mmu,
}

impl Emulator {
    pub fn new(size: usize) -> Self {
        Self {
            mem: Mmu::new(size),
        }
    }
}

/// Isolated memory space.
pub struct Mmu {
    memory: Vec<u8>,
    permissions: Vec<Perm>,
}

impl Mmu {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            permissions: vec![Perm(0); size],
        }
    }
}

fn foo() {
    let mut _emu = Emulator::new(1024 * 1024);
}
