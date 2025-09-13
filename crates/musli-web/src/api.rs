//! Shared traits for defining API types.

use core::fmt;
use core::num::NonZeroU16;

use musli::alloc::Global;
use musli::mode::Binary;
use musli::{Decode, Encode};

#[doc(inline)]
pub use musli_web_macros::define;

/// A trait for constructing identifiers.
pub trait Id
where
    Self: 'static + fmt::Debug,
{
    /// Construct an identifier from a raw `u16`.
    #[doc(hidden)]
    fn from_raw(id: u16) -> Option<Self>
    where
        Self: Sized;
}

/// The identifier of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[repr(transparent)]
#[musli(transparent)]
pub struct MessageId(NonZeroU16);

impl fmt::Display for MessageId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Start of special identifiers.
const START: u16 = i16::MAX as u16;

impl MessageId {
    /// The message id for [`ErrorMessage`].
    pub const ERROR_MESSAGE: Self = unsafe { Self::new_unchecked(START) };

    /// Try to construct a message id.
    #[doc(hidden)]
    #[inline]
    pub const fn new(id: u16) -> Option<Self> {
        let Some(value) = NonZeroU16::new(id) else {
            return None;
        };

        Some(Self(value))
    }

    /// Get a raw message identifier.
    #[doc(hidden)]
    #[inline]
    pub const fn get(&self) -> u16 {
        self.0.get()
    }

    /// Construct a new message ID.
    ///
    /// # Panics
    ///
    /// Panics if `id` is zero.
    #[doc(hidden)]
    #[inline]
    pub const unsafe fn new_unchecked(id: u16) -> Self {
        Self(unsafe { NonZeroU16::new_unchecked(id) })
    }
}

pub trait Endpoint
where
    Self: 'static,
{
    /// The kind of the endpoint.
    const ID: MessageId;

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
    const ID: MessageId;

    #[doc(hidden)]
    fn __do_not_implement_broadcast();
}

/// Trait implemented for broadcasts which have a primary event.
pub trait BroadcastWithEvent: Broadcast {
    /// The event type related to the broadcast.
    type Event<'de>: Event<Broadcast = Self> + Decode<'de, Binary, Global>
    where
        Self: 'de;

    #[doc(hidden)]
    fn __do_not_implement_broadcast_with_event();
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
#[musli(packed)]
pub struct RequestHeader {
    /// The serial of the request.
    pub serial: u32,
    /// The kind of the request.
    pub id: u16,
}

/// The header of a response.
#[derive(Debug, Clone, Encode, Decode)]
#[doc(hidden)]
#[musli(packed)]
pub struct ResponseHeader {
    /// The serial request this is a response to.
    pub serial: u32,
    /// This is a broadcast over the specified type. If this is non-empty the
    /// serial is 0.
    pub broadcast: u16,
    /// If non-zero, the response contains an error of the given type.
    pub error: u16,
}

/// An error response.
#[derive(Debug, Clone, Encode, Decode)]
#[doc(hidden)]
#[musli(packed)]
pub struct ErrorMessage<'de> {
    /// The error message.
    pub message: &'de str,
}
