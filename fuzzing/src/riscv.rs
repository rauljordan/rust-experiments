use std::{io, path::Path};

pub struct Section {
    pub file_off: usize,
    pub virt_addr: VirtAddr,
    pub file_size: usize,
    pub mem_size: usize,
    pub permissions: Perm,
}

pub struct Emulator {
    /// Memory for the emulator
    pub mmu: Mmu,
    registers: [u64; 33],
}

impl Emulator {
    pub fn new(size: usize) -> Self {
        Self {
            mmu: Mmu::new(size),
            registers: [0u64; 33],
        }
    }
    pub fn fork(&mut self) -> Self {
        Self {
            mmu: self.mmu.fork(),
            registers: self.registers.clone(),
        }
    }
    pub fn reset(&mut self, other: &Self) {
        self.mmu.reset(&other.mmu);
        self.registers = other.registers;
    }
    pub fn reg(&self, r: Register) -> u64 {
        self.registers[r as usize]
    }
    pub fn set_reg(&mut self, r: Register, val: u64) {
        self.registers[r as usize] = val
    }
    pub fn run(&mut self) {
        loop {
            let pc = self.reg(Register::Pc);
            let inst: u32 = self
                .mmu
                .read_perms(VirtAddr(pc as usize), Perm(PERM_READ))
                .unwrap();

            print!("Instruction: {}\n", inst);
            let opcode = inst & 0x0000007f;
            print!("Instruction: {:#x}\n", inst);
            print!("Opcode: {:#x}\n", opcode);
            print!("Opcode: {:b}\n", opcode);
            match opcode {
                // AUIPC
                0b0010111 => {
                    let inst = Utype::from(inst);
                    self.set_reg(inst.rd, (inst.imm as i64 as u64).wrapping_add(pc));
                }
                _ => unimplemented!("Unimplemented opcode: {:b}\n", opcode),
            }
            self.set_reg(Register::Pc, pc.wrapping_add(4));
        }
    }
    pub fn load<P: AsRef<Path>>(&mut self, filename: P, sections: &[Section]) -> Option<()> {
        let contents = std::fs::read(filename).ok()?;
        for section in sections {
            // Allow writable permissions.
            self.mmu
                .set_permissions(section.virt_addr, section.mem_size, Perm(PERM_WRITE))?;

            // Write from contents.
            self.mmu.write_from(
                section.virt_addr,
                contents.get(section.file_off..section.file_off.checked_add(section.file_size)?)?,
            )?;

            // Write in any paddings.
            if section.mem_size > section.file_size {
                let padding = vec![0u8; section.mem_size - section.file_size];
                self.mmu.write_from(
                    VirtAddr(section.virt_addr.0.checked_add(section.file_size)?),
                    &padding,
                )?;
            }

            // Reset.
            self.mmu
                .set_permissions(section.virt_addr, section.mem_size, Perm(PERM_READ))?;

            // Update the allocator beyond any sections.
            self.mmu.cur_alc = VirtAddr(std::cmp::max(
                self.mmu.cur_alc.0,
                (section.virt_addr.0 + section.mem_size + 0xf) & !0xf,
            ));
        }
        Some(())
    }
}

#[derive(Debug)]
struct Utype {
    imm: i32,
    rd: Register,
}

impl From<u32> for Utype {
    fn from(inst: u32) -> Self {
        Utype {
            imm: (inst & !0xfff) as i32,
            rd: Register::from((inst >> 7) & 0b11111),
        }
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        assert!(value < 33);
        unsafe { core::ptr::read_unaligned(&(value as usize) as *const usize as *const Register) }
    }
}

#[derive(Debug)]
#[repr(usize)]
pub enum Register {
    Zero = 0,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
    Pc,
}

pub const PERM_READ: u8 = 1 << 0;
pub const PERM_WRITE: u8 = 1 << 1;
pub const PERM_EXEC: u8 = 1 << 2;
pub const PERM_RAW: u8 = 1 << 3;

