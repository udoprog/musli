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
use rust_alloc::string::String;
#[cfg(feature = "alloc")]
use rust_alloc::vec::Vec;

use core::fmt;

#[cfg(not(feature = "simdutf8"))]
#[doc(inline)]
pub use core::str::from_utf8;

/// Error raised in case the UTF-8 sequence could not be decoded.
#[derive(Debug)]
#[non_exhaustive]
pub struct Utf8Error;

impl core::error::Error for Utf8Error {}

impl fmt::Display for Utf8Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid or incomplete utf-8 sequence")
    }
}

/// The same as [`String::from_utf8`], but the implementation can different
/// depending on if the `simdutf8` feature is enabled.
///
/// [`String::from_utf8`]: rust_alloc::string::String::from_utf8
#[inline]
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
/// [`String::from_utf8`]: rust_alloc::string::String::from_utf8
#[inline]
#[cfg(all(feature = "alloc", feature = "simdutf8"))]
pub fn from_utf8_owned(bytes: Vec<u8>) -> Result<String, Utf8Error> {
    if simdutf8::basic::from_utf8(&bytes).is_err() {
        return Err(Utf8Error);
    }

    // SAFETY: String was checked above.
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}

/// Analogue to [`core::str::from_utf8()`].
///
/// Checks if the passed byte sequence is valid UTF-8 and returns an
/// [`std::str`] reference to the passed byte slice wrapped in `Ok()` if it is.
///
/// # Errors
///
/// Will return the zero-sized Err([`Utf8Error`]) on if the input contains
/// invalid UTF-8.
#[inline]
#[cfg(feature = "simdutf8")]
pub fn from_utf8(input: &[u8]) -> Result<&str, Utf8Error> {
    simdutf8::basic::from_utf8(input).map_err(|_| Utf8Error)
}
