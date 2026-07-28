//! Shared traits for defining API types.

use core::fmt;
use core::num::NonZeroU16;
use core::sync::atomic::{AtomicU16, Ordering};

use musli::alloc::Global;
use musli::mode::Binary;
use musli::{Decode, Encode};

#[doc(inline)]
pub use musli_web_macros::define;

/// A trait for constructing identifiers.
pub trait Id
where
    Self: 'static + Send + Sized + fmt::Debug,
{
    /// Get the raw message identifier for this type.
    fn id(&self) -> MessageId;

    /// Construct an identifier from a raw message identifier.
    fn from_id(id: MessageId) -> Self;

    #[doc(hidden)]
    fn __do_not_implement_id();
}

/// A unique and opaque identifier for a channel over the websocket.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[musli(transparent)]
pub struct ChannelId {
    repr: u16,
}

impl ChannelId {
    /// The channel id used for an invalid channel.
    pub const NONE: Self = Self::from_u16(0);

    /// Construct a new channel id from a raw `u16` representation.
    ///
    /// Note that this does not guarantee that the internal representation of a
    /// channel identifier is exactly a `u16`, only that at least `u16` unique
    /// identifiers can be constructed.
    ///
    /// Using `0` is equivalent to [`ChannelId::NONE`]. When implementing a
    /// custom [`ChannelAllocator`] the allocator must avoid constructor
    /// identifiers with this value since it is equivalent to no channel.
    ///
    /// [`ChannelAllocator`]: crate::ws::ChannelAllocator
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::ChannelId;
    /// let id = ChannelId::from_u16(0);
    /// assert_eq!(id, ChannelId::NONE);
    /// ```
    #[inline]
    pub const fn from_u16(repr: u16) -> Self {
        Self { repr }
    }

    /// Get the raw channel identifier.
    #[inline]
    #[cfg(feature = "ws")]
    pub(crate) const fn raw(&self) -> u16 {
        self.repr
    }
}

impl fmt::Debug for ChannelId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.repr == 0 {
            f.write_str("NONE")
        } else {
            write!(f, "{:04x}", self.repr)
        }
    }
}

/// A [`ChannelId`] which can be shared and updated atomically.
///
/// This behaves like an atomic variable holding a [`ChannelId`], where the
/// value can be read, set, replaced, or taken. Each operation takes an
/// [`Ordering`] which is passed through to the underlying atomic.
///
/// [`Ordering`]: core::sync::atomic::Ordering
///
/// # Examples
///
/// ```
/// use core::sync::atomic::Ordering;
///
/// use musli_web::api::{AtomicChannelId, ChannelId};
///
/// let channel = AtomicChannelId::NONE;
/// assert_eq!(channel.load(Ordering::Acquire), ChannelId::NONE);
///
/// channel.store(ChannelId::from_u16(42), Ordering::Release);
/// assert_eq!(channel.load(Ordering::Acquire), ChannelId::from_u16(42));
///
/// assert_eq!(channel.take(Ordering::AcqRel), ChannelId::from_u16(42));
/// assert_eq!(channel.load(Ordering::Acquire), ChannelId::NONE);
/// ```
#[repr(transparent)]
pub struct AtomicChannelId {
    repr: AtomicU16,
}

impl AtomicChannelId {
    /// An atomic channel id which contains [`ChannelId::NONE`].
    ///
    /// Since this is a constant every use of it constructs a new value, to
    /// share one it has to be bound to a `static` or a variable.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::NONE;
    /// assert_eq!(channel.load(Ordering::Acquire), ChannelId::NONE);
    /// ```
    #[allow(clippy::declare_interior_mutable_const)]
    pub const NONE: Self = Self::new(ChannelId::NONE);

    /// Construct a new atomic channel id containing `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::new(ChannelId::from_u16(1));
    /// assert_eq!(channel.load(Ordering::Acquire), ChannelId::from_u16(1));
    /// ```
    #[inline]
    pub const fn new(id: ChannelId) -> Self {
        Self {
            repr: AtomicU16::new(id.repr),
        }
    }

