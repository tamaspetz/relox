//! Compress ELF32 relocation sections
//!
//! This module can be used to compress ELF32 relocation sections post-link time.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::BTreeMap;
use std::io::{Cursor, Write};

use crate::error::{Error, ErrorKind};
use crate::uleb128;

// Type of a relocation.
type Elf32RelType = u8;

/// Representation of a regular ELF32 relocation.
#[derive(Debug)]
pub struct Elf32Rel {
    offset: u32,
    relocation_type: Elf32RelType,
}

impl Elf32Rel {
    /// Constructs an `Elf32Rel` instace from an in-memory buffer.
    pub fn from_memory(data: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        let offset = data
            .read_u32::<LittleEndian>()
            .map_err(|_| Error::new(ErrorKind::NotEnoughData))?;
        let info = data
            .read_u32::<LittleEndian>()
            .map_err(|_| Error::new(ErrorKind::NotEnoughData))?;
        Ok(Self {
            offset: offset,
            relocation_type: info as u8,
        })
    }

    /// Returns the offset of the relocation.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Returns the type of the relocation.
    pub fn relocation_type(&self) -> Elf32RelType {
        self.relocation_type
    }
}

/// Representation of a regular ELF32 relocation section.
pub struct Elf32Relocs<'a> {
    entries: BTreeMap<Elf32RelType, Vec<Elf32Rel>>,
    data: &'a [u8],
    base_address: u32,
}

