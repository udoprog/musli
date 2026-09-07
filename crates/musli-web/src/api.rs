//! Shared traits for defining API types.
//!
//! # Wire format
//!
//! Every websocket message consists of a fixed *envelope* followed by an
//! optional *body*:
//!
//! ```text
//! +--------------------------+--------------------------------+
//! | envelope (api::Mode)     | body (negotiated api::Format)  |
//! +--------------------------+--------------------------------+
//! ```
//!
//! The envelope is a block of *headers*: a [`RequestHeader`] for messages sent
//! by the client and a [`ResponseHeader`] for messages sent by the server. How
//! it is spelled is decided by the [`Mode`] the socket runs in and never
//! changes with the negotiated format. This is what makes the format negotiable
//! at all, since both peers can always read the envelope regardless of what
//! they have agreed on for bodies.
//!
//! The body is encoded with the [`Format`] identified by the `format` header of
//! the envelope, so every message is self-describing in this respect. A `format`
//! of zero means the message carries no body.
//!
//! # Headers
//!
//! Every header carries an unsigned integer and is addressed by a name. A
//! header which is absent reads as zero, and headers may arrive in any order,
//! so a message which has no use for a header simply does not write one.
//!
//! Both spellings are *self-delimiting*: a reader can find the end of a header
//! it has never seen without understanding what it means. That is deliberate,
//! and it is what makes the next rule possible.
//!
//! **A header which this build does not know about is refused.** It is read,
//! stepped over so that the position of everything after it is still known, and
//! then reported as an error which tears the connection down. A peer writing a
//! header we have never seen is a peer speaking a protocol we do not have, and
//! acting on the part of the message we happen to recognize would mean acting
//! on a message we have not actually understood. See [the protocol version] for
//! how peers avoid getting into that situation in the first place.
//!
//! [the protocol version]: crate::api#the-protocol-version
//!
//! # The binary envelope
//!
//! In [`Mode::Binary`] each header is a tag byte followed by its value, and a
//! tag byte of zero ends the block:
//!
//! ```text
//! +-----+---------------+     +-----+
//! | tag | value (1/2/4) | ... |  0  |
//! +-----+---------------+     +-----+
//! ```
//!
//! The low six bits of the tag are the header's id, which is never zero. The
//! top two bits say how wide the value is — `0` for one byte, `1` for two and
//! `2` for four, little endian — which is what lets an unknown header be
//! stepped over. A value is written in the narrowest of those which holds it,
//! so a small value costs no more than it has to. The remaining class is
//! reserved, and a header which uses it cannot be skipped, so it is refused.
//!
//! # The text envelope
//!
//! In [`Mode::Text`] the envelope is instead a block of `<key>: <value>` lines
//! terminated by an empty line, carried in a text frame:
//!
//! ```text
//! serial: 1
//! id: 1
//! format: 5
//! channel: 0
//!
//! {"message":"Hello!"}
//! ```
//!
//! Every header is written, in the order it is declared, with its value in
//! decimal. Being explicit is worth more than being terse in a spelling meant
//! to be read by people, so a header which is zero is written out too.
//!
//! Nothing about the message is otherwise different, which means the mode only
//! decides how the frame is *spelled*. Since a text frame has to be valid UTF-8
//! in its entirety the body has to be human readable too, so [`Mode::Text`]
//! requires a [`Format`] for which [`Format::is_human_readable`] holds. This is
//! what [`Mode::accepts`] tests.
//!
//! # Negotiating the mode
//!
//! Unlike the format, the mode is never carried in a field. The frame type
//! *is* the mode, so a peer can always tell which envelope it is looking at,
//! and a message can be read before anything has been agreed on.
//!
//! A client sends its [`MessageId::NEGOTIATE`] request in the mode it wants,
//! and the server replies in the mode it settled on. A server which is asked
//! for [`Mode::Text`] together with a format it cannot carry that way settles
//! on [`Mode::DEFAULT`], exactly like it does for a format it does not support.
//! The client adopts whichever mode the reply arrived in, and the mode is then
//! fixed for the lifetime of the connection.
//!
//! The one message which precedes all of this is
//! [`MessageId::SERVER_HELLO`], which is always sent in [`Mode::DEFAULT`] since
//! the server has not heard from the client yet.
//!
//! # Negotiating the format
//!
//! A client picks the [`Format`] it wants to use, defaulting to
//! [`Format::DEFAULT`]. The exchange is:
//!
//! 1. The server sends [`MessageId::SERVER_HELLO`] as soon as the connection is
//!    established. This carries no body.
//! 2. The client responds with a [`MessageId::NEGOTIATE`] request whose
//!    envelope carries the desired [`Format`]. This carries no body.
//! 3. If the server supports that format it records it for the connection and
//!    replies with an empty response whose envelope carries the format that was
//!    accepted. If it does not, it replies with an error listing the formats it
//!    does support, and the connection settles on [`Format::DEFAULT`].
//!
//! Both peers are forced through this before anything else can happen:
//!
//! * A client only reports itself as connected once step 3 has resolved.
//! * A server has no way to write a message until then either, since
//!   [`ws::Connect::connect`] is what produces the [`ws::Server`] which can, and
//!   it does not resolve until step 3 has been flushed. Any other message
//!   arriving in place of step 2 is a protocol violation which closes the
//!   connection.
//!
//! Together this guarantees that server-initiated messages such as broadcasts
//! are encoded with a format the client understands. The format is fixed for
//! the lifetime of a connection, so a second negotiation is refused — a client
//! which wants a different one reconnects.
//!
//! Requests additionally carry their own format in the envelope, so the server
//! decodes each request body with the format that request declares and replies
//! in the same format. Negotiation therefore only matters for messages the
//! server sends on its own initiative.
//!
//! Since formats are gated behind [features], a server may genuinely be unable
//! to speak a format a client asks for, which is why step 3 can fail.
//!
//! [features]: <https://docs.rs/musli-web/latest/musli_web/#features>
//! [`ws::Connect::connect`]: <https://docs.rs/musli-web/latest/musli_web/ws/struct.Connect.html#method.connect>
//! [`ws::Server`]: <https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html>

