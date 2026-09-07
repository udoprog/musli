#![cfg(all(
    feature = "std",
    feature = "alloc",
    feature = "json",
    feature = "value"
))]

//! Floats decoded without a concrete target type must be correctly rounded.
//!
//! The typed path reaches [`dec2flt`] directly, but a document decoded into a
//! [`Value`] goes through the visitor instead. That path used to reassemble the
//! number as `mantissa * 10f64.powi(exp)`, which is off by an ulp for a large
//! share of ordinary literals and collapses entirely at the ends of the range.
//! Everything here therefore compares bits, since an ulp hides in a value
//! comparison, and decodes through [`Value`] rather than into a float.
//!
//! [`dec2flt`]: musli::dec2flt

use std::fmt::Write;

use musli::alloc::Global;
use musli::value::Value;

/// Literals which the `pow10` reconstruction got wrong: the first four by an
/// ulp, the rest by flushing to zero or saturating to infinity because the
/// intermediate power of ten was itself zero or infinite.
const LITERALS: &[&str] = &[
    "0.6",
    "0.35",
    "0.3",
    "0.7",
    "8.98846567431158e307",
    "2.2250738585072014e-308",
    "1e-320",
    "5e-324",
    "1.7976931348623157e308",
];

/// Decode `s` the way a consumer without a concrete target type does.
#[track_caller]
fn via_value<T>(s: &str) -> T
where
    T: for<'de> musli::Decode<'de, musli::mode::Binary, Global>,
{
    let value: Value<Global> = musli::json::from_str(s).expect("Failed to decode JSON");
    musli::value::decode(&value).expect("Failed to decode value")
}

#[test]
fn f64_via_value_is_correctly_rounded() {
    for s in LITERALS {
        let expected: f64 = s.parse().unwrap();
        assert_eq!(via_value::<f64>(s).to_bits(), expected.to_bits(), "{s}");

        let mut negative = String::from("-");
        negative.push_str(s);
        let expected: f64 = negative.parse().unwrap();
        assert_eq!(
            via_value::<f64>(&negative).to_bits(),
            expected.to_bits(),
            "{negative}"
        );
    }
}

#[test]
fn f32_via_value_is_correctly_rounded() {
    for s in LITERALS {
        let expected: f32 = s.parse().unwrap();
        assert_eq!(via_value::<f32>(s).to_bits(), expected.to_bits(), "{s}");
    }
}

/// The ulp drift was not exotic: 146 of these decoded to the wrong `f64` before
/// the visitor path was routed through `dec2flt`.
#[test]
fn every_three_decimal_literal_round_trips() {
    let mut s = String::new();

    for n in 0..=1000u32 {
        s.clear();
        write!(s, "{}.{:03}", n / 1000, n % 1000).unwrap();

        let expected: f64 = s.parse().unwrap();
        assert_eq!(via_value::<f64>(&s).to_bits(), expected.to_bits(), "{s}");
    }
}

/// A drifted `Value` persists the error, since re-encoding writes the extra
/// digits back out and `0.6` becomes the literal `0.6000000000000001`.
#[test]
fn re_encoding_a_value_preserves_the_literal() {
    for s in LITERALS {
        let value: Value<Global> = musli::json::from_str(s).unwrap();
        let encoded = musli::json::to_string(&value).unwrap();

        let expected: f64 = s.parse().unwrap();
        let actual: f64 = encoded.parse().unwrap();
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{s} encoded as {encoded}"
        );
    }
}

/// JSONB stores numbers as the same ASCII payload and shares `parse_float`, so
/// the visitor path there is the same one. `FLOAT` is what SQLite writes for
/// JSON input, `FLOAT5` what it writes for the JSON5 spellings.
#[cfg(feature = "sqlite-jsonb")]
#[test]
fn jsonb_float_via_value_is_correctly_rounded() {
    /// Element type of a JSON float, and of a JSON5 float.
    const FLOAT: u8 = 5;
    const FLOAT5: u8 = 6;

    /// Wrap an ASCII number payload in its element header.
    fn element(kind: u8, payload: &str) -> Vec<u8> {
        let mut out = Vec::new();

        if payload.len() <= 11 {
            out.push(((payload.len() as u8) << 4) | kind);
        } else {
            out.push(0xc0 | kind);
            out.push(payload.len() as u8);
        }

        out.extend_from_slice(payload.as_bytes());
        out
    }

    for s in LITERALS {
        let expected: f64 = s.parse().unwrap();
        let blob = element(FLOAT, s);

        let value: Value<Global> = musli::sqlite_jsonb::from_slice(&blob).unwrap();
        let actual: f64 = musli::value::decode(&value).unwrap();
        assert_eq!(actual.to_bits(), expected.to_bits(), "{s}");
    }

    // The JSON5 spellings, which only ever arrive as `FLOAT5`.
    for (s, expected) in [(".5", 0.5f64), ("5.", 5.0), ("-.6", -0.6)] {
        let blob = element(FLOAT5, s);

        let value: Value<Global> = musli::sqlite_jsonb::from_slice(&blob).unwrap();
        let actual: f64 = musli::value::decode(&value).unwrap();
        assert_eq!(actual.to_bits(), expected.to_bits(), "{s}");
    }
}
