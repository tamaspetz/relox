#[cfg(feature = "default")]
#[test]
fn test_error() {
    use relox::{Error, ErrorKind};

    let error = Error::new(ErrorKind::InvalidData);
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_ne!(ErrorKind::NotEnoughData, ErrorKind::InvalidData);
}

#[cfg(feature = "default")]
#[test]
fn test_elf32rel() {
    use relox::Elf32Rel;
    use std::io::Cursor;

    let elf32rel = Elf32Rel::from_memory(&mut Cursor::new(&[0; 8])).unwrap();
    assert_eq!(elf32rel.offset(), 0x00);
    assert_eq!(elf32rel.relocation_type(), 0x00);
}
