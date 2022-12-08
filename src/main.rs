use elven_parse::Elf;
use elven_parse::program::ProgramType;

use core::arch::x86_64::_rdtsc;

mod mmu;
mod emu;

/// Error type for the emulator
#[derive(Debug)]
pub enum Error {
    /// Indicates that the free function was unable to find the allocation
    InvalidFree
}

/// Wrapper type for the error result
type Result<T> = core::result::Result<T, Error>;

fn main() -> Result<()>{
    // Get the file handle for the intended binary and parse the elf
    let file = std::fs::read("test/test").unwrap();
    let elf = Elf::new(file.as_slice()).parse().unwrap();

    let it = unsafe { _rdtsc() };

    // allocate a 512 MiB memory block for the emulator
    let mut emu = emu::Emu::new(64 * 1024 * 1024);
    
    // Check all the sections and load only the load section with the required
    // permissions
    for program in elf.program_iter() {
        if program.p_type == ProgramType::PtLoad {
            emu.mmu.load(&program, file.as_slice());
        }
    }

    // Calculate the end of the text section from SectionHeader
    let mut text_addr: usize = 0;
    let mut text_size: usize = 0;
    for section in elf.section_iter() {
        if section.sh_flags.is_exec() && section.sh_flags.is_alloc() {
            text_addr = section.sh_addr;
            text_size = section.sh_size;
        }
    }

    // Get the program counter from the elf header
    let mut pc = elf.file_header.e_entry;
    // Calculate the end of the text section
    let text_end = text_size + text_addr;
    loop {
        if pc >= text_end {
            break
        }
        let inst = u32::from_le_bytes(emu.mmu.memory.get(
            pc..pc.checked_add(4).unwrap()).unwrap().try_into().unwrap());
        let inst: emu::Inst = inst.into();
        println!("{:x?}: {:x?}", pc, inst);
        pc = pc + 4;
    }

    let el = unsafe { _rdtsc() };

    let base = emu.mmu.allocate(0x10).ok_or(Error::InvalidFree)?;
    emu.mmu.free(base);
    println!("{:x?}", emu.mmu.alloc_base);

    //println!("{:?}", el - it);
    //println!("{:x?}", emu.mmu.alloc_base);

    Ok(())
}
