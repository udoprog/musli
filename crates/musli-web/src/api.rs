//! Shared traits for defining API types.

use musli::alloc::System;
use musli::mode::Binary;
use musli::{Decode, Encode};

#[macro_export]
macro_rules! __lifetime {
    ($lt:lifetime) => { $lt };
    () => { '__de };
}

#[doc(hidden)]
pub use __lifetime as lifetime;

#[macro_export]
macro_rules! __marker {
    ($lt:lifetime, $ty:ty) => {
        type Type<$lt> = $ty;
    };
    (, $ty:ty) => {
        $crate::api::marker!('__de, $name, $ty);
    };
}

#[doc(hidden)]
pub use __marker as marker;

#[macro_export]
macro_rules! __define {
    (
        $(
            endpoint $endpoint:ident {
                request $(<$request_lt:lifetime>)? = $request:ty;
                response $(<$response_lt:lifetime>)? = $response:ty;
            }
        )*

        $(
            broadcast $broadcast:ident {
                body $(<$body_lt:lifetime>)? = $body:ty;
            }
        )*
    ) => {
        $(
            pub enum $endpoint {}

            impl $crate::api::Marker for $endpoint {
                $crate::api::marker!($($response_lt)*, $response);
            }

            impl $(<$request_lt>)* $crate::api::Request for $request {
                const KIND: &'static str = stringify!($endpoint);
                type Marker = $endpoint;
            }
        )*

        $(
            pub enum $broadcast {}

            impl $crate::api::Broadcast for $broadcast {
                const KIND: &'static str = stringify!($broadcast);
            }

            impl $crate::api::Marker for $broadcast {
                $crate::api::marker!($($body_lt)*, $body);
            }
        )*
    };
}

pub use __define as define;

/// A marker indicating a decodable type.
pub trait Marker: 'static {
    /// The type that can be decoded.
    type Type<'de>: Decode<'de, Binary, System>;
}

/// Trait governing requests.
pub trait Request: Encode<Binary> {
    /// The kind of the request.
    const KIND: &'static str;

    /// Type acting as a token for the response.
    type Marker: Marker;
}

/// A broadcast type marker.
pub trait Broadcast: Marker {
    /// The kind of the broadcast being subscribed to.
    const KIND: &'static str;
}

/// A request header.
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct RequestHeader<'a> {
    /// The index of the request.
    pub index: u32,
    /// The serial number of the request.
    pub serial: u32,
    /// The kind of the request.
    pub kind: &'a str,
}

/// The header of a response.
#[derive(Debug, Clone, Encode, Decode)]
pub struct ResponseHeader<'de> {
    /// The index of the request this is a response to.
    pub index: u32,
    /// The serial number of the request this is a response to.
    pub serial: u32,
    /// This is a broadcast over the specified topic.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub broadcast: Option<&'de str>,
    /// An error message in the response.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub error: Option<&'de str>,
}
