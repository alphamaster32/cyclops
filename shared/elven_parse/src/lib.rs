#![no_std]

mod utils;

pub mod file;
pub mod program;
pub mod section;

use file::FileHeader;
use program::ProgramIterator;
use section::SectionIterator;

/// Elf type to store the parsed information
/// Struct members are defined according to the elf.h C header
pub struct Elf<'a> {
    /// Elf file header
    pub file_header: FileHeader,
    /// Reference to the elf file in memory
    pub elf: &'a [u8],
}

/// Error enum to distinctify the error types
#[derive(Debug)]
pub enum Error {
    BadElf,
    OffsetCalculationFailure,
    UnsupportedClass,
}

/// Wrapper type for the error result
type Result<T> = core::result::Result<T, Error>;

impl<'a> core::fmt::Debug for Elf<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Display only the FileHeader not the elf file slice
        f.debug_struct("Elf")
         .field("file_header", &self.file_header)
         .finish()
    }
}

impl<'a> Elf<'a> {
    /// The default `Elf` constructor
    pub fn new(elf: &'a [u8]) -> Self {
        Elf {
            file_header: FileHeader::new(),
            elf,
        }
    }

    /// Returns the `ProgramIterator` to use in a loop or an iterator
    pub fn program_iter(&'a self) -> program::ProgramIterator {
        ProgramIterator::new(self.file_header.e_phoff, 
            self.file_header.e_phentsize, self.file_header.e_phnum, 
            self.file_header.e_class, self.file_header.e_data, self.elf)
    }

    /// Returns the `SectionIterator` to use in a loop or an iterator
    pub fn section_iter(&'a self) -> section::SectionIterator {
        SectionIterator::new(self.file_header.e_shoff, 
            self.file_header.e_shentsize, self.file_header.e_shnum, 
            self.file_header.e_class, self.file_header.e_data, self.elf)
    }

    /// Parse the elf file and populate the struct
    pub fn parse(mut self) -> Result<Self> {
        // Parse the elf header
        self.file_header = self.file_header.parse(self.elf)?;

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::println;

    #[test]
    fn parse_elf32() {
        let file =
            std::fs::read("./tests/elf_test32")
            .expect("no file was found in the test location");
        let e = Elf::new(file.as_slice());
        let e = e.parse().unwrap();
        for program in e.program_iter() {
            println!("{:#x?}", program);
        }
        for section in e.section_iter() {
            println!("{:#x?}", section);
        }
        println!("{:#x?}", e);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn parse_elf64() {
        let file =
            std::fs::read("./tests/elf_test64")
            .expect("no file was found in the test location");
        let e = Elf::new(file.as_slice());
        let e = e.parse().unwrap();
        for program in e.program_iter() {
            println!("{:#x?}", program);
        }
        for section in e.section_iter() {
            println!("{:#x?}", section);
        }
        println!("{:#x?}", e);
    }
}
