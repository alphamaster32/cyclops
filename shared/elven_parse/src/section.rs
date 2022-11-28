use crate::Result;
use crate::utils::Integer;
use crate::file::{ElfData, ElfClass};

/// Writable
pub const SHF_WRITE: u32 = 1 << 0;
/// Occupies memory during execution
pub const SHF_ALLOC: u32 = 1 << 1;
/// Executable
pub const SHF_EXECINSTR: u32 = 1 << 2;
/// Might be merged?
pub const SHF_MERGE: u32 = 1 << 4;
/// Contains nul-terminated strings
pub const SHF_STRINGS: u32 = 1 << 5;
/// `sh_info` contains SHT index
pub const SHF_INFO_LINK: u32 = 1 << 6;
/// Preserve order after combining
pub const SHF_LINK_ORDER: u32 = 1 << 7;
/// Non-standard OS specific handling
pub const SHF_OS_NONCONFORMING: u32 = 1 << 8;
/// Section is member of a group
pub const SHF_GROUP: u32 = 1 << 9;
/// Section hold thread-local data
pub const SHF_TLS: u32 = 1 << 10;
/// Section with compressed data
pub const SHF_COMPRESSED: u32 = 1 << 11;
/// OS-specific
pub const SHF_MASKOS: u32 = 0x0ff00000;
/// Processor-specific
pub const SHF_MASKPROC: u32 = 0xf0000000;
/// Not to be GCed by linker
pub const SHF_GNU_RETAIN: u32 = 1 << 21;
/// Special ordering requirement
pub const SHF_ORDERED: u32 = 1 << 30;
/// Section is excluded unless referenced or allocated (Solaris)
pub const SHF_EXCLUDE: u32 = 1 << 31;

/// Section header stores data about the sections of the elf file
#[derive(Debug, Copy, Clone)]
pub struct SectionHeader {
    /// Identifies section name as indexes which is an offset of shstrtab
    pub sh_name: u32,
    /// Section type
    pub sh_type: SectionType,
    /// Section flags describe sections permissions in memory
    pub sh_flags: SectionFlags,
    /// Section virtual address at the execution
    pub sh_addr: usize,
    /// Section offset at the file image
    pub sh_offset: usize,
    /// Section size in bytes
    pub sh_size: usize,
    /// Contains the section index of the associated section and info depends
    /// on the `sh_type`
    pub sh_link: u32,
    /// Additional section information which also depends on the `sh_type`
    pub sh_info: u32,
    /// Section alignment which must be a power of two
    pub sh_addralign: usize,
    /// Entry size in bytes for fixed size sections otherwise it is zero
    pub sh_entsize: usize,
}

/// Enum to identify section types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SectionType {
    None,
    /// Section Table entry unused
    ShtNull,
    /// Program data
    ShtProgBits,
    /// Symbol table
    ShtSymTab,
    /// String table
    ShtStrTab,
    /// Relocation entries with addends
    ShtRela,
    /// Symbol hash table
    ShtHash,
    /// Dynamic linking information
    ShtDynamic,
    /// Notes
    ShtNotes,
    /// Program space with no data (bss)
    ShtNoBits,
    /// Relocation entries no addends
    ShtRel,
    /// Reserved
    ShtShlib,
    /// Dynamic linker symbol table
    ShtDynSym,
    /// Array of constructors
    ShtInitArray,
    /// Array of destructors
    ShtFInitArray,
    /// Array of predestructors
    ShtPreInitArray,
    /// Section group
    ShtGroup,
    /// Extended section indices
    ShtSymTabShndx,
    /// RELR relative relocations
    ShtRelr,
    /// Number of defined types
    ShtNum,
    /// Os Specific
    ShtOs,
    /// Gnu object attributes
    ShtGnuAttributes,
    /// Gnu style hash tables
    ShtGnuHash,
    /// Gnu prelink library list
    ShtGnuLibList,
    /// Processor specific sections
    ShtProc,
    /// Application specific sections
    ShtUser,
}

/// SectionFlags tuple struct to implement some is_* functions on
#[derive(Debug, Copy, Clone)]
pub struct SectionFlags(usize);