use core::fmt;
use core::num::NonZeroU16;
use core::sync::atomic::{AtomicU16, Ordering};

use musli::alloc::Global;
use musli::mode::{Binary, Text};
use musli::{Decode, Encode};

#[doc(inline)]
pub use musli_web_macros::define;

/// The serialization format used for message bodies.
///
/// The format is negotiated per connection, see the [negotiation protocol] for the
/// details of how. Message *headers* are never affected by this and always use
/// a fixed envelope, which is what makes negotiation possible in the first
/// place.
///
/// The variants are ordered from least to most capable. Each capability that is
/// dropped makes the encoding more compact:
///
/// | | `reorder` | `missing` | `unknown` | `self` |
/// |-|-|-|-|-|
/// | [`Packed`] | ✗ | ✗ | ✗ | ✗ |
/// | [`Storage`] | ✔ | ✔ | ✗ | ✗ |
/// | [`Wire`] | ✔ | ✔ | ✔ | ✗ |
/// | [`Descriptive`] | ✔ | ✔ | ✔ | ✔ |
/// | [`Json`] | ✔ | ✔ | ✔ | ✔ |
///
/// * `reorder` determines whether fields may be reordered in the model.
/// * `missing` determines whether decoding tolerates missing fields, which is
///   what allows new optional fields to be added.
/// * `unknown` determines whether decoding can skip fields it does not know
///   about. A format which can do this is *fully upgrade safe*, since an old
///   peer can talk to a new one.
/// * `self` determines whether the format is self-descriptive, so that the data
///   can be decoded without the model.
///
/// [`Packed`]: Format::Packed
/// [`Storage`]: Format::Storage
/// [`Wire`]: Format::Wire
/// [`Descriptive`]: Format::Descriptive
/// [`Json`]: Format::Json
///
/// # Examples
///
/// ```
/// use musli_web::api::Format;
///
/// assert_eq!(Format::default(), Format::Wire);
/// assert!(Format::Wire.is_upgrade_safe());
/// assert!(!Format::Storage.is_upgrade_safe());
/// assert!(Format::Json.is_human_readable());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Format {
    /// The [`musli::packed`] format.
    ///
    /// The most compact format, but it requires that both peers use exactly the
    /// same model. Suitable when client and server are deployed together.
    Packed,
    /// The [`musli::storage`] format.
    ///
    /// Tolerates missing fields, but cannot skip fields it does not know about.
    Storage,
    /// The [`musli::wire`] format.
    ///
    /// Fully upgrade safe, so peers built against different versions of the
    /// model can talk to each other. This is the default.
    Wire,
    /// The [`musli::descriptive`] format.
    ///
    /// Fully upgrade safe and self-descriptive, at the cost of a larger
    /// payload.
    Descriptive,
    /// The [`musli::json`] format.
    ///
    /// Human readable, which is useful when the traffic has to be inspected by
    /// hand. Encoded using the [`Text`] mode so that fields are keyed by name.
    ///
    /// [`Text`]: musli::mode::Text
    Json,
}

impl Format {
    /// The default format, which is [`Format::Wire`].
    ///
    /// This is used by a client which has not picked a format, and by a server
    /// for a connection which has not negotiated one.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert_eq!(Format::DEFAULT, Format::Wire);
    /// ```
    pub const DEFAULT: Self = Self::Wire;

