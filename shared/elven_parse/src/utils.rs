use crate::file::ElfData;

pub trait Integer: Sized {
    fn endian_parse(range: core::ops::Range<usize>,
        bytes: &[u8], e_data: &ElfData) -> crate::Result<Self>;
}

impl Integer for usize {
    fn endian_parse(range: core::ops::Range<usize>,
        bytes: &[u8], e_data: &ElfData) -> crate::Result<Self> {
        let arr = bytes.get(range)
            .ok_or(crate::Error::OffsetCalculationFailure)?
            .try_into()
            .map_err(|_err| crate::Error::OffsetCalculationFailure)?;
        Ok(match e_data {
            ElfData::ElfData2Lsb => usize::from_le_bytes(arr),
            ElfData::ElfData2Msb => usize::from_be_bytes(arr),
            ElfData::None        => usize::from_le_bytes(arr),
        })
    }
}

impl Integer for u32 {
    fn endian_parse(range: core::ops::Range<usize>,
        bytes: &[u8], e_data: &ElfData) -> crate::Result<Self> {
        let arr = bytes.get(range)
            .ok_or(crate::Error::OffsetCalculationFailure)?
            .try_into()
            .map_err(|_err| crate::Error::OffsetCalculationFailure)?;
        Ok(match e_data {
            ElfData::ElfData2Lsb => u32::from_le_bytes(arr),
            ElfData::ElfData2Msb => u32::from_be_bytes(arr),
            ElfData::None        => u32::from_le_bytes(arr),
        })
    }
}

impl Integer for u16 {
    fn endian_parse(range: core::ops::Range<usize>,
        bytes: &[u8], e_data: &ElfData) -> crate::Result<Self> {
        let arr = bytes.get(range)
            .ok_or(crate::Error::OffsetCalculationFailure)?
            .try_into()
            .map_err(|_err| crate::Error::OffsetCalculationFailure)?;
        Ok(match e_data {
            ElfData::ElfData2Lsb => u16::from_le_bytes(arr),
            ElfData::ElfData2Msb => u16::from_be_bytes(arr),
            ElfData::None        => u16::from_le_bytes(arr),
        })
    }
}
