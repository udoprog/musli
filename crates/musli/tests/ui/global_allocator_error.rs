//! Tests for fields which use a concrete allocator, such as `Value<Global>`.
//!
//! Without `#[musli(allocator = <type>)]` or `#[musli(global)]` the `Decode`
//! derive introduces its own generic allocator parameter and emits `impl<'de,
//! A> Decode<'de, M, A> for ..  where A: Allocator`, while `Value<A>` and
//! `String<A>` only implement `Decode<'de, M, A>` for the allocator they are
//! allocated in. So the bound `Value<Global>: Decode<'de, M, A>` cannot be
//! satisfied for a generic `A`.
//!
//! Note that `Encode` is unaffected, since `Encode<M>` is not parameterized
//! over the allocator.

use musli::alloc::{Global, String};
use musli::value::Value;
use musli::{Allocator, Decode, Encode};

#[derive(Encode, Decode)]
struct GlobalValue {
    value: Value<Global>,
}

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
