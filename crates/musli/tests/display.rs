#![cfg(feature = "test")]

use core::fmt;
use core::str::FromStr;
use std::net::Ipv4Addr;

use musli::{Decode, Encode};

/// A color which is `#rrggbb` on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rgb(u8, u8, u8);

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

/// Only implements `Display`, not `Error`.
#[derive(Debug)]
struct RgbError;

impl fmt::Display for RgbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected a color like #rrggbb")
    }
}

impl FromStr for Rgb {
    type Err = RgbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix('#').ok_or(RgbError)?;

        if hex.len() != 6 || !hex.is_ascii() {
            return Err(RgbError);
        }

        let part = |n: usize| u8::from_str_radix(&hex[n..n + 2], 16).map_err(|_| RgbError);
        Ok(Rgb(part(0)?, part(2)?, part(4)?))
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithFields {
    #[musli(with = musli::display)]
    address: Ipv4Addr,
    #[musli(with = musli::display)]
    color: Rgb,
    #[musli(with = musli::display::option)]
    gateway: Option<Ipv4Addr>,
    #[musli(with = musli::display::option)]
    accent: Option<Rgb>,
}

#[test]
fn with_roundtrip() {
    musli::macros::assert_roundtrip_eq! {
        full,
        WithFields {
            address: Ipv4Addr::new(10, 0, 0, 2),
            color: Rgb(0x12, 0xab, 0xff),
            gateway: Some(Ipv4Addr::new(10, 0, 0, 1)),
            accent: None,
        },
        json = r##"{"address":"10.0.0.2","color":"#12abff","gateway":"10.0.0.1","accent":null}"##
    };
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct OptionalDefault {
    #[musli(default, skip_encoding_if = Option::is_none, with = musli::display::option)]
    color: Option<Rgb>,
}

#[test]
fn with_option_default() {
    musli::macros::assert_roundtrip_eq! {
        full,
        OptionalDefault { color: None },
        json = r#"{}"#
    };

    musli::macros::assert_roundtrip_eq! {
        full,
        OptionalDefault { color: Some(Rgb(1, 2, 3)) },
        json = r##"{"color":"#010203"}"##
    };

    let value: OptionalDefault = musli::json::from_str("{}").unwrap();
    assert_eq!(value, OptionalDefault { color: None });
}

#[test]
fn with_parse_error() {
    let error = musli::json::from_str::<WithFields>(
        r##"{"address":"10.0.0","color":"#000000","gateway":null,"accent":null}"##,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("invalid IPv4 address syntax"),
        "{error}"
    );

    let error = musli::json::from_str::<WithFields>(
        r##"{"address":"10.0.0.1","color":"red","gateway":null,"accent":null}"##,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("expected a color like #rrggbb"),
        "{error}"
    );

    let error = musli::json::from_str::<WithFields>(
        r##"{"address":"10.0.0.1","color":"#000000","gateway":null,"accent":"#00000"}"##,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("expected a color like #rrggbb"),
        "{error}"
    );
}

#[test]
fn with_not_a_string() {
    assert!(
        musli::json::from_str::<WithFields>(
            r##"{"address":42,"color":"#000000","gateway":null,"accent":null}"##
        )
        .is_err()
    );

    assert!(
        musli::json::from_str::<WithFields>(
            r##"{"address":"10.0.0.1","color":"#000000","gateway":[1],"accent":null}"##
        )
        .is_err()
    );
}

/// A container which is a string on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[musli(display)]
enum Level {
    Low,
    High,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Low => f.write_str("low"),
            Level::High => f.write_str("high"),
        }
    }
}

impl FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Level::Low),
            "high" => Ok(Level::High),
            other => Err(format!("unknown level `{other}`")),
        }
    }
}

/// A generic container, which uses the `Display` and `FromStr` bounds of its
/// own implementations.
#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(display)]
struct Id<T>(T);

impl<T> fmt::Display for Id<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id-{}", self.0)
    }
}

impl<T> FromStr for Id<T>
where
    T: FromStr,
{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix("id-").ok_or("missing id- prefix")?;
        Ok(Id(rest.parse().map_err(|_| "bad id")?))
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Uses {
    level: Level,
    levels: Vec<Level>,
    id: Id<u32>,
    // Last, since the packed format only supports optional values at the end.
    maybe: Option<Level>,
}

#[test]
fn container_roundtrip() {
    musli::macros::assert_roundtrip_eq! {
        full,
        Uses {
            level: Level::High,
            levels: vec![Level::Low, Level::High],
            id: Id(42),
            maybe: Some(Level::Low),
        },
        json = r#"{"level":"high","levels":["low","high"],"id":"id-42","maybe":"low"}"#
    };

    musli::macros::assert_roundtrip_eq! {
        full,
        Uses {
            level: Level::Low,
            levels: vec![],
            id: Id(0),
            maybe: None,
        },
        json = r#"{"level":"low","levels":[],"id":"id-0","maybe":null}"#
    };
}

#[test]
fn container_parse_error() {
    let error = musli::json::from_str::<Level>(r#""medium""#).unwrap_err();
    assert!(
        error.to_string().contains("unknown level `medium`"),
        "{error}"
    );

    let error = musli::json::from_str::<Id<u32>>(r#""id-x""#).unwrap_err();
    assert!(error.to_string().contains("bad id"), "{error}");

    let error = musli::json::from_str::<Option<Level>>(r#""""#).unwrap_err();
    assert!(error.to_string().contains("unknown level ``"), "{error}");

    assert!(musli::json::from_str::<Level>("1").is_err());
    assert!(musli::json::from_str::<Level>(r#"{"Low":{}}"#).is_err());
}

#[test]
fn container_binary_parse_error() {
    // A string which does not parse, in a binary format.
    let bytes = musli::storage::to_vec("medium").unwrap();
    let error = musli::storage::from_slice::<Level>(&bytes).unwrap_err();
    assert!(
        error.to_string().contains("unknown level `medium`"),
        "{error}"
    );

    let bytes = musli::wire::to_vec(&42u32).unwrap();
    assert!(musli::wire::from_slice::<Level>(&bytes).is_err());
}

enum Alt {}

#[test]
fn container_any_mode() {
    const ALT: musli::json::Encoding<Alt> = musli::json::Encoding::new().with_mode();

    let out = ALT.to_string(&Level::High).unwrap();
    assert_eq!(out, r#""high""#);
    assert_eq!(ALT.from_str::<Level>(&out).unwrap(), Level::High);

    let out = ALT.to_string(&Id(7u8)).unwrap();
    assert_eq!(out, r#""id-7""#);
    assert_eq!(ALT.from_str::<Id<u8>>(&out).unwrap(), Id(7));
}