    /// Every format in order of increasing capability.
    ///
    /// Note that this includes formats which the crate might not have been
    /// built with support for, see [`Format::is_supported`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert!(Format::ALL.contains(&Format::Json));
    /// ```
    pub const ALL: &'static [Format] = &[
        Format::Packed,
        Format::Storage,
        Format::Wire,
        Format::Descriptive,
        Format::Json,
    ];

    /// Get the stable identifier used for this format on the wire.
    ///
    /// Zero is never used, so it is available to indicate an absent format.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert_eq!(Format::Wire.to_u8(), 3);
    /// assert_eq!(Format::from_u8(3), Some(Format::Wire));
    /// ```
    #[inline]
    pub const fn to_u8(self) -> u8 {
        match self {
            Format::Packed => 1,
            Format::Storage => 2,
            Format::Wire => 3,
            Format::Descriptive => 4,
            Format::Json => 5,
        }
    }

    /// Construct a format from the stable identifier used on the wire.
    ///
    /// Returns `None` if the identifier is not known, which is how a peer
    /// built against an older version of this crate reports a format it has
    /// never heard of.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert_eq!(Format::from_u8(1), Some(Format::Packed));
    /// assert_eq!(Format::from_u8(0), None);
    /// assert_eq!(Format::from_u8(200), None);
    /// ```
    #[inline]
    pub const fn from_u8(id: u8) -> Option<Self> {
        match id {
            1 => Some(Format::Packed),
            2 => Some(Format::Storage),
            3 => Some(Format::Wire),
            4 => Some(Format::Descriptive),
            5 => Some(Format::Json),
            _ => None,
        }
    }

    /// The name of the format.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert_eq!(Format::Wire.name(), "wire");
    /// ```
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Format::Packed => "packed",
            Format::Storage => "storage",
            Format::Wire => "wire",
            Format::Descriptive => "descriptive",
            Format::Json => "json",
        }
    }

    /// Test if the format can skip over unknown fields, making it fully upgrade
    /// safe.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert!(Format::Wire.is_upgrade_safe());
    /// assert!(!Format::Packed.is_upgrade_safe());
    /// ```
    #[inline]
    pub const fn is_upgrade_safe(self) -> bool {
        matches!(self, Format::Wire | Format::Descriptive | Format::Json)
    }

    /// Test if the format is self-descriptive, so that data can be decoded
    /// without access to the model.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert!(Format::Descriptive.is_self_descriptive());
    /// assert!(!Format::Wire.is_self_descriptive());
    /// ```
    #[inline]
    pub const fn is_self_descriptive(self) -> bool {
        matches!(self, Format::Descriptive | Format::Json)
    }

    /// Test if the format produces output which is meant to be read by humans.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert!(Format::Json.is_human_readable());
    /// assert!(!Format::Wire.is_human_readable());
    /// ```
    #[inline]
    pub const fn is_human_readable(self) -> bool {
        matches!(self, Format::Json)
    }
}

impl Default for Format {
    /// Construct the default format, which is [`Format::DEFAULT`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Format;
    ///
    /// assert_eq!(Format::default(), Format::Wire);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Format {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How a socket frames the messages it carries.
///
/// This picks between the two shapes a message can have on the wire. It is
/// orthogonal to the [`Format`] which bodies are encoded with, except that
/// [`Mode::Text`] requires a human readable one, see the [text envelope].
///
/// [text envelope]: crate::api#the-text-envelope
///
/// # Examples
///
/// ```
/// use musli_web::api::Mode;
///
/// assert_eq!(Mode::default(), Mode::Binary);
/// assert!(Mode::Text.is_text());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Mode {
    /// Messages are binary frames whose envelope is encoded with
    /// [`musli::packed`].
    ///
    /// This is the compact representation and the default.
    Binary,
    /// Messages are text frames whose envelope is a block of
    /// `<key>: <value>` lines.
    ///
    /// The whole frame is human readable, which is what makes it possible to
    /// read the traffic in a browser inspector or a proxy. Since the body is
    /// part of that frame it has to be human readable too, so this requires a
    /// [`Format`] for which [`Format::is_human_readable`] holds.
    Text,
}

impl Mode {
    /// The default mode, which is [`Mode::Binary`].
    ///
    /// This is used by a client which has not picked a mode, and by a server
    /// for a connection which has not negotiated one.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Mode;
    ///
    /// assert_eq!(Mode::DEFAULT, Mode::Binary);
    /// ```
    pub const DEFAULT: Self = Self::Binary;

    /// Every mode a socket can run in.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Mode;
    ///
    /// assert!(Mode::ALL.contains(&Mode::Text));
    /// ```
    pub const ALL: &'static [Mode] = &[Mode::Binary, Mode::Text];

