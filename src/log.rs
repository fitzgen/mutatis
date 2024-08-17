#![allow(unused_macros, unused_imports)]

#[cfg(feature = "log")]
pub(crate) use ::log::{debug, error, info, trace, warn};

#[cfg(not(feature = "log"))]
macro_rules! debug {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "log"))]
pub(crate) use debug;

#[cfg(not(feature = "log"))]
macro_rules! error {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "log"))]
pub(crate) use error;

#[cfg(not(feature = "log"))]
macro_rules! info {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "log"))]
pub(crate) use info;

#[cfg(not(feature = "log"))]
macro_rules! trace {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "log"))]
pub(crate) use trace;

#[cfg(not(feature = "log"))]
macro_rules! warn_impl {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "log"))]
pub(crate) use warn_impl as warn;
