//! [`Context`] implementations.
//!
//! [`Context`]: crate::Context

mod access;
use self::access::{Access, Shared};

mod trace;
#[doc(inline)]
pub use self::trace::{Error, Errors, NoTrace, Report, Trace, TraceImpl, TraceMode};

mod capture;
#[doc(inline)]
pub use self::capture::{Capture, Emit, ErrorMode, Ignore};

mod error_marker;
#[doc(inline)]
pub use self::error_marker::ErrorMarker;

mod default_context;
#[doc(inline)]
pub use self::default_context::{DEFAULT_NESTING_LIMIT, DefaultContext};

mod context_error;
#[doc(inline)]
pub use self::context_error::ContextError;

use crate::Allocator;
#[cfg(feature = "alloc")]
use crate::alloc::Global;

/// Construct a new default context using the [`Global`] allocator.
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
#[inline]
pub fn new() -> DefaultContext<Global, NoTrace, Ignore> {
    DefaultContext::new()
}

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
#[inline]
pub fn new_in<A>(alloc: A) -> DefaultContext<A, NoTrace, Ignore>
where
    A: Allocator,
{
    DefaultContext::new_in(alloc)
}

/// Decode the contents of a nested container through `f`, one level of
/// nesting deeper.
///
/// The level is left whether or not `f` succeeds, so a context which is
/// reused after an error does not keep counting the levels which were active
/// when the error occurred.
#[inline(always)]
pub(crate) fn nested<C, O>(cx: C, f: impl FnOnce() -> Result<O, C::Error>) -> Result<O, C::Error>
where
    C: crate::Context,
{
    cx.enter_nesting()?;
    let result = f();
    cx.leave_nesting();
    result
}
