/// Possible reasons of an [Error](#Error).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    /// The data provided is invalid.
    InvalidData,
    /// There is not enough data to perform the requested operation.
    NotEnoughData,
    /// Buffer is too small.
    BufferSmall,
}

/// Representation of an error.
#[derive(Debug)]
pub struct Error {
    /// Kind of the error.
    reason: ErrorKind,
}

impl Error {
    /// Creates a new `Error` instance.
    pub fn new(reason: ErrorKind) -> Self {
        Self { reason: reason }
    }

    /// Returns the reason of this error.
    pub fn kind(&self) -> ErrorKind {
        self.reason
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        Error::new(ErrorKind::InvalidData);
    }

    #[test]
    fn test_kind() {
        let err = Error::new(ErrorKind::InvalidData);
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert_ne!(err.kind(), ErrorKind::NotEnoughData);
    }

    #[test]
    fn test_partialeq() {
        let err1 = Error::new(ErrorKind::InvalidData);
        let err2 = Error::new(ErrorKind::NotEnoughData);
        let err3 = Error::new(ErrorKind::NotEnoughData);
        assert_ne!(err1, err2);
        assert_eq!(err2, err3);
    }

    #[cfg(not(feature = "no_std"))]
    #[test]
    fn test_std_fmt_debug() {
        println!("{:?}", Error::new(ErrorKind::InvalidData));
    }

    #[test]
    fn test_std_clone_clone() {
        Error::new(ErrorKind::InvalidData.clone());
    }
}
