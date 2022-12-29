use elven_parse::program::{ProgramHeader, Perm};
use crate::Error;

/// VirtAddr represents the virtual address for the guest machine
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

impl From<usize> for VirtAddr {
    fn from(item: usize) -> Self {
        VirtAddr(item)
    }
}

/// Mmu is the representation of the memory management unit
#[derive(Debug, Clone)]
pub struct Mmu {
    /// Guest address space
    /// Offset 0 corresponds to the 0 in the guest address space
    pub memory: Vec<u8>,

    /// Permission bits for every offset in the vector
    pub permissions: Vec<Perm>,

    /// Base of the current allocation
    pub alloc_base: VirtAddr,

    /// Record of the allocations for the free function
    pub alloc_active: std::collections::HashMap<VirtAddr, usize>
}

impl Mmu {
    /// Struct initialization sequence
    pub fn new(size: usize) -> Self {
        Mmu {
            memory: vec![0; size],
            permissions: vec![Perm(false, false, false); size],
            alloc_base: VirtAddr(0),
            alloc_active: std::collections::HashMap::new()
        }
    }

    /// load the section into the memory
    pub fn load(&mut self, program: &ProgramHeader, file: &[u8]) 
        -> Option<()> {
        // Put the loadable section into the memory
        self.write_from(program.p_vaddr.into(), file
            .get(program.p_offset..program.p_offset
                .checked_add(program.p_filesz)?)?).ok()?;
        
        // Set the appropriate permissions
        self.set_permissions(program.p_vaddr.into(), program.p_memsz,
        program.p_flags);

        // Move the base of the allocation to somewhere ahead of the loaded
        // segments and align to 4096 or 12 bit boundary
        self.alloc_base.0 = core::cmp::max(
            program.p_vaddr
            .checked_add(program.p_memsz + (program.p_align - 1))?
            & !(program.p_align - 1), self.alloc_base.0);

        Some(())
    }

    /// Set the permission for the region of memory
    pub fn set_permissions(&mut self, addr:VirtAddr, size: usize, perm: Perm) 
        -> Option<()> {
        // change the permission bits for that block of memory
        self.permissions.get_mut(addr.0..addr.0.checked_add(size)?)?
            .iter_mut().for_each(|x| *x = perm);

        Some(())
    }

    /// Helper function to write a buffer to an virtual address
    pub fn write_from(&mut self, addr: VirtAddr, buf: &[u8]) ->
        Result<(), Error> {
            // We should check for permission errors but not for now
            // Copy the buffer into the emulator memory
            self.memory[addr.0..addr.0 + buf.len()].copy_from_slice(buf);

            // Set the memory as readable
            self.set_permissions(addr, buf.len(), Perm(true, false, false));

            Ok(())
        }

    pub fn write(&mut self, addr: VirtAddr, size: usize, buf: [u8; 4]) ->
        Result<(), Error> {
            // We should check for permission errors but not for now
            // Copy the buffer into the emulator memory
            self.memory[addr.0..addr.0 + size].copy_from_slice(&buf[..size]);

            // Set the memory as readable
            //self.set_permissions(addr, buf.len(), Perm(true, false, false));

            Ok(())
        }

    /// Reads a word from memory 
    //FIXME Redundant
    pub fn read_word(&self, addr: VirtAddr) -> [u8; 4] {
        let mut buf = [0, 0, 0, 0];
        buf.copy_from_slice(&self.memory[addr.0..addr.0 + 4]);
        buf
    }

    /// Reads a byte from memory 
    //FIXME Redundant
    pub fn read_byte(&self, addr: VirtAddr) -> [u8; 1]{
        let mut buf =  [0];
        buf.copy_from_slice(&self.memory[addr.0..addr.0 + 1]);
        buf
    }

    /// Reads two bytes from memory 
    //FIXME Redundant
    pub fn read_half(&self, addr: VirtAddr) -> [u8; 2]{
        let mut buf =  [0, 0];
        buf.copy_from_slice(&self.memory[addr.0..addr.0 + 2]);
        buf
    }

    pub fn read(&self, addr: VirtAddr, size: usize) -> Result<&[u8], Error> {
        Ok(&self.memory[addr.0..addr.0 + size])
    }

    /// Allocate memory in mmu
    pub fn allocate(&mut self, size: usize) -> Option<VirtAddr> {
        // align to byte boundary
        let aligned_size = (size + 0x7) & !0x7;

        // See if you can allocate
        let base = self.alloc_base;
        if  base.0 > self.memory.len() {
            return None;
        }

        // Set the new allocation
        self.alloc_base = VirtAddr(self.alloc_base.0
            .checked_add(aligned_size)?);

        // Check if the new allocation surpasses the amount of memory we have
        if self.alloc_base.0 > self.memory.len() {
            return None;
        }

        // Set the permissions for the new section as read and writable
        self.set_permissions(base, size, Perm(true, true, false))?;

        self.alloc_active.insert(base, size);

        Some(base)

    }

    /// Free the allocated memory
    pub fn free(&mut self, base: VirtAddr) -> Result<(), Error> {
        // Remove the allocation from current ones
        if let Some(size) = self.alloc_active.remove(&base) {
            // Clear permissions
            self.set_permissions(base, size, Perm(false, false, false));

            Ok(())
        } else {
            Err(Error::InvalidFree)
        }
    }
}
