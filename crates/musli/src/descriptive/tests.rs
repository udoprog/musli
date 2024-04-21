use crate::{Decode, Encode};

use super::tag::{Kind, Tag};
use super::MAX_INLINE_LEN;

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name_type = usize)]
struct From<const N: usize> {
    #[musli(name = 0)]
    prefix: Option<u32>,
    #[musli(name = 1)]
    field: Field<N>,
    #[musli(name = 2)]
    suffix: Option<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name_type = usize)]
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
