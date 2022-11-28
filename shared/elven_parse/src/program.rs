use crate::Result;
use crate::utils::Integer;
use crate::file::{ElfData, ElfClass};

pub const PF_X: usize = 1 << 0;
pub const PF_W: usize = 1 << 1;
pub const PF_R: usize = 1 << 2;

/// ProgramHeader stores information regarding to how the image sections
/// should be laid out in the system memory
#[derive(Debug, Copy, Clone)]
pub struct ProgramHeader {
    /// Identifies the type of the segment
    pub p_type: ProgramType,
    /// Segment flags
    pub p_flags: Perm,
    /// Offset of the segment in the file
    pub p_offset: usize,
    /// Virtual address of the segment in memory
    pub p_vaddr: usize,
    /// Reserved for the physical address in the memory
    pub p_paddr: usize,
    /// Size of the segment in bytes
    pub p_filesz: usize,
    /// Size of the segment mapped in memory in bytes
    pub p_memsz: usize,
    /// Specifies alignment and should be integral power of 2 (1 and 0 are no
    /// alignment)
    pub p_align: usize,
}

/// Enum to identify the program header type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProgramType {
    None,
    /// Program header entry is unused
    PtNull,
    /// Loadable program segment
    PtLoad,
    /// Dynamic linking information
    PtDynamic,
    /// Program interpreter
    PtInterp,
    /// Auxiliary information
    PtNote,
    /// Reserved (not sure for what)
    PtShlib,
    /// Entry for the header table itself
    PtPhdr,
    /// Thread-local storage segment
    PtTls,
    /// GCC .eh_frame_hdr segment
    PtGnuEhFrame,
    /// Indicates stack executability
    PtGnuStack,
    /// Read-only after relocation
    PtGnuRelro,
    /// GNU property
    PtGnuProperty,
    /// OS specific segment
    PtOs,
    /// Processor specific segment
    PtProc,
}

/// struct to represent RWX perms on the program header
/// The three booleans represented in the struct are Read, Write, Execute 
/// in order
/// It is best that associated functions be used when using this struct
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Perm(pub bool, pub bool, pub bool);

/// Helper type to implement the iterator type on
/// The best is for the `program_iter()` function to be called
/// on the elf struct
#[derive(Debug, Clone, Copy)]
pub struct ProgramIterator<'a> {
    /// Owned program header struct to address
    program_header: ProgramHeader,
    /// Program header offset in the elf file
    offset: usize,
    /// Program header entry size
    phentsize: u16,
    /// Number of program header entries also used as 
    /// the index of the iteration
    phnum: u16,
    /// Elf class used for parsing
    class: ElfClass,
    /// Elf endianness used for parsing
    data: ElfData,
    /// A reference to the elf file
    elf: &'a [u8],
}


impl Default for ProgramHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramHeader {
    /// The default `ProgramHeader` constructor
    pub fn new() -> Self {
        ProgramHeader {
            p_type:   ProgramType::None,
            p_flags:  Perm(false, false, false),
            p_offset: 0,
            p_vaddr:  0,
            p_paddr:  0,
            p_filesz: 0,
            p_memsz:  0,
            p_align:  0,
        }
    }

    /// Parse the `ProgramHeader` and populate the fields
    pub fn parse(mut self, elf: &[u8], 
        class: ElfClass, data: ElfData) -> Result<Self> {
        // Get the segment type
        self.p_type = match elf.get(0x00..0x04) {
            Some(&[0x00, 0x00, 0x00, 0x00]) => ProgramType::PtNull,
            Some(&[0x01, 0x00, 0x00, 0x00]) => ProgramType::PtLoad,
            Some(&[0x02, 0x00, 0x00, 0x00]) => ProgramType::PtDynamic,
            Some(&[0x03, 0x00, 0x00, 0x00]) => ProgramType::PtInterp,
            Some(&[0x04, 0x00, 0x00, 0x00]) => ProgramType::PtNote,
            Some(&[0x05, 0x00, 0x00, 0x00]) => ProgramType::PtShlib,
            Some(&[0x06, 0x00, 0x00, 0x00]) => ProgramType::PtPhdr,
            Some(&[0x50, 0xe5, 0x74, 0x64]) => ProgramType::PtGnuEhFrame,
            Some(&[0x51, 0xe5, 0x74, 0x64]) => ProgramType::PtGnuStack,
            Some(&[0x52, 0xe5, 0x74, 0x64]) => ProgramType::PtGnuRelro,
            Some(&[0x53, 0xe5, 0x74, 0x64]) => ProgramType::PtGnuProperty,
            Some(&[_, _, _, 0x60..=0x6f])   => ProgramType::PtOs,
            Some(&[_, _, _, 0x70..=0x7f])   => ProgramType::PtProc,
            _ => ProgramType::None,
        };

