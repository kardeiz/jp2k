#[derive(Debug)]
pub enum Error {
    /// Weird FFI errors that should never happen
    /// (i.e. if you get this with a published version it's a bug.)
    FfiError(&'static str),
    /// Reading the header failed for some reason.
    ReadHeader,
    /// There were too many components in the supplied file.
    /// If it was a valid file this is a bug in the crate too.
    TooManyComponents(usize),
    UnspecifiedColorSpace,
    UnknownColorSpace,
    NulError(std::ffi::NulError),
    Io(std::io::Error),
    ImageContainerTooSmall,
}

impl From<std::ffi::NulError> for Error {
    fn from(t: std::ffi::NulError) -> Self { Self::NulError(t) }
}

impl From<std::io::Error> for Error {
    fn from(t: std::io::Error) -> Self { Self::Io(t) }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            FfiError(ref s) => { write!(f, "FFI: {}", s)?; },
            ReadHeader => { write!(f, "Reading the header failed for some reason")?; },
            TooManyComponents(u) => { write!(f, "There were too many components ({}) in the supplied file", u)?; },
            UnspecifiedColorSpace => { write!(f, "Unspecified color space")?; },
            UnknownColorSpace => { write!(f, "Unknown color space")?; },
            NulError(ref e) => { write!(f, "{}", e)?; },
            Io(ref e) => { write!(f, "{}", e)?; },
            ImageContainerTooSmall => { write!(f, "Image container is too small")?; },
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
