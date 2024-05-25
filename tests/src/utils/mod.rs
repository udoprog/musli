/// Included mock framework so that we can rely on reachability analysis for compile checks.
///
/// You can also add this as a basis for adding new frameworks.
#[crate::benchmarker(disabled)]
pub mod mock {
    use core::fmt;

    #[derive(Debug)]
    pub struct Error;

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "mock framework should never run")
        }
    }

    pub fn encode<'buf, T>(#[allow(unused)] value: &T) -> Result<&'buf [u8], Error> {
        Err(Error)
    }

    pub fn decode<T>(#[allow(unused)] buf: &[u8]) -> Result<T, Error> {
        Err(Error)
    }
}

#[allow(unused_imports)]
pub use self::full::*;
mod full;

#[allow(unused_imports)]
pub use self::extra::*;
mod extra;

#[allow(unused_imports)]
pub use self::musli::*;
mod musli;
