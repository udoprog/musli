use musli::{Decode, Encode};

use crate::tag::{Kind, Tag, MAX_INLINE_LEN};

#[test]
fn pack_max() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    struct Value {
        field: Field,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(packed)]
    struct Field {
        value: [u8; MAX_INLINE_LEN],
    }

    let value = Value {
        field: Field {
            value: [1; MAX_INLINE_LEN],
        },
    };

    let bytes = crate::to_vec(&value).unwrap();
    let actual: Value = crate::from_slice(&bytes).unwrap();

    assert_eq!(value, actual);

    assert_eq!(Tag::from_byte(bytes[2]), Tag::new(Kind::Prefix, 62));
    assert_eq!(bytes.len(), MAX_INLINE_LEN + 3);
}

#[test]
fn pow2() {
    const SIZE: usize = 110;

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct Value {
        field: Field,
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    #[musli(packed)]
    struct Field {
        value: [u8; SIZE],
    }

    let value = Value {
        field: Field { value: [1; SIZE] },
    };

    let bytes = crate::to_vec(&value).unwrap();
    let actual: Value = crate::from_slice(&bytes).unwrap();

    assert_eq!(value, actual);

    assert_eq!(Tag::from_byte(bytes[2]), Tag::new(Kind::Pack, 7));
    assert_eq!(bytes.len(), SIZE + 21);
    assert!(&bytes[SIZE + 3..].iter().all(|b| *b == 0));
}
