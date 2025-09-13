use musli::alloc::Global;
use musli::value::Value;
use musli::{Allocator, Decode, Encode};

// The allocator parameter should be usable and provided when constructing values.
#[derive(Encode, Decode)]
#[musli(bound = {A}, decode_bound = {A})]
struct Data1<A>
where
    A: Allocator,
{
    value: Value<A>,
}

#[derive(Encode, Decode)]
struct DecodeData2 {
    #[musli(global)]
    value: Value<Global>,
}
