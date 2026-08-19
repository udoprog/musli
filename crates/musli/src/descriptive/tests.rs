use crate::{Decode, Encode};

use super::MAX_INLINE_LEN;
use super::tag::{Kind, Tag};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name(type = usize))]
struct From<const N: usize> {
    #[musli(name = 0)]
    prefix: Option<u32>,
    #[musli(name = 1)]
    field: Field<N>,
    #[musli(name = 2)]
    suffix: Option<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name(type = usize))]
struct To {
    #[musli(name = 0)]
    prefix: Option<u32>,
    #[musli(name = 2)]
    suffix: Option<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, packed)]
struct Field<const N: usize> {
    #[musli(bytes)]
    value: [u8; N],
}

#[test]
fn pack_inline_max() {
    macro_rules! test {
        ($size:expr) => {
            let value = From {
                prefix: Some(10),
                field: Field { value: [1; $size] },
                suffix: Some(20),
            };

            let bytes = super::to_vec(&value).unwrap();
            let actual: From<$size> = super::from_slice(&bytes).unwrap();
            let to: To = super::from_slice(&bytes).unwrap();

            assert_eq!(value, actual);
            assert_eq!(
                to,
                To {
                    prefix: Some(10),
                    suffix: Some(20)
                }
            );

            assert_eq!(Tag::from_byte(bytes[8]), Tag::new(Kind::Bytes, $size));
            assert_eq!(bytes.len(), $size + 14);
        };
    }

    test!(0);
    test!(23);
}

#[test]
fn max_inline_length() {
    macro_rules! test {
        ($size:expr, $inline:expr) => {
            let value = From {
                prefix: Some(10),
                field: Field {
                    value: [1; { $size }],
                },
                suffix: Some(20),
            };

            let bytes = super::to_vec(&value).unwrap();
            let actual: From<{ $size }> = super::from_slice(&bytes).unwrap();
            let to: To = super::from_slice(&bytes).unwrap();

            assert_eq!(actual, value);
            assert_eq!(
                to,
                To {
                    prefix: Some(10),
                    suffix: Some(20)
                }
            );

            assert_eq!(Tag::from_byte(bytes[8]), Tag::new(Kind::Bytes, $inline));
        };
    }

    test!(MAX_INLINE_LEN, MAX_INLINE_LEN as u8);
    test!(MAX_INLINE_LEN + 10, (MAX_INLINE_LEN + 1) as u8);
}

/// Skipping over a value with an absurd length prefix must be reported as an
/// error rather than overflowing the counter of values left to skip.
#[test]
fn skip_over_absurd_lengths() {
    use rust_alloc::vec;
    use rust_alloc::vec::Vec;

    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(crate, name(type = usize))]
    struct Empty {}

    // `usize::MAX` encoded with continuation encoding.
    fn absurd_len(kind: Kind) -> Vec<u8> {
        // A map with a single entry, where the value of the entry is the
        // container being skipped over.
        let mut bytes = vec![Tag::new(Kind::Map, 1).byte(), 0x59, 0x00];
        // A length which does not fit inline, so it is read as a prefix.
        bytes.push(Tag::new(kind, super::tag::DATA_MASK).byte());
        bytes.extend_from_slice(&[0xFF; 9]);
        bytes.push(0x01);
        // Trailing values so that skipping does not immediately run out of
        // input.
        bytes.extend_from_slice(&[Tag::from_mark(super::tag::Mark::Variant).byte(); 4]);
        bytes
    }

    for kind in [Kind::Sequence, Kind::Map] {
        let bytes = absurd_len(kind);
        assert!(
            super::from_slice::<Empty>(&bytes).is_err(),
            "{kind:?} with an absurd length should be rejected"
        );
    }
}
