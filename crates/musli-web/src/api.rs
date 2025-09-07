//! Shared traits for defining API types.

use musli::alloc::Global;
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
    };

    (@inner broadcast $listener:ident {
        event <$first_event_lt:lifetime> = $first_event:ty;
        $(event $(<$event_lt:lifetime>)? = $event:ty;)*
    }) => {
        pub enum $listener {}

        impl $crate::api::Listener for $listener {
            const KIND: &'static str = stringify!($listener);
            type Broadcast<$first_event_lt> = $first_event;
        }

        impl <$first_event_lt> $crate::api::Broadcast for $first_event {
            type Endpoint = $listener;
        }

        $(
            impl $(<$event_lt>)* $crate::api::Broadcast for $event {
                type Endpoint = $listener;
            }
        )*
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
/// * [`web::Request`]
/// * [`web::Listener`]
///
/// [`web::Request`]: crate::web::Request
/// [`web::Listener`]: crate::web::Listener
///
/// These are in turn extended with the relevant API using them:
///
/// * [`yew021`] for yew `0.21.x`.
///
/// [`yew021`]: crate::yew021
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
/// pub struct TickEvent<'de> {
///     pub message: &'de str,
///     pub tick: u32,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct OwnedTickEvent {
///     pub message: String,
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
///         event<'de> = TickEvent<'de>;
///         event = OwnedTickEvent;
///     }
/// }
/// ```
#[doc(inline)]
pub use __define as define;

pub trait Endpoint
where
    Self: 'static,
{
    /// The kind of the response.
    const KIND: &'static str;

    /// The response type related to the endpoint.
    type Response<'de>: Decode<'de, Binary, Global>;
}

pub trait Listener
where
    Self: 'static,
{
    /// The kind of the response.
    const KIND: &'static str;

    /// The response type related to the endpoint.
    type Broadcast<'de>: Broadcast<Endpoint = Self> + Decode<'de, Binary, Global>;
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
pub trait Broadcast
where
    Self: Encode<Binary>,
{
    /// The endpoint related to the broadcast.
    type Endpoint: Listener;
}

/// A request header.
#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct RequestHeader<'a> {
    /// The serial of the request.
    pub serial: u32,
    /// The kind of the request.
    pub kind: &'a str,
}

/// The header of a response.
#[derive(Debug, Clone, Encode, Decode)]
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