/// Helper type to implement the iterator type on
/// The best is for the `section_iter()` function to be called
/// on the elf struct
#[derive(Debug, Clone, Copy)]
pub struct SectionIterator<'a> {
    /// Owned section header struct to address
    section_header: SectionHeader,
    /// Section header offset in the elf file
    offset: usize,
    /// Section header entry size
    shentsize: u16,
    /// Number of header entries also used as 
    /// the index of the iteration
    shnum: u16,
    /// Elf class used for parsing
    class: ElfClass,
    /// Elf endianness used for parsing
    data: ElfData,
    /// A reference to the elf file
    elf: &'a [u8],
}


impl Default for SectionHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl SectionHeader {
    /// The default `SectionHeader` constructor
    pub fn new() -> Self {
        SectionHeader {
            sh_name:      0,
            sh_type:      SectionType::None,
            sh_flags:     SectionFlags(0),
            sh_addr:      0,
            sh_offset:    0,
            sh_size:      0,
            sh_link:      0,
            sh_info:      0,
            sh_addralign: 0,
            sh_entsize:   0,
        }
    }

    /// Parse the program header and populate the fields
    pub fn parse(mut self, elf: &[u8], 
        class: ElfClass, data: ElfData) -> Result<Self> {
        // Get the pointer to the name of section
        self.sh_name = u32::endian_parse(0x00..0x04, elf, &data)?;

        // Get the section type
        self.sh_type = match elf.get(0x04..0x08) {
            Some(&[0x00, 0x00, 0x00, 0x00]) => SectionType::ShtNull,
            Some(&[0x01, 0x00, 0x00, 0x00]) => SectionType::ShtProgBits,
            Some(&[0x02, 0x00, 0x00, 0x00]) => SectionType::ShtSymTab,
            Some(&[0x03, 0x00, 0x00, 0x00]) => SectionType::ShtStrTab,
            Some(&[0x04, 0x00, 0x00, 0x00]) => SectionType::ShtRela,
            Some(&[0x05, 0x00, 0x00, 0x00]) => SectionType::ShtHash,
            Some(&[0x06, 0x00, 0x00, 0x00]) => SectionType::ShtDynamic,
            Some(&[0x07, 0x00, 0x00, 0x00]) => SectionType::ShtNotes,
            Some(&[0x08, 0x00, 0x00, 0x00]) => SectionType::ShtNoBits,
            Some(&[0x09, 0x00, 0x00, 0x00]) => SectionType::ShtRel,
            Some(&[0x0a, 0x00, 0x00, 0x00]) => SectionType::ShtShlib,
            Some(&[0x0b, 0x00, 0x00, 0x00]) => SectionType::ShtDynSym,
            Some(&[0x0e, 0x00, 0x00, 0x00]) => SectionType::ShtInitArray,
            Some(&[0x0f, 0x00, 0x00, 0x00]) => SectionType::ShtFInitArray,
            Some(&[0x10, 0x00, 0x00, 0x00]) => SectionType::ShtPreInitArray,
            Some(&[0x11, 0x00, 0x00, 0x00]) => SectionType::ShtGroup,
            Some(&[0x12, 0x00, 0x00, 0x00]) => SectionType::ShtSymTabShndx,
            Some(&[0x13, 0x00, 0x00, 0x00]) => SectionType::ShtRelr,
            Some(&[0x14, 0x00, 0x00, 0x00]) => SectionType::ShtNum,
            Some(&[0xf5, 0xff, 0xff, 0x06]) => SectionType::ShtGnuAttributes,
            Some(&[0xf6, 0xff, 0xff, 0x06]) => SectionType::ShtGnuHash,
            Some(&[0xf7, 0xff, 0xff, 0x06]) => SectionType::ShtGnuLibList,
            Some(&[_, _, _, 0x06])          => SectionType::ShtOs,
            Some(&[_, _, _, 0x07])          => SectionType::ShtProc,
            Some(&[_, _, _, 0x08])          => SectionType::ShtUser,
            _ => SectionType::None,
        };

