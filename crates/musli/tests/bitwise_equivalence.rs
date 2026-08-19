//! Differential: the bitwise fast path must produce the same bytes as the
//! element-wise path for a layout-identical type.

use musli::{Decode, Encode};

/// Identity functions which are enough to disable the bitwise fast path, since
/// that requires every field to use the default encode/decode path.
mod identity {
    use musli::de::Decoder;
    use musli::en::Encoder;

    #[inline]
    pub fn encode<E>(value: &u32, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode(value)
    }

    #[inline]
    pub fn decode<'de, D>(decoder: D) -> Result<u32, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode()
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
#[repr(C)]
struct Fast {
    a: u32,
    b: u32,
    c: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(packed)]
#[repr(C)]
struct Slow {
    a: u32,
    b: u32,
    #[musli(encode_with = identity::encode, decode_with = identity::decode)]
    c: u32,
}

const _: () = assert!(musli::is_bitwise_encode::<Fast>());
const _: () = assert!(musli::is_bitwise_decode::<Fast>());
const _: () = assert!(!musli::is_bitwise_encode::<Slow>());
const _: () = assert!(!musli::is_bitwise_decode::<Slow>());

#[test]
fn bitwise_matches_element_wise() {
    let values: Vec<(Fast, Slow)> = [
        (0u32, 0u32, 0u32),
        (1, 2, 3),
        (u32::MAX, 0, u32::MAX),
        (0x0102_0304, 0x0506_0708, 0x090A_0B0C),
    ]
    .into_iter()
    .map(|(a, b, c)| (Fast { a, b, c }, Slow { a, b, c }))
    .collect();

    for (fast, slow) in &values {
        for (name, f, s) in [
            (
                "storage",
                musli::storage::to_vec(fast).unwrap(),
                musli::storage::to_vec(slow).unwrap(),
            ),
            (
                "wire",
                musli::wire::to_vec(fast).unwrap(),
                musli::wire::to_vec(slow).unwrap(),
            ),
            (
                "descriptive",
                musli::descriptive::to_vec(fast).unwrap(),
                musli::descriptive::to_vec(slow).unwrap(),
            ),
        ] {
            assert_eq!(f, s, "{name} scalar {fast:?}");
        }
    }

    // The same for slices, which have their own bitwise path.
    let fast: Vec<Fast> = values.iter().map(|(f, _)| Fast { ..*f }).collect();
    let slow: Vec<Slow> = values.iter().map(|(_, s)| Slow { ..*s }).collect();

    for (name, f, s) in [
        (
            "storage",
            musli::storage::to_vec(&fast).unwrap(),
            musli::storage::to_vec(&slow).unwrap(),
        ),
        (
            "wire",
            musli::wire::to_vec(&fast).unwrap(),
            musli::wire::to_vec(&slow).unwrap(),
        ),
        (
            "descriptive",
            musli::descriptive::to_vec(&fast).unwrap(),
            musli::descriptive::to_vec(&slow).unwrap(),
        ),
    ] {
        assert_eq!(f, s, "{name} slice");
    }

    // And cross decoding: bytes written by one must decode as the other.
    for (fast, slow) in &values {
        let bytes = musli::storage::to_vec(fast).unwrap();

        assert_eq!(
            &musli::storage::from_slice::<Slow>(&bytes).unwrap(),
            slow,
            "bitwise bytes decoded element-wise"
        );

        let bytes = musli::storage::to_vec(slow).unwrap();

        assert_eq!(
            &musli::storage::from_slice::<Fast>(&bytes).unwrap(),
            fast,
            "element-wise bytes decoded bitwise"
        );
    }
}
