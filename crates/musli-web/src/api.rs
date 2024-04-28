//! Shared traits for defining API types.

use musli::alloc::System;
use musli::mode::Binary;
use musli::{Decode, Encode};

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

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct RequestHeader<'a> {
    pub index: u32,
    pub serial: u32,
    /// The kind of the request.
    pub kind: &'a str,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResponseHeader<'de> {
    pub index: u32,
    pub serial: u32,
    /// The response is a broadcast.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub broadcast: Option<&'de str>,
    /// An error message in the response.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub error: Option<&'de str>,
}