    /// Read the current channel id.
    ///
    /// `ordering` describes the memory ordering of this operation. Possible
    /// values are [`SeqCst`], [`Acquire`] and [`Relaxed`].
    ///
    /// [`SeqCst`]: Ordering::SeqCst
    /// [`Acquire`]: Ordering::Acquire
    /// [`Relaxed`]: Ordering::Relaxed
    ///
    /// # Panics
    ///
    /// Panics if `ordering` is [`Release`] or [`AcqRel`].
    ///
    /// [`Release`]: Ordering::Release
    /// [`AcqRel`]: Ordering::AcqRel
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::new(ChannelId::from_u16(1));
    /// assert_eq!(channel.load(Ordering::Acquire), ChannelId::from_u16(1));
    /// ```
    #[inline]
    pub fn load(&self, ordering: Ordering) -> ChannelId {
        ChannelId::from_u16(self.repr.load(ordering))
    }

    /// Set the current channel id to `id`, discarding the old value.
    ///
    /// `ordering` describes the memory ordering of this operation. Possible
    /// values are [`SeqCst`], [`Release`] and [`Relaxed`].
    ///
    /// [`SeqCst`]: Ordering::SeqCst
    /// [`Release`]: Ordering::Release
    /// [`Relaxed`]: Ordering::Relaxed
    ///
    /// # Panics
    ///
    /// Panics if `ordering` is [`Acquire`] or [`AcqRel`].
    ///
    /// [`Acquire`]: Ordering::Acquire
    /// [`AcqRel`]: Ordering::AcqRel
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::NONE;
    /// channel.store(ChannelId::from_u16(1), Ordering::Release);
    /// assert_eq!(channel.load(Ordering::Acquire), ChannelId::from_u16(1));
    /// ```
    #[inline]
    pub fn store(&self, id: ChannelId, ordering: Ordering) {
        self.repr.store(id.repr, ordering);
    }

    /// Replace the current channel id with `id`, returning the old value.
    ///
    /// `ordering` describes the memory ordering of this operation. All
    /// orderings are possible. Note that using [`Acquire`] makes the store part
    /// of this operation [`Relaxed`], and using [`Release`] makes the load part
    /// [`Relaxed`].
    ///
    /// [`Acquire`]: Ordering::Acquire
    /// [`Release`]: Ordering::Release
    /// [`Relaxed`]: Ordering::Relaxed
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::new(ChannelId::from_u16(1));
    ///
    /// let old = channel.replace(ChannelId::from_u16(2), Ordering::AcqRel);
    /// assert_eq!(old, ChannelId::from_u16(1));
    /// assert_eq!(channel.load(Ordering::Acquire), ChannelId::from_u16(2));
    /// ```
    #[inline]
    pub fn replace(&self, id: ChannelId, ordering: Ordering) -> ChannelId {
        ChannelId::from_u16(self.repr.swap(id.repr, ordering))
    }

    /// Take the current channel id, leaving [`ChannelId::NONE`] in its place.
    ///
    /// `ordering` describes the memory ordering of this operation. All
    /// orderings are possible, see [`replace`] for details.
    ///
    /// [`replace`]: AtomicChannelId::replace
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::new(ChannelId::from_u16(1));
    ///
    /// assert_eq!(channel.take(Ordering::AcqRel), ChannelId::from_u16(1));
    /// assert_eq!(channel.load(Ordering::Acquire), ChannelId::NONE);
    /// assert_eq!(channel.take(Ordering::AcqRel), ChannelId::NONE);
    /// ```
    #[inline]
    pub fn take(&self, ordering: Ordering) -> ChannelId {
        self.replace(ChannelId::NONE, ordering)
    }

    /// Consume the atomic channel id, returning the contained value.
    ///
    /// Since this takes ownership no synchronization is needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::new(ChannelId::from_u16(1));
    /// assert_eq!(channel.into_inner(), ChannelId::from_u16(1));
    /// ```
    #[inline]
    pub fn into_inner(self) -> ChannelId {
        ChannelId::from_u16(self.repr.into_inner())
    }
}

