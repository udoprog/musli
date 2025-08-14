//! Shared traits for defining API types.

use musli::alloc::System;
use musli::mode::Binary;
use musli::{Decode, Encode};

#[macro_export]
#[doc(hidden)]
macro_rules! __define {
    ($($what:ident $endpoint:ident { $($tt:tt)* })*) => {
        $($crate::api::define!(@inner $what $endpoint { $($tt)* });)*
    };

    (@inner endpoint $endpoint:ident {
        request $(<$request_lt:lifetime>)? = $request:ty;
        response<$response_lt:lifetime> = $response:ty;
    }) => {
        pub enum $endpoint {}

        impl $crate::api::Endpoint for $endpoint {
            const KIND: &'static str = stringify!($endpoint);
            type Response<$response_lt> = $response;
        }

        impl $(<$request_lt>)* $crate::api::Request for $request {
            type Endpoint = $endpoint;
        }

        impl<$response_lt> $crate::api::Response<$response_lt> for $response {
            type Endpoint = $endpoint;
        }
    };

    (@inner broadcast $broadcast:ident {
        body<$body_lt:lifetime> = $body:ty;
    }) => {
        pub enum $broadcast {}

        impl $crate::api::BroadcastEndpoint for $broadcast {
            const KIND: &'static str = stringify!($broadcast);
            type Broadcast<$body_lt> = $body;
        }

        impl<$body_lt>  $crate::api::Broadcast<$body_lt> for $body {
            type Endpoint = $broadcast;
        }
    }
}

/// Define API types.
///
/// Defining an `endpoint` causes a type to be generated which is a marker type
/// for the endpoint, which binds together the request and response types.
///
/// Defining a broadcast simply associated a broadcast with a marker type.
///
/// The marker type is used with the various types used when interacting with an
/// API endpoint or broadcast, such as:
///
/// * [`yew021::Request<T>`] where `T: Endpoint`.
/// * [`yew021::Listener<T>`] where `T: BroadcastEndpoint`.
///
/// [`yew021::Request<T>`]: crate::yew021::Request
/// [`yew021::Listener<T>`]: crate::yew021::Listener
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli_web::api;
///
/// #[derive(Encode, Decode)]
/// pub struct HelloRequest<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct HelloResponse<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct TickBody<'de> {
///     pub message: &'de str,
///     pub tick: u32,
/// }
///
/// api::define! {
///     endpoint Hello {
///         request<'de> = HelloRequest<'de>;
///         response<'de> = HelloResponse<'de>;
///     }
///
///     broadcast Tick {
///         body<'de> = TickBody<'de>;
///     }
/// }
/// ```
#[doc(inline)]
pub use __define as define;

pub trait Endpoint {
    /// The kind of the response.
    const KIND: &'static str;

    /// The response type related to the endpoint.
    type Response<'de>: Response<'de, Endpoint = Self>;
}

pub trait BroadcastEndpoint {
    /// The kind of the response.
    const KIND: &'static str;

    /// The response type related to the endpoint.
    type Broadcast<'de>: Broadcast<'de, Endpoint = Self>;
}

/// A marker indicating a response type.
pub trait Response<'de>
where
    Self: Decode<'de, Binary, System>,
{
    /// The endpoint related to the response.
    type Endpoint: Endpoint;
}

/// A marker indicating a request type.
pub trait Request
where
    Self: Encode<Binary>,
{
    /// The endpoint related to the request.
    type Endpoint: Endpoint;
}

/// A broadcast type marker.
pub trait Broadcast<'de>
where
    Self: Encode<Binary> + Decode<'de, Binary, System>,
{
    /// The endpoint related to the broadcast.
    type Endpoint: BroadcastEndpoint<Broadcast<'de> = Self>;
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
