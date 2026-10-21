//! Tests for fields which use a concrete allocator, such as `String<Global>`.
//!
//! Without `#[musli(allocator = <type>)]` or `#[musli(global)]` the `Decode`
//! derive introduces its own generic allocator parameter and emits `impl<'de,
//! A> Decode<'de, M, A> for ..  where A: Allocator`, while `String<A>` only
//! implements `Decode<'de, M, A>` for the allocator it is allocated in. So the
//! bound `String<Global>: Decode<'de, M, A>` cannot be satisfied for a generic
//! `A`.
//!
//! `Value<Global>` is not affected, since it can be decoded by a decoder using
//! any allocator.
//!
//! Note that `Encode` is unaffected, since `Encode<M>` is not parameterized
//! over the allocator.

use musli::alloc::{Global, String};
use musli::value::Value;
use musli::{Allocator, Decode, Encode};

#[derive(Encode, Decode)]
struct GlobalString {
    string: String<Global>,
}

/// `#[musli(global)]` is shorthand for `#[musli(allocator = Global)]`, so
/// specifying both is redundant.
#[derive(Encode, Decode)]
#[musli(global, allocator = Global)]
struct BothAttributes {
    value: Value<Global>,
}

/// The allocator cannot be pinned for a type which is already generic over its
/// allocator.
#[derive(Encode, Decode)]
#[musli(global)]
struct AlreadyGeneric<A>
where
    A: Allocator,
{
    value: Value<A>,
}

/// There is no allocator parameter left to name once it has been pinned.
#[derive(Encode, Decode)]
#[musli(Binary, global, decode_bound<'de, A> = {})]
struct NamedAllocator {
    value: Value<Global>,
}

fn main() {}
