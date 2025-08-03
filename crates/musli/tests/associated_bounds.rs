use core::fmt;

use musli::mode::{Binary, Text};
use musli::{Decode, Encode};

trait Trait0 {
    type Inner;
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {T::Inner: musli::Encode<Binary>}, decode_bound<'de, A> = {T::Inner: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {T::Inner: musli::Encode<Text>}, decode_bound<'de, A> = {T::Inner: musli::de::Decode<'de, Text, A>})]
struct Struct0<T>
where
    T: Trait0,
{
    v: T::Inner,
}

impl<T> fmt::Debug for Struct0<T>
where
    T: Trait0<Inner: fmt::Debug>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Struct0").field("v", &self.v).finish()
    }
}

impl<T> PartialEq for Struct0<T>
where
    T: Trait0<Inner: PartialEq>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}

#[derive(Encode, Decode)]
#[musli(mode = Binary, bound = {Struct0<T>: musli::Encode<Binary>}, decode_bound<'de, A> = {Struct0<T>: musli::de::Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {Struct0<T>: musli::Encode<Text>}, decode_bound<'de, A> = {Struct0<T>: musli::de::Decode<'de, Text, A>})]
struct Struct2<T>
where
    T: Trait0,
{
    v2: Struct0<T>,
}

impl<T> fmt::Debug for Struct2<T>
where
    T: Trait0<Inner: fmt::Debug>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Struct2").field("v2", &self.v2).finish()
    }
}

impl<T> PartialEq for Struct2<T>
where
    T: Trait0<Inner: PartialEq>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.v2 == other.v2
    }
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

impl<T> fmt::Debug for Struct3<T>
where
    T: Trait2<Inner1: fmt::Debug, Inner2: fmt::Debug>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Struct3")
            .field("v1", &self.v1)
            .field("v2", &self.v2)
            .finish()
    }
}

impl<T> PartialEq for Struct3<T>
where
    T: Trait2<Inner1: PartialEq, Inner2: PartialEq>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.v1 == other.v1 && self.v2 == other.v2
    }
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

impl<T> fmt::Debug for Struct4<T>
where
    T: Trait2<Inner1: fmt::Debug, Inner2: fmt::Debug>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Struct4").field("v2", &self.v2).finish()
    }
}

impl<T> PartialEq for Struct4<T>
where
    T: Trait2<Inner1: PartialEq, Inner2: PartialEq>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.v2 == other.v2
    }
}

impl Trait0 for () {
    type Inner = u32;
}

impl Trait2 for () {
    type Inner1 = u32;
    type Inner2 = u32;
}

#[test]
fn associated_bounds() {
    musli::macros::assert_roundtrip_eq!(full, Struct0::<()> { v: 100 });

    musli::macros::assert_roundtrip_eq!(
        full,
        Struct2::<()> {
            v2: Struct0 { v: 100 },
        }
    );

    musli::macros::assert_roundtrip_eq!(full, Struct3::<()> { v1: 100, v2: 200 });

    musli::macros::assert_roundtrip_eq!(
        full,
        Struct4::<()> {
            v2: Struct3 { v1: 100, v2: 200 },
        }
    );
}
