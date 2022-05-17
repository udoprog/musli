use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use musli::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct A {
    pub id: [u8; 16],
    pub ip: IpAddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct B {
    pub id: [u8; 16],
    pub user_id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct C {
    pub id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct D {
    pub id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct E {
    pub id: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum LargeEnumStringVariants {
    #[musli(transparent, rename = "a")]
    A(A),
    #[musli(transparent, rename = "b")]
    B(B),
    #[musli(transparent, rename = "c")]
    C(C),
    #[musli(transparent, rename = "d")]
    D(D),
    #[musli(transparent, rename = "e")]
    E(E),
}

#[test]
fn test_large_enum_string_variants() {
    const ID: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    const USER_ID: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    const IP: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
    const IPV6: IpAddr = IpAddr::V6(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8));

    musli_tests::rt!(LargeEnumStringVariants::A(A { id: ID, ip: IP }));
    musli_tests::rt!(LargeEnumStringVariants::A(A { id: ID, ip: IPV6 }));
    musli_tests::rt!(LargeEnumStringVariants::B(B {
        id: ID,
        user_id: USER_ID
    }));
    musli_tests::rt!(LargeEnumStringVariants::C(C { id: ID }));
    musli_tests::rt!(LargeEnumStringVariants::D(D { id: ID }));
    musli_tests::rt!(LargeEnumStringVariants::E(E { id: ID }));
}
