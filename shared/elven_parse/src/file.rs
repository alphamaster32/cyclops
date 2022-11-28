use crate::Result;
use crate::utils::Integer;
use crate::Error;

/// Elf file header type to store the file header information
#[derive(Debug)]
pub struct FileHeader {
    /// Elf bitness
    pub e_class: ElfClass,
    /// Elf data encodings
    pub e_data: ElfData,
    /// Elf OS ABI
    pub e_abi: ElfOsAbi,
    /// Elf file type
    pub e_type: ElfType,
    /// Elf machine ISA
    pub e_machine: ElfMachine,
    /// Elf virtual address entry point
    pub e_entry: usize,
    /// Pointer to the start of the program header table
    pub e_phoff: usize,
    /// Pointer to the start of the section header table
    pub e_shoff: usize,
    /// Elf flags interpretation of this flag depends on the target 
    /// architecture
    pub e_flags: u32,
    /// Elf header size
    pub e_ehsize: u16,
    /// Elf program header table entry size
    pub e_phentsize: u16,
    /// Elf program header table entry size
    pub e_phnum: u16,
    /// Elf section header table entry size
    pub e_shentsize: u16,
    /// Elf Section header table entry count
    pub e_shnum: u16,
    /// Elf section header string table index that contains section names
    pub e_shstrndx: u16,
}

/// ElfClass specifies elf architecture and bitness
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfClass {
    None,
    Class32,
    Class64,
}

/// ElfData specifies elf data encodings
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfData {
    None,
    ElfData2Lsb,
    ElfData2Msb,
}

/// ElfOSAbi specifies OS ABI of the elf file
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfOsAbi {
    Sysv,
    Hpux,
    Netbsd,
    Gnu,
    Solaris,
    Aix,
    Irix,
    Freebsd,
    Tru64,
    Modesto,
    Openbsd,
    Armeabi,
    Arm,
    Standalone,
}

/// ElfType defines elf object type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfType {
    None,
    Relocatable,
    Executable,
    SharedObject,
    CoreFile,
    OsSpecific,
    CpuSpecific,
}

/// ElfMachine Specifies machine ISA type
/// As this might get too big we will not specify all the available machine
/// types in the libc
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElfMachine {
    None,
    Intel80386,
    Amd64,
    Riscv,
    Arm,
    Bpf,
    UnDefined,
}

