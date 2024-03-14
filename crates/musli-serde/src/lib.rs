#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod deserializer;
mod error;
mod serializer;

use core::cell::RefCell;
use core::fmt;

use musli::{Context, Decoder, Encoder, Mode};
use serde::{Deserialize, Serialize};

use self::deserializer::Deserializer;
use self::serializer::Serializer;

struct SerdeContext<'a, C>
where
    C: Context,
{
    error: RefCell<Option<C::Error>>,
    inner: &'a C,
}

impl<'a, C> Context for SerdeContext<'a, C>
where
    C: Context,
{
    type Mode = C::Mode;
    type Input = C::Input;
    type Error = error::SerdeError;
    type Mark = C::Mark;

    type Buf<'this> = C::Buf<'this>
    where
        Self: 'this;

    #[inline]
    fn mark(&self) -> Self::Mark {
        self.inner.mark()
    }

    #[inline]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.inner.alloc()
    }

    #[inline]
    fn report<T>(&self, error: T) -> Self::Error
    where
        Self::Input: From<T>,
    {
        *self.error.borrow_mut() = Some(self.inner.report(error));
        error::SerdeError::Captured
    }

    #[inline]
    fn custom<T>(&self, error: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        *self.error.borrow_mut() = Some(self.inner.custom(error));
        error::SerdeError::Captured
    }

    #[inline]
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        *self.error.borrow_mut() = Some(self.inner.message(message));
        error::SerdeError::Captured
    }
}

/// Encode the given serde value `T` to the given [Encoder] using the serde
/// compatibility layer.
pub fn encode<M, C, E, T>(value: &T, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
where
    M: Mode,
    C: Context<Input = E::Error>,
    E: Encoder,
    T: Serialize,
{
    let cx = SerdeContext {
        error: RefCell::new(None),
        inner: cx,
    };

    let serializer = Serializer::<_, _, M>::new(&cx, encoder);

    let error = match value.serialize(serializer) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };

    if let error::SerdeError::Custom(message) = error {
        return Err(cx.inner.message(message));
    }

    let Some(error) = cx.error.borrow_mut().take() else {
        return Err(cx.inner.message("error during encoding (no information)"));
    };

    Err(error)
}

/// Decode the given serde value `T` from the given [Decoder] using the serde
/// compatibility layer.
pub fn decode<'de, M, C, D, T>(cx: &C, decoder: D) -> Result<T, C::Error>
where
    M: Mode,
    C: Context<Input = D::Error>,
    D: Decoder<'de>,
    T: Deserialize<'de>,
{
    let cx = SerdeContext {
        error: RefCell::new(None),
        inner: cx,
    };

    let deserializer = Deserializer::<_, _, M>::new(&cx, decoder);

    let error = match T::deserialize(deserializer) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };

    if let error::SerdeError::Custom(message) = error {
        return Err(cx.inner.message(message));
    }

    let Some(error) = cx.error.borrow_mut().take() else {
        return Err(cx.inner.message("error during encoding (no information)"));
    };

    Err(error)
}
