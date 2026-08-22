use crate::Context;
use crate::de::Visitor;

use super::parse::Any;

/// Hand a number which was decoded without a type in mind to `visitor`, using
/// the narrowest type which holds it exactly.
pub(crate) fn visit_any<'de, C, V>(cx: C, any: Any, visitor: V) -> Result<V::Ok, V::Error>
where
    C: Context,
    V: Visitor<'de, C, Error = C::Error, Allocator = C::Allocator>,
{
    match any {
        Any::Unsigned(value) => {
            if value <= u8::MAX as u128 {
                return visitor.visit_u8(cx, value as u8);
            }

            if value <= u16::MAX as u128 {
                return visitor.visit_u16(cx, value as u16);
            }

            if value <= u32::MAX as u128 {
                return visitor.visit_u32(cx, value as u32);
            }

            if value <= u64::MAX as u128 {
                return visitor.visit_u64(cx, value as u64);
            }

            if value <= usize::MAX as u128 {
                return visitor.visit_usize(cx, value as usize);
            }

            visitor.visit_u128(cx, value)
        }
        Any::Signed(value) => {
            if value >= i8::MIN as i128 && value <= i8::MAX as i128 {
                return visitor.visit_i8(cx, value as i8);
            }

            if value >= i16::MIN as i128 && value <= i16::MAX as i128 {
                return visitor.visit_i16(cx, value as i16);
            }

            if value >= i32::MIN as i128 && value <= i32::MAX as i128 {
                return visitor.visit_i32(cx, value as i32);
            }

            if value >= i64::MIN as i128 && value <= i64::MAX as i128 {
                return visitor.visit_i64(cx, value as i64);
            }

            if value >= isize::MIN as i128 && value <= isize::MAX as i128 {
                return visitor.visit_isize(cx, value as isize);
            }

            visitor.visit_i128(cx, value)
        }
        Any::Float(value) => visitor.visit_f64(cx, value),
    }
}
