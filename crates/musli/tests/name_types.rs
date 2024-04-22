use musli::{Decode, Encode};

#[test]
fn struct_fields() {
    macro_rules! test_case {
        ($ty:ty) => {{
            #[derive(Debug, PartialEq, Encode, Decode)]
            #[musli(name_type = $ty)]
            struct Struct {
                #[musli(name = <$ty>::MIN)]
                min: u32,
                #[musli(name = <$ty>::MAX)]
                max: u32,
            }

            musli::rt!(
                full,
                Struct { min: 42, max: 43 },
                json = format!(r#"{{"{}":42,"{}":43}}"#, <$ty>::MIN, <$ty>::MAX)
            );
        }};
    }

    test_case!(u8);
    test_case!(u16);
    test_case!(u32);
    test_case!(u64);
    test_case!(u128);
    test_case!(i8);
    test_case!(i16);
    test_case!(i32);
    test_case!(i64);
    test_case!(i128);
    test_case!(isize);
    test_case!(usize);
}

#[test]
fn variant_names() {
    macro_rules! test_case {
        ($ty:ty) => {{
            #[derive(Debug, PartialEq, Encode, Decode)]
            #[musli(name_type = $ty)]
            enum Enum {
                #[musli(packed, name = <$ty>::MAX)]
                Variant1(u32),
                #[musli(packed, name = <$ty>::MIN)]
                Variant2(u32),
            }

            musli::rt!(
                full,
                Enum::Variant1(43),
                json = format!(r#"{{"{}":[43]}}"#, <$ty>::MAX)
            );

            musli::rt!(
                full,
                Enum::Variant2(44),
                json = format!(r#"{{"{}":[44]}}"#, <$ty>::MIN)
            );
        }};
    }

    test_case!(u8);
    test_case!(u16);
    test_case!(u32);
    test_case!(u64);
    test_case!(u128);
    test_case!(i8);
    test_case!(i16);
    test_case!(i32);
    test_case!(i64);
    test_case!(i128);
    test_case!(isize);
    test_case!(usize);
}