impl Default for FileHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl FileHeader {
    /// The default `FileHeader` constructor
    pub fn new() -> Self {
        FileHeader {
            e_class:       ElfClass::None,
            e_data:        ElfData::None,
            e_abi:         ElfOsAbi::Standalone,
            e_type:        ElfType::None,
            e_machine:     ElfMachine::None,
            e_entry:       0,
            e_phoff:       0,
            e_shoff:       0,
            e_flags:       0,
            e_ehsize:      0,
            e_phentsize:   0,
            e_phnum:       0,
            e_shentsize:   0,
            e_shnum:       0,
            e_shstrndx:    0,
        }
    }
    /// Parse the elf header and populate the fields
    pub fn parse(mut self, elf: &[u8]) -> Result<Self> {
        // Get the elf magic number from the start of the file
        if elf.get(0x00..0x04) != Some(b"\x7FELF") {
            return Err(Error::BadElf);
        }

        // Check for the bitness of the elf file and report it back as ElfClass
        // Option type is cloned to avoid storing numbers in the .data section
        self.e_class = if elf.get(0x04) == Some(&1) {
            ElfClass::Class32
        } else if elf.get(0x04) == Some(&2) {
            ElfClass::Class64
        } else {
            ElfClass::None
        };

        #[cfg(target_pointer_width = "32")]
        if self.e_class == ElfClass::Class64 || 
            self.e_class == ElfClass::None {
                return Err(Error::UnsupportedClass)
        }

        // Check the data encoding of the elf file
        self.e_data = if elf.get(0x05) == Some(&1) {
            ElfData::ElfData2Lsb
        } else if elf.get(0x05) == Some(&2) {
            ElfData::ElfData2Msb
        } else {
            ElfData::None
        };

        // Check for the elf version currently this value must be 1
        if elf.get(0x06) != Some(&1) {
            return Err(Error::BadElf);
        }

        // Check for the OS ABI
        self.e_abi = match elf.get(0x07) {
            Some(e) => match e {
                0   => ElfOsAbi::Sysv,
                1   => ElfOsAbi::Hpux,
                2   => ElfOsAbi::Netbsd,
                3   => ElfOsAbi::Gnu,
                6   => ElfOsAbi::Solaris,
                8   => ElfOsAbi::Aix,
                9   => ElfOsAbi::Irix,
                10  => ElfOsAbi::Freebsd,
                11  => ElfOsAbi::Tru64,
                12  => ElfOsAbi::Modesto,
                64  => ElfOsAbi::Armeabi,
                97  => ElfOsAbi::Arm,
                255 => ElfOsAbi::Standalone,
                _ => ElfOsAbi::Sysv,
            },
            None => {
                return Err(Error::BadElf);
            }
        };

        // Get the abi version but discard it for future
        let _e_abi_ver = elf.get(0x08).unwrap_or(&0);

        // Discard the padding byte at 0x09 to 0x10
        let _padding = elf.get(0x09..0x10).unwrap_or(&[0u8; 7]);

        // Identify the elf type
        self.e_type = match elf.get(0x10..0x12) {
            Some(&[0x00, 0x00]) => ElfType::None,
            Some(&[0x01, 0x00]) => ElfType::Relocatable,
            Some(&[0x02, 0x00]) => ElfType::Executable,
            Some(&[0x03, 0x00]) => ElfType::SharedObject,
            Some(&[0x04, 0x00]) => ElfType::CoreFile,
            Some(&[0xfe, _])    => ElfType::OsSpecific,
            Some(&[0xff, _])    => ElfType::CpuSpecific,
            _ => ElfType::None,
        };

        // Get the target ISA
        self.e_machine = match elf.get(0x12..0x14) {
            Some(&[0x03, 0x00]) => ElfMachine::Intel80386,
            Some(&[0x3e, 0x00]) => ElfMachine::Amd64,
            Some(&[0x28, 0x00]) => ElfMachine::Arm,
            Some(&[0xf3, 0x00]) => ElfMachine::Riscv,
            Some(&[0xf7, 0x00]) => ElfMachine::Bpf,
            _ => ElfMachine::UnDefined,
        };

        // Check for the elf version for another time apparently this has more
        // options like none and current version which is 1
        if elf.get(0x14..0x18) != Some(&[0x01, 0x00, 0x00, 0x00]) {
            return Err(Error::BadElf);
        }

        //Now we need the parser position in file
        let mut pos: usize = 0x18;

        // Calculate the next header part ending
        let mut next: usize = match self.e_class {
            ElfClass::Class64 => 0x08 + pos,
            ElfClass::Class32 => 0x04 + pos,
            ElfClass::None    => 0x08 + pos,
        };

        // This section depends on the elf file class so we branch out
        if self.e_class == ElfClass::Class64 || 
            self.e_class == ElfClass::None {
            // Get the elf virtual address entry point
            self.e_entry = usize::endian_parse(pos..next,
                elf, &self.e_data)?;

            // Move the position to the new header part
            pos = next;

            // Calculate the next header part ending
            next = pos + 0x08;
            
            // Get the elf program header offset
            self.e_phoff = usize::endian_parse(pos..next,
                elf, &self.e_data)?;

            // Move the position to the new header part
            pos = next;

            // Calculate the next header part ending
            next = pos + 0x08;

            // Get the elf section header offset
            self.e_shoff = usize::endian_parse(pos..next,
                elf, &self.e_data)?;

        } else if self.e_class == ElfClass::Class32 {
            // Get the elf virtual address entry point
            self.e_entry = u32::endian_parse(pos..next,
                elf, &self.e_data)? as usize;


            // Move the position to the new header part
            pos = next;

            // Calculate the next header part ending
            next = pos + 0x04;

            // Get the elf program header offset
            self.e_phoff = u32::endian_parse(pos..next,
                elf, &self.e_data)? as usize;

            // Move the position to the new header part
            pos  = next;

            // Calculate the next header part ending
            next = pos + 0x04;

            // Get the elf section header offset
            self.e_shoff = u32::endian_parse(pos..next,
                elf, &self.e_data)? as usize;
        }

        // Move the position to the new header part
        pos = next;

        // Get the elf processor specific flags
        self.e_flags = u32::endian_parse(pos..(pos + 0x04),
            elf, &self.e_data)?;

        // Move the position to the new header part
        pos += 0x04;
        self.e_ehsize = u16::endian_parse(pos..(pos + 0x02),
            elf, &self.e_data)?;

        // Move the position to the new header part
        pos += 0x02;

        // Get the elf program header entry size
        self.e_phentsize = u16::endian_parse(pos..(pos + 0x02),
            elf, &self.e_data)?;

        // Move the position to the new header part
        pos += 0x02;

        // Get the elf program header entry size
        self.e_phnum = u16::endian_parse(pos..(pos + 0x02),
            elf, &self.e_data)?;

        // Move the position to the new header part
        pos += 0x02;

        // Get the elf section header table entry size
        self.e_shentsize = u16::endian_parse(pos..(pos + 0x02),
            elf, &self.e_data)?;

        // Move the position to the new header part
        pos += 0x02;

        // Get the elf section header table entry count
        self.e_shnum = u16::endian_parse(pos..(pos + 0x02),
            elf, &self.e_data)?;

        // Move the position to the new header part
        let pos: usize = pos + 0x02;

        // Get the elf section header string table
        self.e_shstrndx = u16::endian_parse(pos..(pos + 0x02),
            elf, &self.e_data)?;

        Ok(self)
    }
}
