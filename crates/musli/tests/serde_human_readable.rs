//! Test that `is_human_readable` is set for the `Text` mode.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Serde {
    #[musli(with = musli::serde)]
    ip: IpAddr,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StringField {
    ip: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[musli(name_type = str)]
enum IpRepr {
    #[musli(name = "V4", transparent)]
    V4([u8; 4]),
    #[musli(name = "V6", transparent)]
    V6([u8; 16]),
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Musli {
    ip: IpRepr,
}

#[test]
fn human_readable_ip_addr() {
    macro_rules! test_case {
        ($variant:ident, $ty:ty, $array:expr, $human:expr, $expected:literal $(, $args:expr)*) => {
            musli::macros::assert_decode_eq!(
                text_mode,
                Serde {
                    ip: IpAddr::$variant(<$ty>::new($($args),*))
                },
                StringField {
                    ip: String::from($human)
                },
                json = concat!(r#"{"ip":""#, $human, r#""}"#),
            );

            musli::macros::assert_decode_eq!(
                text_mode,
                StringField {
                    ip: String::from($human)
                },
                Serde {
                    ip: IpAddr::$variant(<$ty>::new($($args),*))
                },
                json = concat!(r#"{"ip":""#, $human, r#""}"#),
            );

            musli::macros::assert_decode_eq!(
                binary_mode,
                Serde {
                    ip: IpAddr::$variant(<$ty>::new($($args),*))
                },
                Musli {
                    ip: IpRepr::$variant($array)
                },
                json_binary = concat!(r#"{"0":{""#, stringify!($variant), r#"":"#, $expected, r#"}}"#),
            );

            musli::macros::assert_decode_eq!(
                binary_mode,
                Musli {
                    ip: IpRepr::$variant($array)
                },
                Serde {
                    ip: IpAddr::$variant(<$ty>::new($($args),*))
                },
                json_binary = concat!(r#"{"0":{""#, stringify!($variant), r#"":"#, $expected, r#"}}"#),
            );
        }
    }

    test_case! {
        V4, Ipv4Addr,
        [127,0,0,1],
        "127.0.0.1",
        "[127,0,0,1]",
        127, 0, 0, 1
    };

    test_case! {
        V6, Ipv6Addr,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xc0, 0x0a, 0x02, 0xff],
        "::ffff:192.10.2.255",
        "[0,0,0,0,0,0,0,0,0,0,255,255,192,10,2,255]",
        0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff
    };

    test_case! {
        V6, Ipv6Addr,
        [0, 38, 0, 0, 1, 201, 0, 0, 0, 0, 175, 200, 0, 16, 0, 1],
        "26:0:1c9::afc8:10:1",
        "[0,38,0,0,1,201,0,0,0,0,175,200,0,16,0,1]",
        0x26, 0, 0x1c9, 0, 0, 0xafc8, 0x10, 0x1
    };
}