impl<'a> Elf32Relocs<'a> {
    /// Creates a new `Elf32Relocs` instance.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            entries: BTreeMap::new(),
            data: data,
            base_address: u32::max_value(),
        }
    }

    /// Compresses this regular ELF32 relocation section and writes the
    /// compressed data to the provided in-memory buffer.
    /// Returns the number of bytes written if the compression is successful.
    pub fn compress(&mut self, output: &mut [u8]) -> Result<usize, Error> {
        self.collect_entries()?;
        let mut writer = Cursor::new(output);
        self.write_header(&mut writer)?;
        for key in self.entries.keys() {
            self.write_group(&mut writer, *key)?;
        }
        Ok(writer.position() as usize)
    }

    /// Collects relocation entries.
    fn collect_entries(&mut self) -> Result<(), Error> {
        let mut cursor = Cursor::new(self.data);
        loop {
            if let Ok(entry) = Elf32Rel::from_memory(&mut cursor) {
                if self.entries.len() == 0 {
                    self.base_address = entry.offset();
                } else if self.base_address > entry.offset() {
                    return Err(Error::new(ErrorKind::InvalidData));
                }
                if !self.entries.contains_key(&entry.relocation_type()) {
                    self.entries.insert(entry.relocation_type(), Vec::new());
                }
                self.entries
                    .get_mut(&entry.relocation_type())
                    .unwrap()
                    .push(entry);
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Writes the header.
    fn write_header(&self, writer: &mut Cursor<&mut [u8]>) -> Result<(), Error> {
        writer
            .write_u32::<LittleEndian>(self.base_address)
            .map_err(|_| Error::new(ErrorKind::BufferSmall))?;
        writer
            .write_u8(self.entries.keys().len() as u8)
            .map_err(|_| Error::new(ErrorKind::BufferSmall))?;
        Ok(())
    }

    /// Writes a group.
    fn write_group(&self, writer: &mut Cursor<&mut [u8]>, key: u8) -> Result<(), Error> {
        writer
            .write_u8(key)
            .map_err(|_| Error::new(ErrorKind::BufferSmall))?;
        let mut count: [u8; 5] = [0; 5];
        let written = uleb128::write_u32(self.entries[&key].len() as u32, &mut count)?;
        writer
            .write_all(&count[0..written])
            .map_err(|_| Error::new(ErrorKind::BufferSmall))?;
        let mut base_address = self.base_address;
        for entry in self.entries[&key].iter() {
            let written = uleb128::write_u32(entry.offset() - base_address, &mut count)?;
            writer
                .write_all(&count[0..written])
                .map_err(|_| Error::new(ErrorKind::BufferSmall))?;
            base_address = entry.offset();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn test_elf32rel_std_fmt_debug() {
        let memory: [u8; 8] = [0; 8];
        let mut cursor = Cursor::new(&memory[..]);
        let elf32rel = Elf32Rel::from_memory(&mut cursor).unwrap();
        println!("{:?}", elf32rel);
    }

    #[test]
    fn test_elf32rel_from_memory_offset_bad() {
        let memory: [u8; 3] = [0; 3];
        let mut cursor = Cursor::new(&memory[..]);
        let err = Elf32Rel::from_memory(&mut cursor).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotEnoughData);
    }

    #[test]
    fn test_elf32rel_from_memory_info_bad() {
        let memory: [u8; 7] = [0; 7];
        let mut cursor = Cursor::new(&memory[..]);
        let err = Elf32Rel::from_memory(&mut cursor).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotEnoughData);
    }

    #[test]
    fn test_elf32rel_from_memory() {
        let memory: [u8; 8] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut cursor = Cursor::new(&memory[..]);
        let rel = Elf32Rel::from_memory(&mut cursor).unwrap();
        let offset = rel.offset();
        let relocation_type = rel.relocation_type();
        assert_eq!(offset, 0x04030201);
        assert_eq!(relocation_type, 0x05);
    }

    #[test]
    fn test_elf32relocs_new() {
        let memory: [u8; 0] = [0; 0];
        let _ = Elf32Relocs::new(&memory);
    }

    #[test]
    fn test_elf32relocs_compress_header_small_base_address() {
        let memory: [u8; 0] = [0; 0];
        let mut output: [u8; 3] = [0; 3];
        let mut relocs = Elf32Relocs::new(&memory);
        let err = relocs.compress(&mut output).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::BufferSmall);
    }

    #[test]
    fn test_elf32relocs_compress_header_small_count() {
        let memory: [u8; 0] = [0; 0];
        let mut output: [u8; 4] = [0; 4];
        let mut relocs = Elf32Relocs::new(&memory);
        let err = relocs.compress(&mut output).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::BufferSmall);
    }

    #[test]
    fn test_elf32relocs_compress_header_only() {
        let memory: [u8; 0] = [0; 0];
        let mut output: [u8; 5] = [0; 5];
        let mut relocs = Elf32Relocs::new(&memory);
        let written = relocs.compress(&mut output).unwrap();
        assert_eq!(written, 5);
        assert_eq!(output[0], 0xFF);
        assert_eq!(output[1], 0xFF);
        assert_eq!(output[2], 0xFF);
        assert_eq!(output[3], 0xFF);
        assert_eq!(output[4], 0x00);
    }

    #[test]
    fn test_elf32relocs_compress_group_small_for_type() {
        let memory: [u8; 8] = [
            0x01, 0x02, 0x03, 0x04, // Elf32Rel[0], will become base address
            0x05, 0x00, 0x00, 0x00, // Type is 5
        ];
        let mut output: [u8; 5] = [0; 5];
        let mut relocs = Elf32Relocs::new(&memory);
        let err = relocs.compress(&mut output).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::BufferSmall);
    }

    #[test]
    fn test_elf32relocs_compress_group_small_for_count() {
        let memory: [u8; 8] = [
            0x01, 0x02, 0x03, 0x04, // Elf32Rel[0], will become base address
            0x05, 0x00, 0x00, 0x00, // Type is 5
        ];
        let mut output: [u8; 6] = [0; 6];
        let mut relocs = Elf32Relocs::new(&memory);
        let err = relocs.compress(&mut output).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::BufferSmall);
    }

    #[test]
    fn test_elf32relocs_compress_group_small_for_offset0() {
        let memory: [u8; 8] = [
            0x01, 0x02, 0x03, 0x04, // Elf32Rel[0], will become base address
            0x05, 0x00, 0x00, 0x00, // Type is 5
        ];
        let mut output: [u8; 7] = [0; 7];
        let mut relocs = Elf32Relocs::new(&memory);
        let err = relocs.compress(&mut output).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::BufferSmall);
    }

    #[test]
    fn test_elf32relocs_compress_offsets_not_sorted() {
        let memory: [u8; 16] = [
            0x02, 0x00, 0x00, 0x00, // Elf32Rel[0], will become base address
            0x05, 0x00, 0x00, 0x00, // Type is 5
            0x01, 0x00, 0x00, 0x00, // Elf32Rel[1]
            0x05, 0x00, 0x00, 0x00, // Type is 5
        ];
        let mut output: [u8; 128] = [0; 128];
        let mut relocs = Elf32Relocs::new(&memory);
        let err = relocs.compress(&mut output).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_elf32relocs_compress_one_group() {
        let memory: [u8; 16] = [
            0x01, 0x02, 0x03, 0x04, // Elf32Rel[0], will become base address
            0x05, 0x00, 0x00, 0x00, // Type is 5
            0x0F, 0x02, 0x03, 0x04, // Elf32Rel[1]
            0x05, 0x00, 0x00, 0x00, // Type is 5
        ];
        let mut output: [u8; 128] = [0; 128];
        let mut relocs = Elf32Relocs::new(&memory);
        let written = relocs.compress(&mut output).unwrap();
        assert_eq!(written, 9);
        // Header
        //   base_address
        assert_eq!(output[0], 0x01);
        assert_eq!(output[1], 0x02);
        assert_eq!(output[2], 0x03);
        assert_eq!(output[3], 0x04);
        //   count
        assert_eq!(output[4], 0x01);
        //   groups[0]
        //     relocation_type
        assert_eq!(output[5], 0x05);
        //     count
        assert_eq!(output[6], 0x02);
        //     offsets[0]
        assert_eq!(output[7], 0x00);
        //     offsets[1]
        assert_eq!(output[8], 0x0F - 0x01);
    }

    #[test]
    fn test_elf32relocs_compress_two_groups() {
        let memory: [u8; 24] = [
            0x01, 0x02, 0x03, 0x04, // Elf32Rel[0], will become base address
            0x05, 0x00, 0x00, 0x00, // Type is 5
            0x02, 0x02, 0x03, 0x04, // Elf32Rel[1]
            0x05, 0x00, 0x00, 0x00, // Type is 5
            0x41, 0x02, 0x03, 0x04, // Elf32Rel[2]
            0x01, 0x00, 0x00, 0x00, // Type is 1
        ];
        let mut output: [u8; 128] = [0; 128];
        let mut relocs = Elf32Relocs::new(&memory);
        let written = relocs.compress(&mut output).unwrap();
        assert_eq!(written, 12);
        // Header
        //   base_address
        assert_eq!(output[0], 0x01);
        assert_eq!(output[1], 0x02);
        assert_eq!(output[2], 0x03);
        assert_eq!(output[3], 0x04);
        //   count
        assert_eq!(output[4], 0x02);
        //   groups[0]
        //     relocation_type
        assert_eq!(output[5], 0x01);
        //     count
        assert_eq!(output[6], 0x01);
        //     offsets[0]
        assert_eq!(output[7], 0x41 - 0x01);
        //   groups[1]
        //     relocation_type
        assert_eq!(output[8], 0x05);
        //     count
        assert_eq!(output[9], 0x02);
        //     offsets[0]
        assert_eq!(output[10], 0x00);
        //     offsets[1]
        assert_eq!(output[11], 0x01);
    }
}
