use core::fmt;
use core::str::FromStr;

use musli::{Decode, Encode};

macro_rules! display {
    ($ty:ident) => {
        impl fmt::Display for $ty {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                Ok(())
            }
        }

        impl FromStr for $ty {
            type Err = &'static str;

            fn from_str(_: &str) -> Result<Self, Self::Err> {
                Err("never")
            }
        }
    };
}

enum Alt {}

#[derive(Encode, Decode)]
#[musli(mode = Alt, display)]
struct ModeSpecific;
display!(ModeSpecific);

#[derive(Encode, Decode)]
#[musli(encode_only, display)]
struct EncodeOnly;
display!(EncodeOnly);

#[derive(Encode, Decode)]
#[musli(display, packed)]
struct Packed;
display!(Packed);

#[derive(Encode, Decode)]
#[musli(display)]
#[musli(name_all = "kebab-case")]
struct NameAll;
display!(NameAll);

#[derive(Encode)]
#[musli(display)]
struct NoDisplay;

fn main() {}
