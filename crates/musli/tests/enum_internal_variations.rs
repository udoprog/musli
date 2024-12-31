use bstr::BStr;
use musli::{Decode, Encode};

macro_rules! integer_tag {
    ($name:ident, $name_value:ident, $ty:ty) => {
        #[derive(Encode, Decode)]
        #[musli(tag(value = 10, type = $ty))]
        pub enum $name {
            Value,
        }

        #[derive(Encode, Decode)]
        #[musli(tag(value = 10, type = $ty, method = "value"))]
        pub enum $name_value {
            Value,
        }
    };
}

integer_tag!(U8, U8Value, u8);
integer_tag!(U16, U16Value, u16);
integer_tag!(U32, U32Value, u32);
integer_tag!(U64, U64Value, u64);
integer_tag!(U128, U128Value, u128);
integer_tag!(I8, I8Value, i8);
integer_tag!(I16, I16Value, i16);
integer_tag!(I32, I32Value, i32);
integer_tag!(I64, I64Value, i64);
integer_tag!(I128, I128Value, i128);
integer_tag!(Usize, UsizeValue, usize);
integer_tag!(Isize, IsizeValue, isize);

#[derive(Encode, Decode)]
#[musli(tag(value = 10))]
pub enum Integer {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag(value = 10, method = "value"))]
pub enum IntegerValue {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag = "tag")]
pub enum String {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag(value = "tag"))]
pub enum StringValue {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag(value = "tag", type = str))]
pub enum StringValueType {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag(value = "tag", type = str, method = "unsized"))]
pub enum StringValueTypeUnsized {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag(value = b"tag", format_with = BStr::new))]
pub enum BytesValue {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag(value = b"tag", type = [u8], format_with = BStr::new))]
pub enum BytesValueType {
    Value,
}

#[derive(Encode, Decode)]
#[musli(tag(value = b"tag", type = [u8], method = "unsized_bytes", format_with = BStr::new))]
pub enum BytesValueTypeMethod {
    Value,
}
