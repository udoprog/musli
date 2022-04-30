//! This is a test that ensures that arbitrary packs of data can be successfully skipped over.

use musli::mode::DefaultMode;
use musli::{Decode, Encode};
use musli_wire::tag::MAX_INLINE_LEN;
use musli_wire::{Variable, WireEncoding};

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
    #[musli(tag = 1)]
    small_pack: SmallPack,
    #[musli(tag = 3)]
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreSmall {
    prefix: u32,
    #[musli(tag = 2)]
    large_pack: LargePack,
    #[musli(tag = 3)]
    suffix: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct IgnoreBoth {
    prefix: u32,
    #[musli(tag = 3)]
    suffix: u32,
}

#[test]
fn test_packed_compat() {
    const ENCODING: WireEncoding<DefaultMode, Variable, Variable, 128> =
        WireEncoding::new().with_max_pack();

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

    let actual: IgnoreSmall = ENCODING.from_slice(&data).unwrap();
    assert_eq!(
        actual,
        IgnoreSmall {
            prefix: 42,
            large_pack: LargePack { large: [0; 128] },
            suffix: 84
        }
    );

    let actual: IgnoreLarge = ENCODING.from_slice(&data).unwrap();
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

    let actual: IgnoreBoth = ENCODING.from_slice(&data).unwrap();
    assert_eq!(
        actual,
        IgnoreBoth {
            prefix: 42,
            suffix: 84
        }
    );
}
