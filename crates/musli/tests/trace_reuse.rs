//! Reusing one tracing context across several decodes must not leak path state
//! from one decode into the next.

use core::fmt;

use musli::alloc::{Allocator, ArrayBuffer, Slice};
use musli::context;
use musli::{Context, Decode, Decoder};

/// Enters more sequence indexes than the path has room for, then errors
/// without leaving them, the way a decode does when it errors deep inside a
/// structure.
struct Deep;

impl<'de, M, A> Decode<'de, M, A> for Deep
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        let cx = decoder.cx();
        // Report the error first so that it is allocated before the path runs
        // out of room.
        let error = cx.message("deep");
        decoder.skip()?;

        for index in 0..1024 {
            cx.enter_sequence_index(index);
        }

        Err(error)
    }
}

/// Errors at the path `[0]`, after having entered and left `[1]`.
struct Shallow;

impl<'de, M, A> Decode<'de, M, A> for Shallow
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        let cx = decoder.cx();
        decoder.skip()?;
        cx.enter_sequence_index(0);
        cx.enter_sequence_index(1);
        cx.leave_sequence_index();
        Err(cx.message("shallow"))
    }
}

/// A map key which fails to format.
struct Unformattable;

impl fmt::Display for Unformattable {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
}

/// Enters a map key which cannot be formatted under the path `[0]`, and a step
/// under that key, then errors either inside of them or after leaving them.
struct UnformattableKey<const INSIDE: bool>;

impl<'de, M, A, const INSIDE: bool> Decode<'de, M, A> for UnformattableKey<INSIDE>
where
    A: Allocator,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        let cx = decoder.cx();
        decoder.skip()?;
        cx.enter_sequence_index(0);
        cx.enter_map_key(Unformattable);
        // A step entered under the key which can be formatted.
        cx.enter_sequence_index(1);

        if INSIDE {
            return Err(cx.message("inside"));
        }

        cx.leave_sequence_index();
        cx.leave_map_key();
        Err(cx.message("outer"))
    }
}

macro_rules! errors {
    ($cx:expr) => {
        $cx.errors().map(|e| e.to_string()).collect::<Vec<_>>()
    };
}

/// A path which overflowed the capacity of the context in one decode must not
/// affect the path of the next.
#[test]
fn capped_path_is_reset() {
    let mut buf = ArrayBuffer::<4096>::with_size();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc).with_trace();
    let encoding = musli::json::Encoding::new();

    assert!(encoding.from_str_with::<_, Deep>(&cx, "0").is_err());
    let deep = errors!(cx);
    assert!(
        deep.iter().any(|e| e.contains("capped step")),
        "the path must overflow for this test to be meaningful: {deep:?}"
    );

    assert!(encoding.from_str_with::<_, Shallow>(&cx, "0").is_err());
    assert_eq!(errors!(cx), ["[0]: shallow (at byte 1)"]);
}

/// A map key which fails to format is not part of the path, so leaving it must
/// not remove its parent from the path.
#[test]
fn unformattable_map_key() {
    let mut buf = ArrayBuffer::<4096>::with_size();
    let alloc = Slice::new(&mut buf);
    let cx = context::new_in(&alloc).with_trace();
    let encoding = musli::json::Encoding::new();

    assert!(
        encoding
            .from_str_with::<_, UnformattableKey<false>>(&cx, "0")
            .is_err()
    );
    assert_eq!(errors!(cx), ["[0]: outer (at byte 1)"]);

    // Steps entered under the key are not recorded either, since they would
    // otherwise be removed in the wrong order when leaving them.
    assert!(
        encoding
            .from_str_with::<_, UnformattableKey<true>>(&cx, "0")
            .is_err()
    );
    assert_eq!(errors!(cx), ["[0] .. *2 capped steps*: inside (at byte 1)"]);
}
