#[derive(Debug)]
pub enum DecodeError {
    /// Weird FFI errors that should never happen
    /// (i.e. if you get this with a published version it's a bug.)
    FfiError(&'static str),

    /// Reading the header failed for some reason.
    ReadHeader,

    /// There were too many components in the supplied file.
    /// If it was a valid file this is a bug in the crate too.
    TooManyComponents(usize),

    // TODO: This should not be a problem in the future.
    UnspecifiedColorSpace,
    UnknownColorSpace,
}
