use crate::wire::MAX_INLINE_LEN;
use crate::wire::tag::{Kind, Tag};
use crate::{Decode, Encode};

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
        ($size:expr, $len:expr) => {
            let value = From {
                prefix: Some(10),
                field: Field { value: [1; $size] },
                suffix: Some(20),
            };

            let bytes = crate::wire::to_vec(&value).unwrap();
            let actual: From<$size> = crate::wire::from_slice(&bytes).unwrap();
            let to: To = crate::wire::from_slice(&bytes).unwrap();

            assert_eq!(value, actual);
            assert_eq!(
                to,
                To {
                    prefix: Some(10),
                    suffix: Some(20)
                }
            );

            assert_eq!(Tag::from_byte(bytes[5]), Tag::new(Kind::Prefix, $len));
            assert_eq!(bytes.len(), $size + 9);
        };
    }

    test!(0, 0);
    test!(23, 23);
    test!(MAX_INLINE_LEN, 62);
}
