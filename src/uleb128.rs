///! Unsigned LEB128 encoding
///!
///! https://en.wikipedia.org/wiki/LEB128
use crate::error::{Error, ErrorKind};

const CONTINUE_BIT: u8 = 0x80;

/// Writes an unsigned value as ULEB128 into a buffer
/// and returns the number of bytes written.
///
/// # Errors
///
/// If the provided buffer is smaller than required.
fn write_unsigned(mut value: u32, bytes: &mut [u8]) -> Result<usize, Error> {
    let mut split = (value & 0x7F) as u8;
    for (index, byte) in bytes.iter_mut().enumerate() {
        value = value.wrapping_shr(7);
        if value > 0 {
            // Write byte with continuation bit set.
            *byte = split | CONTINUE_BIT;
            split = (value & 0x7F) as u8;
        } else {
            // Store last byte.
            *byte = split;
            return Ok(index + 1);
        }
    }
    Err(Error::new(ErrorKind::NotEnoughData))
}

/// Writes an unsigned 8-bit value as ULEB128 into a buffer
/// and returns the number of bytes written.
///
/// # Errors
///
/// If the provided buffer is smaller than required.
#[allow(unused)]
pub fn write_u8(value: u8, bytes: &mut [u8]) -> Result<usize, Error> {
    write_unsigned(value as u32, bytes)
}

/// Writes an unsigned 16-bit value as ULEB128 into a buffer
/// and returns the number of bytes written.
///
/// # Errors
///
/// If the provided buffer is smaller than required.
#[allow(unused)]
pub fn write_u16(value: u16, bytes: &mut [u8]) -> Result<usize, Error> {
    write_unsigned(value as u32, bytes)
}

/// Writes an unsigned 32-bit value as ULEB128 into a buffer
/// and returns the number of bytes written.
///
/// # Errors
///
/// If the provided buffer is smaller than required.
#[allow(unused)]
pub fn write_u32(value: u32, bytes: &mut [u8]) -> Result<usize, Error> {
    write_unsigned(value, bytes)
}

/// Returns an unsigned value deccoded from ULEB128 from a buffer and
/// the number of bytes read.
///
/// # Errors
///
/// If the provided buffer is smaller than required or if the decoded value is
/// greater than the max value of the expected type.
fn read_unsigned(
    bytes: &[u8],
    last_split_max: u32,
    shift_max: u32,
    value: &mut u32,
) -> Result<usize, Error> {
    let mut shift: u32 = 0;
    for (index, byte) in bytes.iter().enumerate() {
        let split: u32 = (byte & !CONTINUE_BIT) as u32;
        if !cfg!(feature = "no_sanity_check") && (shift == shift_max) && (split > last_split_max) {
            return Err(Error::new(ErrorKind::InvalidData));
        } else {
            *value |= split.wrapping_shl(shift);
            if (byte & CONTINUE_BIT) == CONTINUE_BIT {
                shift += 7;
                if !cfg!(feature = "no_sanity_check") && (shift > shift_max) {
                    return Err(Error::new(ErrorKind::InvalidData));
                }
            } else {
                return Ok(index + 1);
            }
        }
    }
    Err(Error::new(ErrorKind::NotEnoughData))
}

/// Returns an unsigned 8-bit value deccoded from ULEB128 from a buffer
/// and the number of bytes read.
///
/// # Errors
///
/// If the provided buffer is smaller than required or if the decoded value is
/// greater than the max value of the expected type.
#[allow(unused)]
pub fn read_u8(bytes: &[u8], value: &mut u8) -> Result<usize, Error> {
    let mut tmp: u32 = 0;
    let result = read_unsigned(bytes, 0x01, 7, &mut tmp);
    if result.is_ok() {
        *value = tmp as u8;
    }
    result
}

/// Returns an unsigned 16-bit value deccoded from ULEB128 from a buffer
/// and the number of bytes read.
///
/// # Errors
///
/// If the provided buffer is smaller than required or if the decoded value is
/// greater than the max value of the expected type.
#[allow(unused)]
pub fn read_u16(bytes: &[u8], value: &mut u16) -> Result<usize, Error> {
    let mut tmp: u32 = 0;
    let result = read_unsigned(bytes, 0x03, 14, &mut tmp);
    if result.is_ok() {
        *value = tmp as u16;
    }
    result
}

