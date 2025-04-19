use musli::mode::{Binary, Text};
use musli::{Decode, Encode};

trait Example {
    type Inner;
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {T::Inner: musli::Encode<Binary>}, decode_bound<'de, A> = {T::Inner: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {T::Inner: musli::Encode<Text>}, decode_bound<'de, A> = {T::Inner: musli::de::Decode<'de, Text, A>})]
struct Struct0<T>
where
    T: Example,
{
    v: T::Inner,
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {Struct0<T>: musli::Encode<Binary>}, decode_bound<'de, A> = {Struct0<T>: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {Struct0<T>: musli::Encode<Text>}, decode_bound<'de, A> = {Struct0<T>: musli::de::Decode<'de, Text, A>})]
struct Struct2<T>
where
    T: Example,
{
    v2: Struct0<T>,
}

pub trait Trait2 {
    type Inner1;
    type Inner2;
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {T::Inner1: musli::Encode<Binary>, T::Inner2: musli::Encode<Binary>}, decode_bound<'de, A> = {T::Inner1: musli::de::Decode<'de, Binary, A>, T::Inner2: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {T::Inner1: musli::Encode<Text>, T::Inner2: musli::Encode<Text>}, decode_bound<'de, A> = {T::Inner1: musli::de::Decode<'de, Text, A>, T::Inner2: musli::de::Decode<'de, Text, A>})]
pub struct Struct3<T>
where
    T: Trait2,
{
    v1: T::Inner1,
    v2: T::Inner2,
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {Struct3<T>: musli::Encode<Binary>}, decode_bound<'de, A> = {Struct3<T>: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {Struct3<T>: musli::Encode<Text>}, decode_bound<'de, A> = {Struct3<T>: musli::de::Decode<'de, Text, A>})]
pub struct Struct4<T>
where
    T: Trait2,
{
    v2: Struct3<T>,
}
