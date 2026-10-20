use core::fmt;
use core::str::FromStr;

use musli::{Decode, Encode};

#[derive(Encode)]
#[musli(crate = musli, display)]
struct Borrowed<'a>(&'a str);

impl fmt::Display for Borrowed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Encode, Decode)]
#[musli(display)]
struct Fixed<const N: usize>([u8; N]);

impl<const N: usize> fmt::Display for Fixed<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", N)
    }
}

impl<const N: usize> FromStr for Fixed<N> {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _: usize = s.parse()?;
        Ok(Fixed([0; N]))
    }
}

fn main() {}