    /// The name of the mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Mode;
    ///
    /// assert_eq!(Mode::Text.name(), "text");
    /// ```
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Mode::Binary => "binary",
            Mode::Text => "text",
        }
    }

    /// Test if the mode uses text frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Mode;
    ///
    /// assert!(Mode::Text.is_text());
    /// assert!(!Mode::Binary.is_text());
    /// ```
    #[inline]
    pub const fn is_text(self) -> bool {
        matches!(self, Mode::Text)
    }

    /// Test if `format` can be used for message bodies in this mode.
    ///
    /// A text frame has to be valid UTF-8 in its entirety, so a mode which
    /// uses them can only carry bodies which are human readable.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::{Format, Mode};
    ///
    /// assert!(Mode::Binary.accepts(Format::Wire));
    /// assert!(Mode::Text.accepts(Format::Json));
    /// assert!(!Mode::Text.accepts(Format::Wire));
    /// ```
    #[inline]
    pub const fn accepts(self, format: Format) -> bool {
        match self {
            Mode::Binary => true,
            Mode::Text => format.is_human_readable(),
        }
    }

    /// A dense index for the mode, so that it can be kept in an atomic.
    ///
    /// This is not a wire representation, the frame type is what carries the
    /// mode between peers.
    #[inline]
    #[cfg_attr(not(feature = "client"), allow(dead_code))]
    pub(crate) const fn to_index(self) -> u8 {
        match self {
            Mode::Binary => 0,
            Mode::Text => 1,
        }
    }

    /// Recover a mode from [`Mode::to_index`].
    #[inline]
    #[cfg_attr(not(feature = "client"), allow(dead_code))]
    pub(crate) const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Mode::Binary),
            1 => Some(Mode::Text),
            _ => None,
        }
    }
}

impl Default for Mode {
    /// Construct the default mode, which is [`Mode::DEFAULT`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_web::api::Mode;
    ///
    /// assert_eq!(Mode::default(), Mode::Binary);
    /// ```
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Mode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Types which can be encoded in every mode that this crate supports.
///
/// This is a blanket trait covering [`Encode`] in both the [`Binary`] and
/// [`Text`] modes, which is what allows a message body to be encoded with any
/// [`Format`] including [`Format::Json`].
///
/// Deriving [`Encode`] implements every mode, so this is implemented
/// automatically unless the type has been restricted to a specific mode.
pub trait EncodeBody
where
    Self: Encode<Binary> + Encode<Text>,
{
}

impl<T> EncodeBody for T where T: ?Sized + Encode<Binary> + Encode<Text> {}

/// Types which can be decoded in every mode that this crate supports.
///
/// This is a blanket trait covering [`Decode`] in both the [`Binary`] and
/// [`Text`] modes, which is what allows a message body to be decoded with any
/// [`Format`] including [`Format::Json`].
///
/// Deriving [`Decode`] implements every mode, so this is implemented
/// automatically unless the type has been restricted to a specific mode.
pub trait DecodeBody<'de>
where
    Self: Decode<'de, Binary, Global> + Decode<'de, Text, Global>,
{
}

impl<'de, T> DecodeBody<'de> for T where T: Decode<'de, Binary, Global> + Decode<'de, Text, Global> {}

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
    #[cfg_attr(
        not(any(feature = "ws", feature = "client", feature = "web03")),
        allow(dead_code)
    )]
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

    /// A request from the client to use a particular [`Format`] for the
    /// remainder of the connection.
    ///
    /// See the [negotiation protocol] for how this is used.
    pub const NEGOTIATE: Self = unsafe { Self::new_unchecked((i16::MAX as u16) + 5) };

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
    type Type<'de>: DecodeBody<'de>;

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
    type Response<'de>: DecodeBody<'de>;

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
    type Event<'de>: Event<Broadcast = Self> + DecodeBody<'de>
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
    Self: EncodeBody,
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
    Self: EncodeBody,
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
///
/// This is the envelope of every message the server sends, see the [wire
/// format].
///
/// [wire format]: crate::api#wire-format
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct ResponseHeader {
    /// The serial request this is a response to.
    pub serial: u32,
    /// This is a broadcast over the specified type. If this is non-empty the
    /// serial is 0.
    pub broadcast: u16,
    /// If non-zero, the response contains an error of the given type.
    pub error: u16,
    /// The [`Format`] the body of this response is encoded with, as given by
    /// [`Format::to_u8`]. Zero if the response carries no body.
    pub format: u8,
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
///
/// This is the envelope of every message the client sends, see the [wire
/// format].
///
/// [wire format]: crate::api#wire-format
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct RequestHeader {
    /// The serial of the request.
    pub serial: u32,
    /// The kind of the request.
    pub id: u16,
    /// The [`Format`] the body of this request is encoded with, as given by
    /// [`Format::to_u8`]. Zero if the request carries no body.
    pub format: u8,
    /// The channel over which the request was received.
    pub channel: ChannelId,
}
