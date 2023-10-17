//! This is a test that ensures that arbitrary packs of data can be successfully skipped over.

#[cfg(feature = "test")]
use musli::mode::DefaultMode;
use musli::{Decode, Encode};
#[cfg(feature = "test")]
use musli_wire::int::Variable;
#[cfg(feature = "test")]
use musli_wire::tag::MAX_INLINE_LEN;
#[cfg(feature = "test")]
use musli_wire::Encoding;
#[cfg(not(feature = "test"))]
const MAX_INLINE_LEN: usize = 64;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Inner;

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
struct SmallPack {
    small: [u8; MAX_INLINE_LEN],
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
struct LargePack {
    large: [u8; 128],
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct SmallPackCompat {
    prefix: u32,
    small_pack: SmallPack,
    large_pack: LargePack,
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreLarge {
    prefix: u32,
    #[musli(rename = 1)]
    small_pack: SmallPack,
    #[musli(rename = 3)]
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreSmall {
    prefix: u32,
    #[musli(rename = 2)]
    large_pack: LargePack,
    #[musli(rename = 3)]
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreBoth {
    prefix: u32,
    #[musli(rename = 3)]
    suffix: u32,
}

#[test]
#[cfg(feature = "test")]
fn test_packed_compat() {
    const ENCODING: Encoding<DefaultMode, Variable, Variable> = Encoding::new();

    let data = ENCODING
        .to_vec(&SmallPackCompat {
            prefix: 42,
            small_pack: SmallPack {
                small: [0; MAX_INLINE_LEN],
            },
            large_pack: LargePack { large: [0; 128] },
            suffix: 84,
        })
        .unwrap();

    let actual: IgnoreSmall = ENCODING.from_slice(data.as_slice()).unwrap();
    assert_eq!(
        actual,
        IgnoreSmall {
            prefix: 42,
            large_pack: LargePack { large: [0; 128] },
            suffix: 84
        }
    );

    let actual: IgnoreLarge = ENCODING.from_slice(data.as_slice()).unwrap();
    assert_eq!(
        actual,
        IgnoreLarge {
            prefix: 42,
            small_pack: SmallPack {
                small: [0; MAX_INLINE_LEN]
            },
            suffix: 84
        }
    );

    let actual: IgnoreBoth = ENCODING.from_slice(data.as_slice()).unwrap();
    assert_eq!(
        actual,
        IgnoreBoth {
            prefix: 42,
            suffix: 84
        }
    );
}