/// Block size used for resetting and tracking memory which has been
/// written-to. The bigger this is, the fewer but more expensive memcpys need to occur.
/// The smaller, the greater but less expensive ones need to occur.
const DIRTY_BLOCK_SIZE: usize = 4096;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Perm(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

/// Isolated memory space.
pub struct Mmu {
    memory: Vec<u8>,
    permissions: Vec<Perm>,
    /// Tracks block indices in memory which are dirty.
    dirty: Vec<usize>,
    /// Tracks which parts of memory have been dirty.
    dirty_bitmap: Vec<u64>,
    cur_alc: VirtAddr,
}

impl Mmu {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            permissions: vec![Perm(0); size],
            dirty: Vec::with_capacity(size / DIRTY_BLOCK_SIZE + 1),
            dirty_bitmap: vec![0u64; size / DIRTY_BLOCK_SIZE / 64 + 1],
            cur_alc: VirtAddr(0x10000),
        }
    }
    pub fn fork(&mut self) -> Self {
        let size = self.memory.len();
        Self {
            memory: self.memory.clone(),
            permissions: self.permissions.clone(),
            dirty: Vec::with_capacity(size / DIRTY_BLOCK_SIZE + 1),
            dirty_bitmap: vec![0u64; size / DIRTY_BLOCK_SIZE / 64 + 1],
            cur_alc: self.cur_alc.clone(),
        }
    }
    /// Restores all dirty blocks to state of other.
    pub fn reset(&mut self, other: &Mmu) {
        for &block in &self.dirty {
            // Start and end addrs of the dirty memory.
            let start = block + DIRTY_BLOCK_SIZE;
            let end = (block + 1) + DIRTY_BLOCK_SIZE;

            // Zero the bitmap.
            self.dirty_bitmap[block / 64] = 0;

            // Restore memory.
            self.memory[start..end].copy_from_slice(&other.memory[start..end]);

            // Restore permissions.
            self.permissions[start..end].copy_from_slice(&other.permissions[start..end]);
        }
        self.dirty.clear();
    }
    pub fn write_from(&mut self, addr: VirtAddr, buf: &[u8]) -> Option<()> {
        let perms = self
            .permissions
            .get_mut(addr.0..addr.0.checked_add(buf.len())?)?;

        // All bits must have write perms.
        let mut has_raw = false;
        if !perms.iter().all(|x| {
            has_raw |= (x.0 & PERM_RAW) != 0;
            (x.0 & PERM_WRITE) != 0
        }) {
            return None;
        }

        self.memory
            .get_mut(addr.0..addr.0.checked_add(buf.len())?)?
            .copy_from_slice(buf);

        // Compute dirt bit blocks;
        let start = addr.0 / DIRTY_BLOCK_SIZE;
        let end = (addr.0 + buf.len()) / DIRTY_BLOCK_SIZE;
        for block in start..=end {
            // Bitmap position of the dirty block.
            let idx = start / 64;
            let bit = start % 64;

            // Check if block is not dirty.
            if self.dirty_bitmap[idx] & (1 << bit) == 0 {
                // Block is not dirty, add it to the dirty bitmap and vec.
                self.dirty.push(block);
                self.dirty_bitmap[idx] |= 1 << bit;
            }
        }

        // Update RaW bits.
        if has_raw {
            perms.iter_mut().for_each(|x| {
                if (x.0 & PERM_RAW) != 0 {
                    *x = Perm(x.0 | PERM_READ);
                }
            });
        }
        Some(())
    }

    pub fn read_perms(&mut self, addr: VirtAddr, exp_perms: Perm) -> Result<u32, ()> {
        let mut tmp = [0u8; 16];
        self.read_into_perms(addr, &mut tmp[..core::mem::size_of::<u32>()], exp_perms)
            .unwrap();
        Ok(unsafe { core::ptr::read_unaligned(tmp.as_ptr() as *const u32) })
    }

    pub fn read_into(&mut self, addr: VirtAddr, buf: &mut [u8]) -> Option<()> {
        self.read_into_perms(addr, buf, Perm(PERM_READ))
    }

    /// Read the memory at `addr` into `buf`
    /// This function checks to see if all bits in `exp_perms` are set in the
    /// permission bytes. If this is zero, we ignore permissions entirely.
    pub fn read_into_perms(
        &mut self,
        addr: VirtAddr,
        buf: &mut [u8],
        exp_perms: Perm,
    ) -> Option<()> {
        let perms = self
            .permissions
            .get(addr.0..addr.0.checked_add(buf.len())?)?;

        for (idx, &perm) in perms.iter().enumerate() {
            if (perm.0 & exp_perms.0) != exp_perms.0 {
                return None;
            }
        }

        buf.copy_from_slice(
            self.memory
                .get_mut(addr.0..addr.0.checked_add(buf.len())?)?,
        );
        Some(())
    }

    // Allocates a region of memory as RW in the address space
    pub fn allocate(&mut self, size: usize) -> Option<VirtAddr> {
        let align_size = (size * 0xf) & !0xf;
        let VirtAddr(base) = self.cur_alc;
        if base >= self.memory.len() {
            return None;
        }
        self.cur_alc = VirtAddr(self.cur_alc.0.checked_add(align_size)?);

        if self.cur_alc.0 > self.memory.len() {
            return None;
        }
        // Mark as uninitialized and writable.
        self.set_permissions(VirtAddr(base), size, Perm(PERM_RAW | PERM_WRITE));
        Some(VirtAddr(base))
    }
    pub fn set_permissions(&mut self, addr: VirtAddr, size: usize, perm: Perm) -> Option<()> {
        self.permissions
            .get_mut(addr.0..addr.0.checked_add(size)?)?
            .iter_mut()
            .for_each(|x| *x = perm);
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dirty() {
        let mut emu = Emulator::new(1024 * 1024);
        let tmp = emu.mmu.allocate(4096).unwrap();
        let data = b"asdf";
        emu.mmu.write_from(VirtAddr(tmp.0), data).unwrap();
        println!("Original dirty {:x?}", emu.mmu.dirty);

        {
            let mut forked = emu.mmu.fork();
            // Write and read into forked mem.
            forked.write_from(VirtAddr(tmp.0), data).unwrap();
            let mut bytes = [0u8; 4];
            forked.read_into(tmp, &mut bytes).unwrap();
            println!("Forked dirty {:x?}", forked.dirty);

            forked.reset(&emu.mmu);

            let mut bytes = [0u8; 4];
            let read_result = forked.read_into(tmp, &mut bytes);
            println!("{:?}", read_result);

            println!("Forked after reset {:x?}", forked.dirty);
        }
    }
    #[test]
    fn allocate_then_write_then_read() {
        let mut emu = Emulator::new(1024 * 1024);
        let tmp = emu.mmu.allocate(4096);
        assert_eq!(tmp.is_some(), true);
        let tmp = tmp.unwrap();
        let data = b"asdf";
        assert_eq!(emu.mmu.write_from(VirtAddr(tmp.0), data).is_some(), true);
        let mut buf = vec![0u8; 32];
        assert_eq!(emu.mmu.read_into(tmp, &mut buf).is_none(), true);
    }
}
