//! Shared traits for defining API types.

use musli::alloc::Global;
use musli::mode::Binary;
use musli::{Decode, Encode};

#[doc(inline)]
pub use musli_web_macros::define;

pub trait Endpoint
where
    Self: 'static,
{
    /// The kind of the endpoint.
    const KIND: &'static str;

    /// The primary response type related to the endpoint.
    type Response<'de>: Decode<'de, Binary, Global>;

    #[doc(hidden)]
    fn __do_not_implement_endpoint();
}

/// The marker trait used for broadcasts.
///
/// Do not implement manually, instead use the [`define!`] macro.
pub trait Broadcast
where
    Self: 'static,
{
    /// The kind of the broadcast.
    const KIND: &'static str;

    /// The primary event related to the broadcast.
    type Event<'de>: Event<Broadcast = Self> + Decode<'de, Binary, Global>;

    #[doc(hidden)]
    fn __do_not_implement_broadcast();
}

/// A marker indicating a request type.
///
/// Do not implement manually, instead use the [`define!`] macro.
pub trait Request
where
    Self: Encode<Binary>,
{
    /// The endpoint related to the request.
    type Endpoint: Endpoint;

    #[doc(hidden)]
    fn __do_not_implement_request();
}

/// The event of a broadcast.
///
/// Do not implement manually, instead use the [`define!`] macro.
pub trait Event
where
    Self: Encode<Binary>,
{
    /// The endpoint related to the broadcast.
    type Broadcast: Broadcast;

    #[doc(hidden)]
    fn __do_not_implement_event();
}

/// A request header.
#[derive(Debug, Clone, Copy, Encode, Decode)]
#[doc(hidden)]
pub struct RequestHeader<'a> {
    /// The serial of the request.
    pub serial: u32,
    /// The kind of the request.
    pub kind: &'a str,
}

/// The header of a response.
#[derive(Debug, Clone, Encode, Decode)]
#[doc(hidden)]
pub struct ResponseHeader<'de> {
    /// The serial request this is a response to.
    pub serial: u32,
    /// This is a broadcast over the specified topic. If this is set, then
    /// serial is `0`.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub broadcast: Option<&'de str>,
    /// An error message in the response.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub error: Option<&'de str>,
}
