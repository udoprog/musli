//! [`Context`] implementations.
//!
//! [`Context`]: crate::Context

mod access;
use self::access::{Access, Shared};

mod error_marker;
#[doc(inline)]
pub use self::error_marker::ErrorMarker;

mod default_context;
#[doc(inline)]
pub use self::default_context::{DefaultContext, Error};

mod context_error;
#[doc(inline)]
pub use self::context_error::ContextError;

mod same;
#[doc(inline)]
pub use self::same::Same;

mod capture;
#[doc(inline)]
pub use self::capture::Capture;

mod ignore;
#[doc(inline)]
pub use self::ignore::Ignore;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::Allocator;

/// Construct a new default context using the provided allocator.
///
/// # Examples
///
/// The `default` macro provides access to the default allocator. This is how it
/// can be used with this method:
///
/// ```
/// use musli::context;
///
/// musli::alloc::default(|alloc| {
///     let cx = context::with_alloc(alloc);
///     let encoding = musli::json::Encoding::new();
///     let string = encoding.to_string_with(&cx, &42)?;
///     assert_eq!(string, "42");
///     Ok(())
/// })?;
/// # Ok::<_, musli::context::ErrorMarker>(())
/// ```
///
/// We can also very conveniently set up an allocator which uses an existing
/// buffer:
///
/// ```
/// use musli::{alloc, context};
///
/// let mut buf = alloc::ArrayBuffer::new();
/// let alloc = alloc::Slice::new(&mut buf);
/// let cx = context::with_alloc(&alloc);
///
/// let encoding = musli::json::Encoding::new();
/// let string = encoding.to_string_with(&cx, &42)?;
/// assert_eq!(string, "42");
/// # Ok::<_, musli::context::ErrorMarker>(())
/// ```
///
pub fn with_alloc<A, M>(alloc: A) -> DefaultContext<A, M>
where
    A: Clone + Allocator,
{
    DefaultContext::with_alloc(alloc)
}

/// Construct a new default context using the static [`System`] allocator.
///
/// [`System`]: crate::alloc::System
///
/// # Examples
///
/// ```
/// use musli::context;
///
/// musli::alloc::default(|alloc| {
///     let cx = context::new();
///     let encoding = musli::json::Encoding::new();
///     let string = encoding.to_string_with(&cx, &42)?;
///     assert_eq!(string, "42");
///     Ok(())
/// })?;
/// # Ok::<_, musli::context::ErrorMarker>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub fn new<M>() -> DefaultContext<System, M> {
    DefaultContext::new()
}
