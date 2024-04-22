use std::net::IpAddr;

use musli::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct A {
    #[musli(bytes)]
    pub id: [u8; 16],
    pub ip: IpAddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct B {
    #[musli(bytes)]
    pub id: [u8; 16],
    #[musli(bytes)]
    pub user_id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct C {
    #[musli(bytes)]
    pub id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct D {
    #[musli(bytes)]
    pub id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct E {
    #[musli(bytes)]
    pub id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "name")]
pub enum LargeEnumStringVariants {
    #[musli(transparent, name = "a")]
    A(A),
    #[musli(transparent, name = "b")]
    B(B),
    #[musli(transparent, name = "c")]
    C(C),
    #[musli(transparent, name = "d")]
    D(D),
    #[musli(transparent, name = "e")]
    E(E),
}

#[test]
fn large_enum_string_variants() {
    use std::net::{Ipv4Addr, Ipv6Addr};

    const ID: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    const USER_ID: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    const IP: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
    const IPV6: IpAddr = IpAddr::V6(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8));

    // TODO: Fix this for JSON.
    musli::rt!(no_json, LargeEnumStringVariants::A(A { id: ID, ip: IP }));
    musli::rt!(no_json, LargeEnumStringVariants::A(A { id: ID, ip: IPV6 }));
    musli::rt!(
        no_json,
        LargeEnumStringVariants::B(B {
            id: ID,
            user_id: USER_ID
        })
    );
    musli::rt!(no_json, LargeEnumStringVariants::C(C { id: ID }));
    musli::rt!(no_json, LargeEnumStringVariants::D(D { id: ID }));
    musli::rt!(no_json, LargeEnumStringVariants::E(E { id: ID }));
}
