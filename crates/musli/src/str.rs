//! Functions for working with strings. The exported implementations change
//! depending on if the `simdutf8` feature is enabled.

#![cfg(any(
    feature = "storage",
    feature = "wire",
    feature = "descriptive",
    feature = "json",
    feature = "value"
))]

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::fmt;

#[cfg(not(feature = "simdutf8"))]
#[doc(inline)]
pub use core::str::from_utf8;
#[cfg(feature = "simdutf8")]
#[doc(inline)]
pub use simdutf8::basic::from_utf8;

/// Error raised in case the UTF-8 sequence could not be decoded.
#[non_exhaustive]
#[derive(Debug)]
pub struct Utf8Error;

#[cfg(feature = "std")]
impl std::error::Error for Utf8Error {}

impl fmt::Display for Utf8Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid or incomplete utf-8 sequence")
    }
}

/// The same as [`String::from_utf8`], but the implementation can different
/// depending on if the `simdutf8` feature is enabled.
///
/// [`String::from_utf8`]: alloc::string::String::from_utf8
#[inline(always)]
#[cfg(all(feature = "alloc", not(feature = "simdutf8")))]
pub fn from_utf8_owned(bytes: Vec<u8>) -> Result<String, Utf8Error> {
    match String::from_utf8(bytes) {
        Ok(string) => Ok(string),
        Err(..) => Err(Utf8Error),
    }
}

/// The same as [`String::from_utf8`], but the implementation can different
/// depending on if the `simdutf8` feature is enabled.
///
/// [`String::from_utf8`]: alloc::string::String::from_utf8
#[inline(always)]
#[cfg(all(feature = "alloc", feature = "simdutf8"))]
pub fn from_utf8_owned(bytes: Vec<u8>) -> Result<String, Utf8Error> {
    if from_utf8(&bytes).is_err() {
        return Err(Utf8Error);
    }

    // SAFETY: String was checked above.
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}