/// Returns an unsigned 32-bit value deccoded from ULEB128 from a buffer
/// and the number of bytes read.
///
/// # Errors
///
/// If the provided buffer is smaller than required or if the decoded value is
/// greater than the max value of the expected type.
#[allow(unused)]
pub fn read_u32(bytes: &[u8], value: &mut u32) -> Result<usize, Error> {
    *value = 0;
    read_unsigned(bytes, 0x0F, 28, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_write_u8() {
        let mut buffer: [u8; 2] = [0; 2];

        // 1 byte
        assert_eq!(write_u8(0, &mut buffer[0..0]).is_err(), true);

        assert_eq!(write_u8(0, &mut buffer).unwrap(), 1);
        assert_eq!(buffer[0], 0);

        assert_eq!(write_u8(0x7F, &mut buffer).unwrap(), 1);
        assert_eq!(buffer[0], 0x7F);

        // 2 bytes
        assert_eq!(write_u8(0x80, &mut buffer[0..1]).is_err(), true);

        assert_eq!(write_u8(CONTINUE_BIT, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x01);

        assert_eq!(write_u8(0xFF, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x01);
    }

    #[test]
    fn test_write_u16() {
        let mut buffer: [u8; 3] = [0; 3];

        // 1 byte
        assert_eq!(write_u16(0, &mut buffer[0..0]).is_err(), true);

        assert_eq!(write_u16(0, &mut buffer).unwrap(), 1);
        assert_eq!(buffer[0], 0);

        assert_eq!(write_u16(0x7F, &mut buffer).unwrap(), 1);
        assert_eq!(buffer[0], 0x7F);

        // 2 bytes
        assert_eq!(write_u16(0x80, &mut buffer[0..1]).is_err(), true);

        assert_eq!(write_u16(0x80, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x01);

        assert_eq!(write_u16(0xFF, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x01);

        assert_eq!(write_u16(0x3F_FF, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x7F);

        // 3 bytes
        assert_eq!(write_u16(0x40_00, &mut buffer[0..2]).is_err(), true);

        assert_eq!(write_u16(0x40_00, &mut buffer).unwrap(), 3);
        assert_eq!(buffer[0], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x01);

        assert_eq!(write_u16(0xFF_FF, &mut buffer).unwrap(), 3);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x03);
    }

    #[test]
    fn test_write_u32() {
        let mut buffer: [u8; 5] = [0; 5];

        // 1 byte
        assert_eq!(write_u32(0, &mut buffer[0..0]).is_err(), true);

        assert_eq!(write_u32(0, &mut buffer).unwrap(), 1);
        assert_eq!(buffer[0], 0);

        assert_eq!(write_u32(0x7F, &mut buffer).unwrap(), 1);
        assert_eq!(buffer[0], 0x7F);

        // 2 bytes
        assert_eq!(write_u32(0x80, &mut buffer[0..1]).is_err(), true);

        assert_eq!(write_u32(0x80, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x01);

        assert_eq!(write_u32(0xFF, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x01);

        assert_eq!(write_u32(0x3F_FF, &mut buffer).unwrap(), 2);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x7F);

        // 3 bytes
        assert_eq!(write_u32(0x40_00, &mut buffer[0..2]).is_err(), true);

        assert_eq!(write_u32(0x40_00, &mut buffer).unwrap(), 3);
        assert_eq!(buffer[0], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x01);

        assert_eq!(write_u32(0xFF_FF, &mut buffer).unwrap(), 3);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x03);

        assert_eq!(write_u32(0x1F_FF_FF, &mut buffer).unwrap(), 3);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x7F);

        // 4 bytes
        assert_eq!(write_u32(0x20_00_00, &mut buffer[0..3]).is_err(), true);

        assert_eq!(write_u32(0x20_00_00, &mut buffer).unwrap(), 4);
        assert_eq!(buffer[0], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[3], 0x01);

        assert_eq!(write_u32(0xF_FF_FF_FF, &mut buffer).unwrap(), 4);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[3], 0x7F);

        // 5 bytes
        assert_eq!(write_u32(0x10_00_00_00, &mut buffer[0..4]).is_err(), true);

        assert_eq!(write_u32(0x10_00_00_00, &mut buffer).unwrap(), 5);
        assert_eq!(buffer[0], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[3], 0x00 | CONTINUE_BIT);
        assert_eq!(buffer[4], 0x01);

        assert_eq!(write_u32(0xFF_FF_FF_FF, &mut buffer).unwrap(), 5);
        assert_eq!(buffer[0], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[1], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[2], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[3], 0x7F | CONTINUE_BIT);
        assert_eq!(buffer[4], 0x0F);

        // Specific data
        assert_eq!(write_u32(624485, &mut buffer).unwrap(), 3);
        assert_eq!(buffer[0], 0xE5);
        assert_eq!(buffer[1], 0x8E);
        assert_eq!(buffer[2], 0x26);
    }

    #[test]
    fn test_read_u8() {
        let mut value: u8 = 0;

        assert_eq!(read_u8(&[0x00; 0], &mut value).is_err(), true);
        assert_eq!(read_u8(&[CONTINUE_BIT], &mut value).is_err(), true);
        assert_eq!(
            read_u8(&[CONTINUE_BIT, CONTINUE_BIT], &mut value).is_err(),
            true
        );
        #[cfg(not(feature = "no_sanity_check"))]
        assert_eq!(
            read_u8(&[CONTINUE_BIT, CONTINUE_BIT, 0], &mut value).is_err(),
            true
        );

        // 1 byte
        assert_eq!(read_u8(&[0x00], &mut value).unwrap(), 1);
        assert_eq!(value, 0x00);
        assert_eq!(read_u8(&[0x7F], &mut value).unwrap(), 1);
        assert_eq!(value, 0x7F);

        // 2 bytes
        assert_eq!(
            read_u8(&[0x7F | CONTINUE_BIT, 0x01], &mut value).unwrap(),
            2
        );
        assert_eq!(value, 0xFF);

        // Out-of-range
        #[cfg(not(feature = "no_sanity_check"))]
        assert_eq!(
            read_u8(&[0x7F | CONTINUE_BIT, 0x02], &mut value).is_err(),
            true
        );
    }

    #[test]
    fn test_read_u16() {
        let mut value: u16 = 0;

        assert_eq!(read_u16(&[0x00; 0], &mut value).is_err(), true);
        assert_eq!(read_u16(&[CONTINUE_BIT], &mut value).is_err(), true);
        assert_eq!(
            read_u16(&[CONTINUE_BIT, CONTINUE_BIT], &mut value).is_err(),
            true
        );
        assert_eq!(
            read_u16(&[CONTINUE_BIT, CONTINUE_BIT, CONTINUE_BIT], &mut value).is_err(),
            true
        );
        #[cfg(not(feature = "no_sanity_check"))]
        assert_eq!(
            read_u16(&[CONTINUE_BIT, CONTINUE_BIT, CONTINUE_BIT, 0], &mut value).is_err(),
            true
        );

        // 1 byte
        assert_eq!(read_u16(&[0x00], &mut value).unwrap(), 1);
        assert_eq!(value, 0x00);
        assert_eq!(read_u16(&[0x7F], &mut value).unwrap(), 1);
        assert_eq!(value, 0x7F);

        // 2 bytes
        assert_eq!(
            read_u16(&[0x7F | CONTINUE_BIT, 0x01], &mut value).unwrap(),
            2
        );
        assert_eq!(value, 0xFF);
        assert_eq!(
            read_u16(&[0x7F | CONTINUE_BIT, 0x7F], &mut value).unwrap(),
            2
        );
        assert_eq!(value, 0x3F_FF);

        // 3 bytes
        assert_eq!(
            read_u16(
                &[0x00 | CONTINUE_BIT, 0x00 | CONTINUE_BIT, 0x01],
                &mut value
            )
            .unwrap(),
            3
        );
        assert_eq!(value, 0x40_00);
        assert_eq!(
            read_u16(
                &[0x7F | CONTINUE_BIT, 0x7F | CONTINUE_BIT, 0x03],
                &mut value
            )
            .unwrap(),
            3
        );
        assert_eq!(value, 0xFF_FF);

        // Out-of-range
        #[cfg(not(feature = "no_sanity_check"))]
        assert_eq!(
            read_u16(
                &[0x7F | CONTINUE_BIT, 0x7F | CONTINUE_BIT, 0x04],
                &mut value
            )
            .is_err(),
            true
        );
    }

    #[test]
    fn test_read_u32() {
        let mut value: u32 = 0;

        assert_eq!(read_u32(&[0x00; 0], &mut value).is_err(), true);
        assert_eq!(read_u32(&[CONTINUE_BIT], &mut value).is_err(), true);
        assert_eq!(
            read_u32(&[CONTINUE_BIT, CONTINUE_BIT], &mut value).is_err(),
            true
        );
        assert_eq!(
            read_u32(&[CONTINUE_BIT, CONTINUE_BIT, CONTINUE_BIT], &mut value).is_err(),
            true
        );
        assert_eq!(
            read_u32(
                &[CONTINUE_BIT, CONTINUE_BIT, CONTINUE_BIT, CONTINUE_BIT],
                &mut value
            )
            .is_err(),
            true
        );
        assert_eq!(
            read_u32(
                &[
                    CONTINUE_BIT,
                    CONTINUE_BIT,
                    CONTINUE_BIT,
                    CONTINUE_BIT,
                    CONTINUE_BIT
                ],
                &mut value
            )
            .is_err(),
            true
        );
        #[cfg(not(feature = "no_sanity_check"))]
        assert_eq!(
            read_u32(
                &[
                    CONTINUE_BIT,
                    CONTINUE_BIT,
                    CONTINUE_BIT,
                    CONTINUE_BIT,
                    CONTINUE_BIT,
                    0
                ],
                &mut value
            )
            .is_err(),
            true
        );

        // 1 byte
        assert_eq!(read_u32(&[0x00], &mut value).unwrap(), 1);
        assert_eq!(value, 0x00);
        assert_eq!(read_u32(&[0x7F], &mut value).unwrap(), 1);
        assert_eq!(value, 0x7F);

        // 2 bytes
        assert_eq!(
            read_u32(&[0x7F | CONTINUE_BIT, 0x01], &mut value).unwrap(),
            2
        );
        assert_eq!(value, 0xFF);
        assert_eq!(
            read_u32(&[0x7F | CONTINUE_BIT, 0x7F], &mut value).unwrap(),
            2
        );
        assert_eq!(value, 0x3F_FF);

        // 3 bytes
        assert_eq!(
            read_u32(
                &[0x00 | CONTINUE_BIT, 0x00 | CONTINUE_BIT, 0x01],
                &mut value
            )
            .unwrap(),
            3
        );
        assert_eq!(value, 0x40_00);
        assert_eq!(
            read_u32(
                &[0x7F | CONTINUE_BIT, 0x7F | CONTINUE_BIT, 0x7F],
                &mut value
            )
            .unwrap(),
            3
        );
        assert_eq!(value, 0x1F_FF_FF);

        // 4 bytes
        assert_eq!(
            read_u32(
                &[
                    0x00 | CONTINUE_BIT,
                    0x00 | CONTINUE_BIT,
                    0x00 | CONTINUE_BIT,
                    0x01
                ],
                &mut value
            )
            .unwrap(),
            4
        );
        assert_eq!(value, 0x20_00_00);
        assert_eq!(
            read_u32(
                &[
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x7F
                ],
                &mut value
            )
            .unwrap(),
            4
        );
        assert_eq!(value, 0xF_FF_FF_FF);

        // 5 bytes
        assert_eq!(
            read_u32(
                &[
                    0x00 | CONTINUE_BIT,
                    0x00 | CONTINUE_BIT,
                    0x00 | CONTINUE_BIT,
                    0x00 | CONTINUE_BIT,
                    0x01
                ],
                &mut value
            )
            .unwrap(),
            5
        );
        assert_eq!(value, 0x10_00_00_00);
        assert_eq!(
            read_u32(
                &[
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x0F
                ],
                &mut value
            )
            .unwrap(),
            5
        );
        assert_eq!(value, 0xFF_FF_FF_FF);

        // Out-of-range
        #[cfg(not(feature = "no_sanity_check"))]
        assert_eq!(
            read_u32(
                &[
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x7F | CONTINUE_BIT,
                    0x1F
                ],
                &mut value
            )
            .is_err(),
            true
        );
    }

    #[test]
    fn test_random_u32() {
        let mut rng = rand::thread_rng();
        let mut buffer: [u8; 5] = [0; 5];
        #[allow(unused)]
        'assert: for _ in 0..4096 {
            let value: u32 = rng.gen();
            let mut decoded_value: u32 = 0;
            write_u32(value, &mut buffer).unwrap();
            read_u32(&buffer, &mut decoded_value).unwrap();
            assert_eq!(value, decoded_value);
        }
    }
}
