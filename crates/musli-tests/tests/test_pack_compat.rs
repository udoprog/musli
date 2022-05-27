//! This is a test that ensures that arbitrary packs of data can be successfully skipped over.

use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_wire::int::Variable;
use musli_wire::tag::MAX_INLINE_LEN;
use musli_wire::Encoding;

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
fn test_packed_compat() {
    const ENCODING: Encoding<DefaultMode, Variable, Variable, 128> =
        Encoding::new().with_max_pack();

    let data = ENCODING
        .to_buffer(&SmallPackCompat {
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