impl Default for AtomicChannelId {
    /// Construct an atomic channel id containing [`ChannelId::NONE`].
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use musli_web::api::{AtomicChannelId, ChannelId};
    ///
    /// let channel = AtomicChannelId::default();
    /// assert_eq!(channel.load(Ordering::Acquire), ChannelId::NONE);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

impl From<ChannelId> for AtomicChannelId {
    #[inline]
    fn from(id: ChannelId) -> Self {
        Self::new(id)
    }
}

impl fmt::Debug for AtomicChannelId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.load(Ordering::Relaxed).fmt(f)
    }
}

/// A raw identifier for a message.
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

impl MessageId {
    /// The message id for [`ErrorMessage`].
    pub const ERROR_MESSAGE: Self = unsafe { Self::new_unchecked((i16::MAX as u16) + 1) };

    /// A connect of a channel.
    pub const CONNECT: Self = unsafe { Self::new_unchecked((i16::MAX as u16) + 2) };

    /// A clean disconnect of a channel.
    pub const DISCONNECT: Self = unsafe { Self::new_unchecked((i16::MAX as u16) + 3) };

    /// The first message the server sends to indicat that a connection is open.
    pub const SERVER_HELLO: Self = unsafe { Self::new_unchecked((i16::MAX as u16) + 4) };

    /// The message id for an empty packet constructed using [`Packet::empty`]
    /// or [`RawPacket::empty`].
    ///
    /// [`Packet::empty`]: crate::web::Packet::empty
    /// [`RawPacket::empty`]: crate::web::RawPacket::empty
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::MessageId;
    /// use musli_web::web::{RawPacket, Packet};
    ///
    /// let packet = RawPacket::empty();
    /// assert_eq!(packet.id(), MessageId::EMPTY);
    ///
    /// let packet = Packet::<()>::empty();
    /// assert_eq!(packet.id(), MessageId::EMPTY);
    /// ```
    pub const EMPTY: Self = unsafe { Self::new_unchecked(u16::MAX) };

    /// Construct a raw message id.
    #[inline]
    pub const fn new(id: u16) -> Option<Self> {
        let Some(value) = NonZeroU16::new(id) else {
            return None;
        };

        Some(Self(value))
    }

    /// Get a raw message identifier.
    #[inline]
    pub const fn get(&self) -> u16 {
        self.0.get()
    }

    /// Construct a new message ID.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided `id` is non-zero.
    #[inline]
    pub const unsafe fn new_unchecked(id: u16) -> Self {
        Self(unsafe { NonZeroU16::new_unchecked(id) })
    }
}

/// A trait implemented for types which can be decoded into something.
///
/// Do not implement manually, instead use the [`define!`] macro.
pub trait Decodable {
    /// The decodable type related to this.
    type Type<'de>: Decode<'de, Binary, Global>;

    #[doc(hidden)]
    fn __do_not_implement_decodable();
}

/// An endpoint marker trait.
///
/// Do not implement manually, instead use the [`define!`] macro.
pub trait Endpoint
where
    Self: 'static,
    for<'de> Self: Decodable<Type<'de> = Self::Response<'de>>,
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
pub trait BroadcastWithEvent
where
    Self: Broadcast,
    for<'de> Self: Decodable<Type<'de> = Self::Event<'de>>,
{
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

/// A request to connect.
#[derive(Debug, Clone, Copy, Encode, Decode)]
#[doc(hidden)]
#[musli(packed)]
pub struct Connect;

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
    /// The channel over which the response will be sent.
    pub channel: ChannelId,
}

/// An error response.
#[derive(Debug, Clone, Encode, Decode)]
#[doc(hidden)]
#[musli(packed)]
pub struct ErrorMessage<'de> {
    /// The error message.
    pub message: &'de str,
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
    /// The channel over which the request was received.
    pub channel: ChannelId,
}
