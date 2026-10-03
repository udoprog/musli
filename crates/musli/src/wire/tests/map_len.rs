use rust_alloc::collections::BTreeMap;
use rust_alloc::vec;
use rust_alloc::vec::Vec;

use crate::wire::tag::{DATA_MASK, Kind, Tag};
use crate::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name(type = usize))]
struct Inner {
    #[musli(name = 0)]
    a: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name(type = usize))]
struct Outer {
    #[musli(name = 0)]
    inner: Inner,
    #[musli(name = 1)]
    b: u32,
}

fn cont(n: u8) -> u8 {
    Tag::new(Kind::Continuation, n).byte()
}

/// The sequence of pairs making up `Inner`, with one extra value at the end
/// which belongs to no pair.
fn odd_inner(header: &[u8]) -> Vec<u8> {
    let mut bytes = vec![Tag::new(Kind::Sequence, 4).byte(), cont(0)];
    bytes.extend_from_slice(header);
    bytes.extend_from_slice(&[cont(0), cont(10), cont(1), cont(20)]);
    bytes
}

/// A pair sequence with an odd number of items must be rejected, rather than
/// leaving the last item for whatever is decoded next.
#[test]
fn odd_struct_len() {
    let even = crate::wire::to_vec(&Outer {
        inner: Inner { a: 10 },
        b: 20,
    })
    .unwrap();
    assert!(crate::wire::from_slice::<Outer>(&even).is_ok());

    // Inline length.
    let bytes = odd_inner(&[Tag::new(Kind::Sequence, 3).byte()]);
    let result = crate::wire::from_slice::<Outer>(&bytes);
    assert!(result.is_err(), "{result:?}");

    // Prefixed length.
    let bytes = odd_inner(&[Tag::new(Kind::Sequence, DATA_MASK).byte(), 3]);
    let result = crate::wire::from_slice::<Outer>(&bytes);
    assert!(result.is_err(), "{result:?}");
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(crate, name(type = usize))]
struct WithMap {
    #[musli(name = 0)]
    map: BTreeMap<u32, u32>,
    #[musli(name = 1)]
    b: u32,
}

#[test]
fn odd_map_len() {
    let expected = WithMap {
        map: BTreeMap::from([(0, 1)]),
        b: 20,
    };
    let even = crate::wire::to_vec(&expected).unwrap();
    assert_eq!(crate::wire::from_slice::<WithMap>(&even).unwrap(), expected);

    let bytes = vec![
        Tag::new(Kind::Sequence, 4).byte(),
        cont(0),
        Tag::new(Kind::Sequence, 3).byte(),
        cont(0),
        cont(1),
        cont(1),
        cont(20),
    ];
    let result = crate::wire::from_slice::<WithMap>(&bytes);
    assert!(result.is_err(), "{result:?}");

    // A map on its own.
    for len in [1, 3] {
        let mut bytes = vec![Tag::new(Kind::Sequence, len).byte()];
        bytes.extend((0..len).map(cont));
        let result = crate::wire::from_slice::<BTreeMap<u32, u32>>(&bytes);
        assert!(result.is_err(), "{len}: {result:?}");
    }
}