        // Branch and parse according to the elf architecture class
        if class == ElfClass::Class32 {
            // Get the program offset of the segment
            self.p_offset = 
                u32::endian_parse(0x04..0x08, elf, &data)? as usize;

            // Get the program offset of the segment in the virtual memory
            self.p_vaddr= u32::endian_parse(0x08..0x0c, elf, &data)? as usize;

            // Get the program offset of the segment in the physical memory
            // This part is only relevant in systems without which use 
            // memory segmentation
            self.p_paddr = u32::endian_parse(0x0c..0x10, elf, &data)? as usize;

            // Size of the file image segment in bytes
            self.p_filesz = 
                u32::endian_parse(0x10..0x14, elf, &data)? as usize;

            // Size of the segment mapped in the memory in bytes
            self.p_memsz = u32::endian_parse(0x14..0x18, elf, &data)? as usize;

            // Get the memory permissions of the segment
            let flags = u32::endian_parse(0x18..0x1c, elf, &data)? as usize;
            self.p_flags.2 = flags as usize & PF_X != 0;
            self.p_flags.1 = flags as usize & PF_W != 0;
            self.p_flags.0 = flags as usize & PF_R != 0;

            // Specifies alignment
            // 0 and 1 specify no alignment otherwise it should be integral
            // power of 2
            self.p_align = u32::endian_parse(0x1c..0x20, elf, &data)? as usize;
        } else if class == ElfClass::Class64 {
            // Get the memory permissions of the segment
            let flags = u32::endian_parse(0x04..0x08, elf, &data)?;
            self.p_flags.2 = flags as usize & PF_X != 0;
            self.p_flags.1 = flags as usize & PF_W != 0;
            self.p_flags.0 = flags as usize & PF_R != 0;

            // Get the program offset of the segment in the file image
            self.p_offset = usize::endian_parse(0x08..0x10, elf, &data)?;

            // Get the program offset of the segment in the virtual memory
            self.p_vaddr= usize::endian_parse(0x10..0x18, elf, &data)?;

            // Get the program offset of the segment in the physical memory
            // This part is only relevant in systems without which use 
            // memory segmentation
            self.p_paddr = usize::endian_parse(0x18..0x20, elf, &data)?;

            // Size of the file image segment in bytes
            self.p_filesz = usize::endian_parse(0x20..0x28, elf, &data)?;

            // Size of the segment mapped in the memory in bytes
            self.p_memsz = usize::endian_parse(0x28..0x30, elf, &data)?;

            // Specifies alignment
            // 0 and 1 specify no alignment otherwise it should be integral
            // power of 2
            self.p_align = usize::endian_parse(0x30..0x38, elf, &data)?;
        }

        Ok(self)
    }
}

impl Perm {
    /// Return if the section is readable
    pub fn is_read(self) -> bool {
        self.0
    }

    /// Return if the section is writable
    pub fn is_write(self) -> bool {
        self.1
    }

    /// Return if the section is executable
    pub fn is_exec(self) -> bool {
        self.2
    }
}

impl<'a> Iterator for ProgramIterator<'a> {
    type Item = ProgramHeader;
    fn next(&mut self) -> Option<Self::Item> {
        // If the number of program headers is zero then abort the iterator
        if self.phnum == 0 {
            None
        } else {
            // Parse the program header into the struct
            self.program_header = 
                self.program_header.parse(
                    &self.elf[self.offset..self.offset + 
                    self.phentsize as usize],
                    self.class, self.data).ok()?;

            // Calculate the next offset for the next program header
            self.offset += self.phentsize as usize;

            // Subtract one from the number of the program headers
            self.phnum -= 1;

            Some(self.program_header)
        }
    }
}

impl<'a> ProgramIterator<'a> {
    pub fn new(e_phoff: usize, e_phentsize: u16, e_phnum: u16, 
        class: ElfClass, data: ElfData, elf: &'a [u8]) -> Self {
        // Construct a empty program header for the program iterator
        let program = ProgramHeader::new();

        ProgramIterator {
            program_header: program,
            offset: e_phoff,
            phentsize: e_phentsize,
            phnum: e_phnum,
            class,
            data,
            elf,
        }
    }
}
