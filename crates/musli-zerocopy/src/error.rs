use core::fmt;

#[derive(Debug)]
pub struct Error {
    repr: ErrorKind,
}

impl Error {
    #[inline]
    pub(crate) const fn new(repr: ErrorKind) -> Self {
        Self { repr }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.repr.fmt(f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ErrorKind {
    OutOfBounds { start: usize, end: usize },
    IndexOutOfBounds { index: usize, len: usize },
    FailedPhf,
    BadUtf8,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::OutOfBounds { start, end } => {
                write!(f, "Tried to access data out-of-bounds {start}-{end}")
            }
            ErrorKind::IndexOutOfBounds { index, len } => {
                write!(
                    f,
                    "Tried to access out-of-bounds slice index {index} not in 0-{len}"
                )
            }
            ErrorKind::FailedPhf => {
                write!(f, "Failed to build phf")
            }
            ErrorKind::BadUtf8 => {
                write!(f, "Bad utf-8 string")
            }
        }
    }
}
