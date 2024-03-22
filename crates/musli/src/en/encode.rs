use crate::en::Encoder;
use crate::mode::DefaultMode;
use crate::Context;

pub use musli_macros::Encode;

/// Trait governing how types are encoded.
///
/// This is typically implemented automatically using the [`Encode` derive].
///
/// [`Encode` derive]: https://docs.rs/musli/latest/musli/derives/
///
/// # Examples
///
/// ```
/// use musli::Encode;
///
/// #[derive(Encode)]
/// struct MyType {
///     data: [u8; 128],
/// }
/// ````
///
/// Implementing manually:
///
/// ```
/// use musli::{Context, Encode, Encoder};
///
/// struct MyType {
///     data: [u8; 128],
/// }
///
/// impl<M> Encode<M> for MyType {
///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
///     where
///         C: ?Sized + Context<Mode = M>,
///         E: Encoder<C>,
///     {
///         encoder.encode_array(cx, &self.data)
///     }
/// }
/// ```
pub trait Encode<M = DefaultMode> {
    /// Encode the given output.
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        E: Encoder<C>;
}

/// Trait governing how types are encoded specifically for tracing.
///
/// This is used for types where some extra bounds might be necessary to trace a
/// container such as a [`HashMap<K, V>`] where `K` would have to implement
/// [`fmt::Display`].
///
/// [`HashMap<K, V>`]: std::collections::HashMap
/// [`fmt::Display`]: std::fmt::Display
pub trait TraceEncode<M = DefaultMode> {
    /// Encode the given output.
    fn trace_encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        E: Encoder<C>;
}

impl<T, M> Encode<M> for &T
where
    T: ?Sized + Encode<M>,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        E: Encoder<C>,
    {
        (**self).encode(cx, encoder)
    }
}

impl<T, M> Encode<M> for &mut T
where
    T: ?Sized + Encode<M>,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        E: Encoder<C>,
    {
        (**self).encode(cx, encoder)
    }
}
