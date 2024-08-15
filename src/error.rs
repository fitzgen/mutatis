//! Error and result types for the `mutatis` crate.

use core::fmt;

/// A result that is either `Ok(T)` or `Err(mutatis::Error)`.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// An extension trait for [`mutatis::Result`][crate::Result] that provides
/// additional methods.
pub trait ResultExt {
    /// Ignores the error if it is
    /// [`MutatorExhausted`][ErrorKind::MutatorExhausted], returning `Ok(())`
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use mutatis::{Error, Result, ResultExt};
    ///
    /// let result: Result<()> = Err(Error::mutator_exhausted());
    /// let result = result.ignore_mutator_exhausted();
    /// assert!(result.is_ok());
    /// ```
    fn ignore_mutator_exhausted(self) -> Result<()>;
}

impl<T> ResultExt for Result<T> {
    #[inline]
    fn ignore_mutator_exhausted(self) -> Result<()> {
        match self {
            Ok(_) => Ok(()),
            Err(err) => {
                if err.is_mutator_exhausted() {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }
}

/// An error that can occur when using the `mutatis` crate.
///
/// This type is a thin wrapper around [`ErrorKind`], which contains the
/// specific kind of error that occurred.
///
/// # Examples
///
/// ```
/// use mutatis::{Error, ErrorKind};
///
/// let error: Error = {
///     // ...
/// #   Error::mutator_exhausted()
/// };
///
/// if error.is_mutator_exhausted() {
///     println!("exhausted!");
/// }
///
/// match error.kind() {
///     ErrorKind::MutatorExhausted => println!("still exhausted!"),
///     ErrorKind::Other(msg) => println!("other! {msg}"),
///     unknown => println!("unknown! {unknown:?}"),
/// }
/// ```
pub struct Error {
    // When we can, box the inner error kind to save space in the `Error`
    // struct. This is only possible when the `alloc` feature is enabled.
    #[cfg(feature = "alloc")]
    kind: alloc::boxed::Box<ErrorKind>,
    #[cfg(not(feature = "alloc"))]
    kind: ErrorKind,
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Self {
        #[cfg(feature = "alloc")]
        let kind = alloc::boxed::Box::new(kind);
        Self { kind }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            ErrorKind::MutatorExhausted => {
                write!(f, "the mutator is exhausted")
            }
            ErrorKind::InvalidRange => {
                write!(f, "the mutator was given an invalid range")
            }
            ErrorKind::Other(msg) => {
                write!(f, "an unknown error occurred: {msg}")
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl Error {
    /// Returns a new error indicating that the mutator is exhausted.
    #[must_use]
    pub fn mutator_exhausted() -> Self {
        ErrorKind::MutatorExhausted.into()
    }

    /// Returns a new error indicating that the given range is invalid.
    #[must_use]
    pub fn invalid_range() -> Self {
        ErrorKind::InvalidRange.into()
    }

    /// Returns a new error with the given message.
    #[must_use]
    pub fn other(msg: impl Into<ErrorMessage>) -> Self {
        ErrorKind::Other(msg.into()).into()
    }

    /// Returns the kind of this error.
    #[must_use]
    pub fn kind(&self) -> &ErrorKind {
        return &self.kind;
    }

    /// Returns `true` if the error's kind is
    /// [`MutatorExhausted`][ErrorKind::MutatorExhausted].
    #[must_use]
    pub fn is_mutator_exhausted(&self) -> bool {
        matches!(self.kind(), ErrorKind::MutatorExhausted)
    }

    /// Returns `true` if the error's kind is
    /// [`InvalidRange`][ErrorKind::InvalidRange].
    #[must_use]
    pub fn is_invalid_range(&self) -> bool {
        matches!(self.kind(), ErrorKind::InvalidRange)
    }

    /// Returns `true` if the error's kind is
    /// [`Other`][ErrorKind::Other].
    #[must_use]
    pub fn is_other(&self) -> bool {
        matches!(self.kind(), ErrorKind::Other(_))
    }
}

/// The kind of an error that can occur when using the `mutatis` crate.
///
/// This enum is not exhaustive, and new variants may be added in the future.
/// When matching on this enum, a catch-all arm should be used to handle any
/// new variants that are added.
///
/// # Examples
///
/// ```
/// use mutatis::{Error, ErrorKind};
///
/// let error: Error = {
///     // ...
/// #   Error::mutator_exhausted()
/// };
///
/// match error.kind() {
///     ErrorKind::MutatorExhausted => println!("exhausted!"),
///     ErrorKind::Other(msg) => println!("other! {msg}"),
///     unknown => println!("unknown! {unknown:?}"),
/// }
/// ```
#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorKind {
    /// The mutator is exhausted.
    MutatorExhausted,

    /// The mutator was given an invalid range.
    InvalidRange,

    /// Some other error occurred.
    Other(ErrorMessage),
}

impl From<Error> for ErrorKind {
    #[inline]
    fn from(err: Error) -> Self {
        #[cfg(feature = "alloc")]
        return *err.kind;
        #[cfg(not(feature = "alloc"))]
        return err.kind;
    }
}

/// A message that can be attached to an error.
///
/// This should only be used with `ErrorKind::Other` and in situations where
/// there is not a more-specific error kind to use.
///
/// By default, this type is a thin wrapper around a string slice. When the
/// `alloc` feature is enabled, it can be a borrowed or owned string.
///
/// # Examples
///
/// ```
/// use mutatis::ErrorMessage;
///
/// let msg = ErrorMessage::new("something went wrong");
/// assert_eq!(msg.as_str(), "something went wrong");
/// ```
#[derive(Debug)]
pub struct ErrorMessage {
    #[cfg(feature = "alloc")]
    inner: alloc::borrow::Cow<'static, str>,
    #[cfg(not(feature = "alloc"))]
    inner: &'static str,
}

impl ErrorMessage {
    /// Returns a new error message with the given string.
    #[must_use]
    pub fn new(msg: impl Into<ErrorMessage>) -> Self {
        msg.into()
    }

    /// Returns the message as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        return &self.inner;
    }
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&'static str> for ErrorMessage {
    #[inline]
    fn from(s: &'static str) -> Self {
        let inner = s;
        #[cfg(feature = "alloc")]
        let inner = alloc::borrow::Cow::Borrowed(inner);
        Self { inner }
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::borrow::Cow<'static, str>> for ErrorMessage {
    #[inline]
    fn from(s: alloc::borrow::Cow<'static, str>) -> Self {
        Self { inner: s }
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::string::String> for ErrorMessage {
    #[inline]
    fn from(s: alloc::string::String) -> Self {
        let inner = s.into();
        Self { inner }
    }
}