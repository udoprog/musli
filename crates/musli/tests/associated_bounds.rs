use musli::mode::{Binary, Text};
use musli::{Decode, Encode};

trait Example {
    type Inner;
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {T::Inner: musli::Encode<Binary>}, decode_bound<'de, A> = {T::Inner: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {T::Inner: musli::Encode<Text>}, decode_bound<'de, A> = {T::Inner: musli::de::Decode<'de, Text, A>})]
struct S1<T>
where
    T: Example,
{
    v: T::Inner,
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {S1<T>: musli::Encode<Binary>}, decode_bound<'de, A> = {S1<T>: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {S1<T>: musli::Encode<Text>}, decode_bound<'de, A> = {S1<T>: musli::de::Decode<'de, Text, A>})]
struct S2<T>
where
    T: Example,
{
    v2: S1<T>,
}
