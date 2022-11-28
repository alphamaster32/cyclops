use elven_parse::program::{ProgramHeader, Perm};
use crate::Error;

/// VirtAddr represents the virtual address for the guest machine
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct VirtAddr(pub usize);

impl From<usize> for VirtAddr {
    fn from(item: usize) -> Self {
        VirtAddr(item)
    }
}

/// Mmu is the representation of the memory management unit
#[derive(Debug)]
pub struct Mmu {
    /// Guest address space
    /// Offset 0 corresponds to the 0 in the guest address space
    pub memory: Vec<u8>,

    /// Permission bits for every offset in the vector
    pub permissions: Vec<Perm>,
}

impl Mmu {
    /// Struct initialization sequence
    pub fn new(size: usize) -> Self {
        Mmu {
            memory: vec![0; size],
            permissions: vec![Perm(false, false, false); size],
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

    pub fn write_from(&mut self, addr: VirtAddr, buf: &[u8]) ->
        Result<(), Error> {
        // We should check for permission errors but not for now
        // Copy the buffer into the emulator memory
        self.memory[addr.0..addr.0 + buf.len()].copy_from_slice(buf);

        // Set the memory as readable
        self.set_permissions(addr, buf.len(), Perm(true, false, false));

        Ok(())
    }
}
