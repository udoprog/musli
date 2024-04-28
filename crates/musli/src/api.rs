//! API definitions for Musli.
//!
//! This provides types and traits for defining a simple binary strictly-typed
//! interchange API.

use crate::{Decode, Encode};

#[cfg(feature = "storage")]
mod encoding;
#[cfg(feature = "storage")]
pub use self::encoding::*;

#[cfg(test)]
mod tests;

/// Define an endpoint with a well-known name and a request and response type.
///
/// This derive requires one unique identifiers and a couple of types to be
/// designated. Once provider it defines an enum marker type that implements the
/// [`Endpoint`] trait.
/// * The unique identifier of the request, like `"ping"`, if this is not
///   specified using the `name = "..."` attribute it will default to the name
///   of the endpoint type in lower kebab-case.
/// * The response type which must implement `Encode` and `Decode` and can
///   optionally take a lifetime. This is specified with the
///   `#[endpoint(response = <ty>)]` or `#[endpoint(response<'de> = <ty>)]`
///   attribute. Responses can have a lifetime since it allows for local buffers
///   to be re-used and avoiding copies.
///
/// The overall structure of the derive is as follows:
///
/// ```
/// # use musli::{Encode, Decode};
/// # use musli::api::Request;
/// # #[derive(Request, Encode, Decode)]
/// # #[request(endpoint = Hello)]
/// # pub struct HelloRequest;
/// # #[derive(Encode, Decode)]
/// # pub struct HelloResponse<'de> { _marker: core::marker::PhantomData<&'de ()> }
/// use musli::api::Endpoint;
///
/// #[derive(Endpoint)]
/// #[endpoint(response<'de> = HelloResponse<'de>)]
/// pub enum Hello {}
/// ```
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::api::{Endpoint, Request};
///
/// #[derive(Endpoint)]
/// #[endpoint(response = Pong)]
/// pub enum PingPong {}
///
/// #[derive(Request, Encode, Decode)]
/// #[request(endpoint = PingPong)]
/// pub struct Ping(u32);
///
/// #[derive(Encode, Decode)]
/// pub struct Pong(u32);
///
/// #[derive(Encode, Decode)]
/// pub struct MessageOfTheDayResponse<'de> {
///     pub message_of_the_day: &'de str,
/// }
///
/// #[derive(Request, Encode)]
/// #[request(endpoint = MessageOfTheDay)]
/// pub struct MessageOfTheDayRequest;
///
/// #[derive(Endpoint)]
/// #[endpoint(response<'de> = MessageOfTheDayResponse<'de>)]
/// pub enum MessageOfTheDay {}
/// ```
#[doc(inline)]
pub use musli_macros::Endpoint;

/// Define an request and the endpoint it is associated with.
///
/// The overall structure of the derive is as follows:
///
/// ```
/// # #[derive(Encode, Decode)]
/// # pub struct HelloResponse<'de> { _marker: core::marker::PhantomData<&'de ()> }
/// use musli::{Encode, Decode};
/// use musli::api::{Endpoint, Request};
///
/// #[derive(Request, Encode, Decode)]
/// #[request(endpoint = Hello)]
/// pub struct HelloRequest;
///
/// #[derive(Endpoint)]
/// #[endpoint(response<'de> = HelloResponse<'de>)]
/// pub enum Hello {}
/// ```
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::api::{Endpoint, Request};
///
/// #[derive(Endpoint)]
/// #[endpoint(response = Pong)]
/// pub enum PingPong {}
///
/// #[derive(Request, Encode, Decode)]
/// #[request(endpoint = PingPong)]
/// pub struct Ping(u32);
///
/// #[derive(Encode, Decode)]
/// pub struct Pong(u32);
///
/// #[derive(Encode, Decode)]
/// pub struct MessageOfTheDayResponse<'de> {
///     pub message_of_the_day: &'de str,
/// }
///
/// #[derive(Request, Encode)]
/// #[request(endpoint = MessageOfTheDay)]
/// pub struct MessageOfTheDayRequest;
///
/// #[derive(Endpoint)]
/// #[endpoint(response<'de> = MessageOfTheDayResponse<'de>)]
/// pub enum MessageOfTheDay {}
/// ```
#[doc(inline)]
pub use musli_macros::Request;

/// A trait implemented for marker types for endpoints.
///
/// These types can be used to statically associate endpoints with their
/// responses, which can be useful for clients. They can unambigiously make use
/// of the marker type to get access to the kind of the endpoint for routing
/// purposes and the response type.
///
/// You must only use the [derive@Endpoint] macro to implement this trait.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::api::{Endpoint, Request};
///
/// #[derive(Encode, Decode)]
/// pub struct MessageOfTheDayResponse<'de> {
///     pub message_of_the_day: &'de str,
/// }
///
/// #[derive(Request, Encode)]
/// #[request(endpoint = MessageOfTheDay)]
/// pub struct MessageOfTheDayRequest;
///
/// #[derive(Endpoint)]
/// #[endpoint(response<'de> = MessageOfTheDayResponse<'de>)]
/// pub enum MessageOfTheDay {}
/// ```
pub trait Endpoint: 'static {
    /// The name of the endpoint.
    const KIND: &'static str;

    /// The response type of the endpoint.
    ///
    /// The lifetime allows the response to make use of local buffers to avoid
    /// copying the underlying data unecessarily.
    type Response<'de>;

    /// Marker function to indicate that the request should not be implemented
    /// by hand.
    #[doc(hidden)]
    fn __do_not_implement();
}

/// Trait governing requests.
///
/// Requests are associated with a single endpoint, which lets them statically
/// know both the type of the endpoint and the response type.
///
/// You must only use the [derive@Request] macro to implement this trait.
///
/// # Examples
///
/// ```
/// use musli::{Encode, Decode};
/// use musli::api::{Endpoint, Request};
///
/// #[derive(Encode, Decode)]
/// pub struct MessageOfTheDayResponse<'de> {
///     pub message_of_the_day: &'de str,
/// }
///
/// #[derive(Request, Encode)]
/// #[request(endpoint = MessageOfTheDay)]
/// pub struct MessageOfTheDayRequest;
///
/// #[derive(Endpoint)]
/// #[endpoint(response<'de> = MessageOfTheDayResponse<'de>)]
/// pub enum MessageOfTheDay {}
/// ```
pub trait Request {
    /// The endpoint the request is associated with.
    type Endpoint: Endpoint;

    /// Marker function to indicate that the request should not be implemented
    /// by hand.
    #[doc(hidden)]
    fn __do_not_implement();
}

/// The API header of a request.
#[derive(Debug, Clone, Copy, Encode, Decode)]
#[musli(crate)]
pub struct RequestHeader<'a> {
    /// Identifier of the request.
    pub serial: u64,
    /// The kind of the request.
    pub kind: &'a str,
}

/// The API header of a response.
#[derive(Debug, Clone, Encode, Decode)]
#[musli(crate)]
pub struct ResponseHeader<'de> {
    /// Identifier of the request this response belongs to.
    pub serial: u64,
    /// The response is a broadcast belonging to the given type.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub broadcast: Option<&'de str>,
    /// An error message in the response.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub error: Option<&'de str>,
}
