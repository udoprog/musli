use anyhow::Result;
use musli::alloc::{Disabled, Global, String};
use musli::context;
use musli::value::{self, Value};
use musli::{Allocator, Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct<A = Global>
where
    A: Allocator,
{
    value: Value<A>,
}

#[test]
fn with_allocator() -> Result<()> {
    assert_eq!(
        value::encode(Value::<Global>::empty())?,
        Value::<Global>::empty()
    );

    musli::macros::assert_roundtrip_eq! {
        descriptive,
        Struct::<Global> {
            value: Value::empty(),
        },
        json = r#"{"value":null}"#
    };

    Ok(())
}

/// A container which pins its allocator through `#[musli(global)]` can embed
/// types which are tied to the [`Global`] allocator.
#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(global)]
struct GlobalStruct {
    value: Value<Global>,
    string: String<Global>,
}

fn string(s: &str) -> Result<String<Global>> {
    let mut string = String::new_in(Global::new());
    string.push_str(s)?;
    Ok(string)
}

#[test]
fn global_attribute() -> Result<()> {
    musli::macros::assert_roundtrip_eq! {
        descriptive,
        GlobalStruct {
            value: Value::empty(),
            string: string("hello")?,
        },
        json = r#"{"value":null,"string":"hello"}"#
    };

    Ok(())
}

/// `#[musli(allocator = <type>)]` is the general form which `#[musli(global)]`
/// is shorthand for.
#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(allocator = Global)]
struct AllocatorStruct {
    value: Value<Global>,
}

#[test]
fn allocator_attribute() -> Result<()> {
    musli::macros::assert_roundtrip_eq! {
        descriptive,
        AllocatorStruct {
            value: Value::empty(),
        },
        json = r#"{"value":null}"#
    };

    Ok(())
}

/// Pinning the allocator works for enums, and for allocators other than
/// [`Global`].
#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(allocator = Disabled)]
enum DisabledEnum {
    Empty,
    Value(Value<Disabled>),
}

#[test]
fn disabled_allocator_attribute() -> Result<()> {
    let cx = context::new_in(Disabled::new());
    let encoding = musli::descriptive::Encoding::new();

    for expected in [DisabledEnum::Empty, DisabledEnum::Value(Value::from(42u32))] {
        let bytes = encoding.to_vec_with(&cx, &expected)?;
        let actual: DisabledEnum = encoding.from_slice_with(&cx, &bytes)?;
        assert_eq!(actual, expected);
    }

    Ok(())
}
