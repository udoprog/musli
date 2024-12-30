use core::fmt;

use crate::Context;

#[cfg(feature = "alloc")]
use rust_alloc::boxed::Box;
#[cfg(feature = "alloc")]
use rust_alloc::format;

#[derive(Debug)]
pub enum SerdeError {
    Captured,
    #[cfg(not(feature = "alloc"))]
    Custom,
    #[cfg(feature = "alloc")]
    Custom(Box<str>),
}

impl SerdeError {
    pub(super) fn report<C>(self, cx: &C) -> Option<C::Error>
    where
        C: ?Sized + Context,
    {
        match self {
            SerdeError::Captured => None,
            #[cfg(not(feature = "alloc"))]
            SerdeError::Custom => {
                Some(cx.message("Error in musli::serde (enable alloc for details)"))
            }
            #[cfg(feature = "alloc")]
            SerdeError::Custom(message) => Some(cx.message(message)),
        }
    }
}

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in musli::serde")
    }
}

impl serde::ser::Error for SerdeError {
    #[cfg(feature = "alloc")]
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError::Custom(format!("{}", msg).into())
    }

    #[cfg(not(feature = "alloc"))]
    fn custom<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError::Custom
    }
}

impl serde::de::Error for SerdeError {
    #[cfg(feature = "alloc")]
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError::Custom(format!("{}", msg).into())
    }

    #[cfg(not(feature = "alloc"))]
    fn custom<T>(_: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError::Custom
    }
}

impl serde::de::StdError for SerdeError {}
