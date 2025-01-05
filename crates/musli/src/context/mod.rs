//! [`Context`] implementations.
//!
//! [`Context`]: crate::Context

mod access;
use self::access::{Access, Shared};

mod trace;
#[doc(inline)]
pub use self::trace::{Error, Errors, NoTrace, Report, Trace, WithTrace};

mod error_marker;
#[doc(inline)]
pub use self::error_marker::ErrorMarker;

mod default_context;
#[doc(inline)]
pub use self::default_context::DefaultContext;

mod context_error;
#[doc(inline)]
pub use self::context_error::ContextError;

mod same;
#[doc(inline)]
pub use self::same::Same;

mod capture;
#[doc(inline)]
pub use self::capture::Capture;

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
///     let cx = context::new_in(alloc);
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
/// let cx = context::new_in(&alloc);
///
/// let encoding = musli::json::Encoding::new();
/// let string = encoding.to_string_with(&cx, &42)?;
/// assert_eq!(string, "42");
/// # Ok::<_, musli::context::ErrorMarker>(())
/// ```
///
pub fn new_in<A>(alloc: A) -> DefaultContext<A, NoTrace>
where
    A: Clone + Allocator,
{
    DefaultContext::new_in(alloc)
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
pub fn new() -> DefaultContext<System, NoTrace> {
    DefaultContext::new()
}
