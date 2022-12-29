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

const PERM_READ: u8 = 1 << 0;
const PERM_WRITE: u8 = 1 << 1;
const PERM_EXEC: u8 = 1 << 2;
const PERM_RAW: u8 = 1 << 3;

/// Block size used for resetting and tracking memory which has been
/// written-to. The bigger this is, the fewer but more expensive memcpys need to occur.
/// The smaller, the greater but less expensive ones need to occur.
const DIRTY_BLOCK_SIZE: usize = 4096;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Perm(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);

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
    pub fn read_into(&mut self, addr: VirtAddr, buf: &mut [u8]) -> Option<()> {
        let perms = self
            .permissions
            .get(addr.0..addr.0.checked_add(buf.len())?)?;

        if !perms.iter().all(|x| (x.0 & PERM_READ) != 0) {
            return None;
        }
        buf.copy_from_slice(
            self.memory
                .get_mut(addr.0..addr.0.checked_add(buf.len())?)?,
        );
        Some(())
    }
    pub fn read(&mut self, addr: VirtAddr) {}

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

pub struct Emulator {
    /// Memory for the emulator
    pub mmu: Mmu,
}

impl Emulator {
    pub fn new(size: usize) -> Self {
        Self {
            mmu: Mmu::new(size),
        }
    }
}
