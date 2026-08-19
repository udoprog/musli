use rust_alloc::vec;
use rust_alloc::vec::Vec;

use crate::Decode;
use crate::wire::tag::{DATA_MASK, Kind, Tag};

#[derive(Debug, PartialEq, Decode)]
#[musli(crate, name(type = usize))]
struct Empty {}

/// Skipping over a value with an absurd length prefix must be reported as an
/// error rather than overflowing the counter of values left to skip.
#[test]
fn skip_over_absurd_lengths() {
    // A sequence whose length does not fit inline, so it is read as a prefix.
    let prefix = Tag::new(Kind::Sequence, DATA_MASK).byte();
    // `usize::MAX` in continuation encoding.
    let mut max = vec![0xFFu8; 9];
    max.push(0x01);

    // A struct decodes as a sequence of key/value pairs. The value of the only
    // pair is a sequence with an absurd length, followed by more of the same so
    // that skipping does not immediately run out of input.
    let mut bytes: Vec<u8> = vec![
        Tag::new(Kind::Sequence, 2).byte(),
        Tag::new(Kind::Continuation, 0).byte(),
    ];

    for _ in 0..4 {
        bytes.push(prefix);
        bytes.extend_from_slice(&max);
    }

    assert!(super::super::from_slice::<Empty>(&bytes).is_err());
}
