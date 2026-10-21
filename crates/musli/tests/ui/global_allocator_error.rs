//! Tests for fields which use a concrete allocator, such as `String<Disabled>`.
//!
//! Without `#[musli(allocator = <type>)]` or `#[musli(global)]` the `Decode`
//! derive introduces its own generic allocator parameter and emits `impl<'de,
//! A> Decode<'de, M, A> for ..  where A: Allocator`, while `String<B>` only
//! implements `Decode<'de, M, A>` for allocators `B` which can be obtained from
//! `A` through `FromAllocator<A>`. `Disabled` can only be obtained from itself,
//! so the bound `String<Disabled>: Decode<'de, M, A>` cannot be satisfied for a
//! generic `A`.
//!
//! `String<Global>`, `Vec<T, Global>` and `Value<Global>` are not affected,
//! since `Global` can be obtained from any allocator.
//!
//! Note that `Encode` is unaffected, since `Encode<M>` is not parameterized
//! over the allocator.

use musli::alloc::{Disabled, Global, String, Vec};
use musli::value::Value;
use musli::{Allocator, Decode, Encode};

#[derive(Encode, Decode)]
struct DisabledString {
    string: String<Disabled>,
}

/// Containers in the global allocator need no attributes.
#[derive(Encode, Decode)]
struct GlobalContainers {
    string: String<Global>,
    vec: Vec<String<Global>, Global>,
    #[musli(bytes)]
    bytes: Vec<u8, Global>,
    value: Value<Global>,
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
