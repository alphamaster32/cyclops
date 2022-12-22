use elven_parse::Elf;
use elven_parse::program::ProgramType;

use core::arch::x86_64::_rdtsc;

mod mmu;
mod emu;
mod vcpu;

/// Error type for the emulator
#[derive(Debug)]
pub enum Error {
    /// Integer overflow or underflow
    ArithmeticException,
    /// Indicates that the free function was unable to find the allocation
    InvalidFree,
    /// Stage is occupied
    NotEmpty,
}

/// Wrapper type for the error result
type Result<T> = core::result::Result<T, Error>;

fn main() -> Result<()>{
    // Get the file handle for the intended binary and parse the elf
    let file = std::fs::read("test/test").unwrap();
    let elf = Elf::new(file.as_slice()).parse().unwrap();

    // Allocate a 512 MiB memory block for the emulator
    let mut emu = emu::Emu::new(128 * 1024 * 1024);
    
    // Allocate stack
    let stack = emu.mmu.allocate(64 * 1024);
    let stack = stack.unwrap().0 + 64 * 1024;
    
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
    let pc = elf.file_header.e_entry;
    // Calculate the end of the text section
    let text_end = text_size + text_addr;
    // set the pc in emu
    emu.pc = pc;
    emu.state.set(emu::Register::Pc, pc as u32);
    emu.state.set(emu::Register::Sp, stack as u32);

    //Cycle a instruction in emu
    for i in 1..10000 {
        let pc = emu.state.reg(emu::Register::Pc);
        emu.cycle()?;
        //println!("{:#}", emu);
    }
    println!("{:#}", emu);
    //println!("{:#x?}", emu.pipe);

    Ok(())
}
