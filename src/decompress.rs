//! Decompress a compressed ELF32 relocation section
//!
//! This module can be used to decompress a compressed ELF32 relocation section.

use crate::error::{Error, ErrorKind};
use crate::uleb128;

/// Processes a compressed ELF32 relocation section and calls `op` for every
/// relocation for further processing.
///
/// # Errors
///
/// If the compressed relocation section is malformed.
///
/// # Panics
///
/// If the provided data is too small for any reason and `no_bounds_check`
/// feature is not requested.
pub fn elf32_relocate<F>(data: &[u8], op: &mut F) -> Result<usize, Error>
where
    F: FnMut(u8, u32) -> Result<(), Error>,
{
    let base_address = read_u32_np(data)?;
    let mut count = slice_read_u8(data, 4)?;
    let mut index = 5;
    while count > 0 {
        index += elf32_relocate_group(array_from_slice_u8(data, index)?, base_address, op)?;
        count -= 1;
    }
    Ok(index)
}

/// Processes a single compressed relocation group.
fn elf32_relocate_group<F>(data: &[u8], mut address: u32, op: &mut F) -> Result<usize, Error>
where
    F: FnMut(u8, u32) -> Result<(), Error>,
{
    let relocation_type = slice_read_u8(data, 0)?;
    let mut index = 1;
    let mut count = 0;
    index += uleb128::read_u32(array_from_slice_u8(data, 1)?, &mut count)?;
    while count > 0 {
        let mut offset = 0;
        index += uleb128::read_u32(array_from_slice_u8(data, index)?, &mut offset)?;
        address += offset;
        op(relocation_type, address)?;
        count -= 1;
    }
    Ok(index)
}

/// Reads an unsigned u32 value without panicing.
fn read_u32_np(data: &[u8]) -> Result<u32, Error> {
    if cfg!(feature = "no_bounds_check") || data.len() >= 4 {
        Ok(unsafe { core::ptr::read(data.as_ptr() as *const u32) })
    } else {
        Err(Error::new(ErrorKind::NotEnoughData))
    }
}

/// Reads an unsigned 8-bit value from a byte slice without panicing.
fn slice_read_u8(data: &[u8], index: usize) -> Result<u8, Error> {
    if cfg!(feature = "no_bounds_check") || data.len() > index {
        Ok(unsafe { *data.get_unchecked(index) })
    } else {
        Err(Error::new(ErrorKind::NotEnoughData))
    }
}

/// Creates a sub-slice with nonzero length from a slice without panicing.
fn array_from_slice_u8<'a>(data: &'a [u8], offset: usize) -> Result<&'a [u8], Error> {
    if cfg!(feature = "no_bounds_check") || data.len() > offset {
        Ok(unsafe { core::slice::from_raw_parts(data.as_ptr().add(offset), data.len() - offset) })
    } else {
        Err(Error::new(ErrorKind::NotEnoughData))
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused)]
    use super::*;

    #[cfg(not(feature = "no_bounds_check"))]
    #[test]
    fn test_decompress_no_data() {
        elf32_relocate(&[0; 0], &mut |_, _| unreachable!()).unwrap_err();
    }

    #[cfg(not(feature = "no_bounds_check"))]
    #[test]
    fn test_decompress_base_address_only() {
        elf32_relocate(&[0; 4], &mut |_, _| unreachable!()).unwrap_err();
    }

    #[cfg(not(feature = "no_bounds_check"))]
    #[test]
    fn test_decompress_count_only() {
        elf32_relocate(&[1; 5], &mut |_, _| unreachable!()).unwrap_err();
    }

    #[cfg(not(feature = "no_bounds_check"))]
    #[test]
    fn test_decompress_count_is_zero() {
        elf32_relocate(&[0; 5], &mut |_, _| unreachable!()).unwrap();
    }

    #[cfg(not(feature = "no_bounds_check"))]
    #[test]
    fn test_decompress_group_reloc_type_no_data() {
        elf32_relocate(&[1; 6], &mut |_, _| unreachable!()).unwrap_err();
    }

    #[cfg(not(feature = "no_bounds_check"))]
    #[test]
    fn test_decompress_group_count_no_data() {
        elf32_relocate(&[1; 6], &mut |_, _| unreachable!()).unwrap_err();
    }

    #[cfg(not(feature = "no_bounds_check"))]
    #[test]
    fn test_decompress_group_offset_no_data() {
        elf32_relocate(&[1; 7], &mut |_, _| unreachable!()).unwrap_err();
    }

    #[test]
    fn test_decompress_relocate_one() {
        let memory = [
            0x04, 0x03, 0x02, 0x01, // base_address
            0x01, // count
            0x01, // group[0].relocation_type
            0x01, // group[0].count
            0x00, // group[0].offsets[0]
        ];
        let read = elf32_relocate(&memory, &mut |relocation_type, address| {
            assert_eq!(relocation_type, 0x01);
            assert_eq!(address, 0x01020304);
            Ok(())
        })
        .unwrap();
        assert_eq!(read, 8);
    }
}