        // Branch and parse according to the elf architecture class
        if class == ElfClass::Class32 {
            // Get the section flags
            self.sh_flags.0 = u32::endian_parse(0x08..0x0c, elf, &data)? 
                as usize;

            // Get the virtual address of the section in memory
            self.sh_addr = u32::endian_parse(0x0c..0x10, elf, &data)? as usize;

            // Get the offset of the section in elf file image
            self.sh_offset = u32::endian_parse(0x10..0x14, elf, &data)? 
                as usize;

            // Get the size of the section in the elf file image in bytes
            self.sh_size = u32::endian_parse(0x14..0x18, elf, &data)? as usize;

            // Get the section index
            self.sh_link = u32::endian_parse(0x18..0x1c, elf, &data)?;

            // Get the section extra info
            self.sh_info = u32::endian_parse(0x1c..0x20, elf, &data)?;

            // Get the section alignment
            self.sh_addralign = u32::endian_parse(0x20..0x24, elf, &data)? 
                as usize;

            // Get the entry size in bytes for sections that contain entries
            self.sh_entsize = u32::endian_parse(0x24..0x28, elf, &data)? 
                as usize;
        } else if class == ElfClass::Class64 {
            // Get the section flags
            self.sh_flags.0 = usize::endian_parse(0x08..0x10, elf, &data)?;

            // Get the virtual address of the section in memory
            self.sh_addr = usize::endian_parse(0x10..0x18, elf, &data)?;

            // Get the offset of the section in elf file image
            self.sh_offset = usize::endian_parse(0x18..0x20, elf, &data)?;

            // Get the size of the section in the elf file image in bytes
            self.sh_size = usize::endian_parse(0x20..0x28, elf, &data)?;

            // Get the section index
            self.sh_link = u32::endian_parse(0x28..0x2c, elf, &data)?;

            // Get the section extra info
            self.sh_info = u32::endian_parse(0x2c..0x30, elf, &data)?;

            // Get the section alignment
            self.sh_addralign = usize::endian_parse(0x30..0x38, elf, &data)?;

            // Get the entry size in bytes for sections that contain entries
            self.sh_entsize = usize::endian_parse(0x38..0x40, elf, &data)?;
        }

        Ok(self)
    }
}

impl SectionFlags {
    pub fn is_write(self) -> bool {
        self.0 & SHF_WRITE as usize == SHF_WRITE as usize
    }
    pub fn is_alloc(self) -> bool {
        self.0 & SHF_ALLOC as usize == SHF_ALLOC as usize
    }
    pub fn is_exec(self) -> bool {
        self.0 & SHF_EXECINSTR as usize == SHF_EXECINSTR as usize
    }
    pub fn is_merge(self) -> bool {
        self.0 & SHF_MERGE as usize == SHF_MERGE as usize
    }
    pub fn is_strings(self) -> bool {
        self.0 & SHF_STRINGS as usize == SHF_STRINGS as usize
    }
    pub fn is_info_link(self) -> bool {
        self.0 & SHF_INFO_LINK as usize == SHF_INFO_LINK as usize
    }
    pub fn is_link_order(self) -> bool {
        self.0 & SHF_LINK_ORDER as usize == SHF_LINK_ORDER as usize
    }
    pub fn is_os_nonconforming(self) -> bool {
        self.0 & SHF_OS_NONCONFORMING as usize == SHF_OS_NONCONFORMING as usize
    }
    pub fn is_group(self) -> bool {
        self.0 & SHF_GROUP as usize == SHF_GROUP as usize
    }
    pub fn is_tls(self) -> bool {
        self.0 & SHF_TLS as usize == SHF_TLS as usize
    }
    pub fn is_compressed(self) -> bool {
        self.0 & SHF_COMPRESSED as usize == SHF_COMPRESSED as usize
    }
}

impl<'a> Iterator for SectionIterator<'a> {
    type Item = SectionHeader;
    fn next(&mut self) -> Option<Self::Item> {
        // If the number of section headers is zero then abort the iterator
        if self.shnum == 0 {
            None
        } else {
            // Parse the section header into the struct
            self.section_header = 
                self.section_header.parse(
                    &self.elf[self.offset..self.offset + 
                    self.shentsize as usize],
                    self.class, self.data).ok()?;

            // Calculate the next offset for the next program header
            self.offset += self.shentsize as usize;

            // Subtract one from the number of the program headers
            self.shnum -= 1;

            Some(self.section_header)
        }
    }
}

impl<'a> SectionIterator<'a> {
    pub fn new(e_shoff: usize, e_shentsize: u16, e_shnum: u16,
        class: ElfClass, data: ElfData, elf: &'a [u8]) -> Self {
        // Construct a empty section header for the program iterator
        let section = SectionHeader::new();

        SectionIterator {
            section_header: section,
            offset: e_shoff,
            shentsize: e_shentsize,
            shnum: e_shnum,
            class,
            data,
            elf,
        }
    }
}
