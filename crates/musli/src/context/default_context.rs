use core::error::Error;
use core::fmt;

#[cfg(not(target_has_atomic = "ptr"))]
use core::cell::Cell;
#[cfg(target_has_atomic = "ptr")]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "alloc")]
use crate::alloc::Global;
use crate::{Allocator, Context};

use super::{
    Capture, ContextError, Emit, ErrorMode, Errors, Ignore, NoTrace, Report, Trace, TraceImpl,
    TraceMode,
};

/// The default context which uses an allocator to track the location of errors.
///
/// This is typically constructed using [`new`] and by default uses the
/// [`Global`] allocator to allocate memory. To customized the allocator to use
/// [`new_in`] can be used during construction.
///
/// The default constructor is only available when the `alloc` feature is
/// enabled, and will use the [`Global`] allocator.
///
/// # Nesting limit
///
/// Decoding nested containers recurses, so the context limits how deeply input
/// may be nested to avoid overflowing the stack on untrusted input. By default
/// at most [`DEFAULT_NESTING_LIMIT`] (128) levels are accepted, where every
/// sequence, map, pack, variant and present optional value counts as one
/// level. Deeper input is rejected with an error.
///
/// The limit can be changed with [`with_nesting_limit`] or removed with
/// [`without_nesting_limit`].
///
/// [`new`]: super::new
/// [`new_in`]: super::new_in
/// [`with_nesting_limit`]: DefaultContext::with_nesting_limit
/// [`without_nesting_limit`]: DefaultContext::without_nesting_limit
pub struct DefaultContext<A, T, C>
where
    A: Allocator,
    T: TraceMode,
{
    alloc: A,
    trace: T::Impl<A>,
    capture: C,
    depth: Depth,
    nesting_limit: Option<usize>,
}

/// The default number of nested levels accepted when decoding through a
/// [`DefaultContext`].
///
/// This matches the default recursion limit in `serde_json`.
pub const DEFAULT_NESTING_LIMIT: usize = 128;

/// The current nesting depth.
///
/// The depth is only ever accessed through relaxed loads and stores, which
/// keeps [`DefaultContext`] [`Sync`] where pointer-sized atomics are available.
/// A context shared between threads which decode concurrently would see a
/// confused depth, but decoding through a shared context is not meaningful in
/// the first place.
struct Depth {
    #[cfg(target_has_atomic = "ptr")]
    value: AtomicUsize,
    #[cfg(not(target_has_atomic = "ptr"))]
    value: Cell<usize>,
}

impl Depth {
    #[inline]
    const fn new() -> Self {
        Self {
            #[cfg(target_has_atomic = "ptr")]
            value: AtomicUsize::new(0),
            #[cfg(not(target_has_atomic = "ptr"))]
            value: Cell::new(0),
        }
    }

    #[inline]
    fn get(&self) -> usize {
        #[cfg(target_has_atomic = "ptr")]
        {
            self.value.load(Ordering::Relaxed)
        }

        #[cfg(not(target_has_atomic = "ptr"))]
        {
            self.value.get()
        }
    }

    #[inline]
    fn set(&self, value: usize) {
        #[cfg(target_has_atomic = "ptr")]
        {
            self.value.store(value, Ordering::Relaxed);
        }

        #[cfg(not(target_has_atomic = "ptr"))]
        {
            self.value.set(value);
        }
    }
}

#[cfg(feature = "alloc")]
impl DefaultContext<Global, NoTrace, Ignore> {
    /// Construct the default context which uses the [`Global`] allocator for
    /// memory.
    #[inline]
    pub(super) fn new() -> Self {
        Self::new_in(Global::new())
    }
}

impl<A> DefaultContext<A, NoTrace, Ignore>
where
    A: Allocator,
{
    /// Construct a new context which uses allocations to a fixed but
    /// configurable number of diagnostics.
    #[inline]
    pub(super) fn new_in(alloc: A) -> Self {
        let trace = NoTrace::new_in(alloc);
        Self {
            alloc,
            trace,
            capture: Ignore,
            depth: Depth::new(),
            nesting_limit: Some(DEFAULT_NESTING_LIMIT),
        }
    }
}

#[cfg(feature = "alloc")]
impl Default for DefaultContext<Global, NoTrace, Ignore> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A, T, C> DefaultContext<A, T, C>
where
    A: Allocator,
    T: TraceMode,
    C: ErrorMode<A>,
{
    /// Enable tracing through the current allocator `A`.
    ///
    /// Note that this makes diagnostics methods such as [`report`] and
    /// [`errors`] available on the type.
    ///
    /// Tracing requires the configured allocator to work, if for example the
    /// [`Disabled`] allocator was in use, no diagnostics would be collected.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context;
    ///
    /// let cx = context::new().with_trace();
    /// // Use cx for encoding/decoding to get detailed error information
    /// ```
    ///
    /// [`report`]: DefaultContext::report
    /// [`errors`]: DefaultContext::errors
    /// [`Disabled`]: crate::alloc::Disabled
    #[inline]
    pub fn with_trace(self) -> DefaultContext<A, Trace, C> {
        let trace = Trace::new_in(self.alloc);

        DefaultContext {
            alloc: self.alloc,
            trace,
            capture: self.capture,
            depth: Depth::new(),
            nesting_limit: self.nesting_limit,
        }
    }

    /// Capture the specified error type.
    ///
    /// This gives access to the last captured error through
    /// [`DefaultContext::unwrap`] and [`DefaultContext::result`].
    ///
    /// Capturing instead of forwarding the error might be beneficial if the
    /// error type is large.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::alloc::Global;
    /// use musli::context;
    /// use musli::json::{Encoding, Error};
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let cx = context::new().with_capture::<Error>();
    ///
    /// let mut data = Vec::new();
    ///
    /// ENCODING.encode_with(&cx, &mut data, &Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 61,
    /// })?;
    ///
    /// assert!(cx.result().is_ok());
    ///
    /// let _: Result<Person, _> = ENCODING.from_slice_with(&cx, &data[..data.len() - 2]);
    /// assert!(cx.result().is_err());
    /// Ok::<_, musli::context::ErrorMarker>(())
    /// ```
    #[inline]
    pub fn with_capture<E>(self) -> DefaultContext<A, T, Capture<E>>
    where
        E: ContextError<A>,
    {
        DefaultContext {
            alloc: self.alloc,
            trace: self.trace,
            capture: Capture::new(),
            depth: Depth::new(),
            nesting_limit: self.nesting_limit,
        }
    }

    /// Emit the specified error type `E`.
    ///
    /// This causes the method receiving the context to return the specified
    /// error type directly instead through [`Context::Error`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::alloc::Global;
    /// use musli::context;
    /// use musli::json::{Encoding, Error};
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let cx = context::new().with_error();
    ///
    /// let mut data = Vec::new();
    ///
    /// ENCODING.encode_with(&cx, &mut data, &Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 61,
    /// })?;
    ///
    /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
    /// assert_eq!(person.name, "Aristotle");
    /// assert_eq!(person.age, 61);
    /// Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn with_error<E>(self) -> DefaultContext<A, T, Emit<E>>
    where
        E: ContextError<A>,
    {
        DefaultContext {
            alloc: self.alloc,
            trace: self.trace,
            capture: Emit::new(),
            depth: Depth::new(),
            nesting_limit: self.nesting_limit,
        }
    }

    /// Limit how many levels of nested containers are accepted while
    /// decoding.
    ///
    /// Every sequence, map, pack, variant and present optional value counts as
    /// one level. Input nested deeper than `limit` levels is rejected with an
    /// error. The default limit is [`DEFAULT_NESTING_LIMIT`] (128).
    ///
    /// Since decoding nested containers recurses, a large limit allows
    /// deeply nested input to use a correspondingly large amount of stack.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context;
    /// use musli::json::Encoding;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// let input = "[[[[1]]]]";
    ///
    /// let cx = context::new().with_nesting_limit(3);
    /// assert!(ENCODING.from_str_with::<_, Vec<Vec<Vec<Vec<u32>>>>>(&cx, input).is_err());
    ///
    /// let cx = context::new().with_nesting_limit(4);
    /// let value: Vec<Vec<Vec<Vec<u32>>>> = ENCODING.from_str_with(&cx, input)?;
    /// assert_eq!(value, [[[[1]]]]);
    /// # Ok::<_, musli::context::ErrorMarker>(())
    /// ```
    #[inline]
    pub fn with_nesting_limit(mut self, limit: usize) -> Self {
        self.nesting_limit = Some(limit);
        self
    }

    /// Accept any level of nesting while decoding.
    ///
    /// This restores the behavior from before nesting was limited by default.
    /// Note that decoding nested containers recurses, so deeply nested input
    /// can then overflow the stack and abort the process. Only use this for
    /// trusted input.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context;
    /// use musli::json::Encoding;
    /// use musli::value::Value;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// let input = format!("{}{}", "[".repeat(200), "]".repeat(200));
    ///
    /// let cx = context::new();
    /// assert!(ENCODING.from_str_with::<_, Value<_>>(&cx, &input).is_err());
    ///
    /// let cx = context::new().without_nesting_limit();
    /// let value: Value<_> = ENCODING.from_str_with(&cx, &input)?;
    /// # Ok::<_, musli::context::ErrorMarker>(())
    /// ```
    #[inline]
    pub fn without_nesting_limit(mut self) -> Self {
        self.nesting_limit = None;
        self
    }
}

impl<A, C> DefaultContext<A, Trace, C>
where
    A: Allocator,
{
    /// If tracing is enabled through [`DefaultContext::with_trace`], this
    /// configured the context to visualize type information, and not just
    /// variant and fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context;
    ///
    /// let cx = context::new().with_trace().with_type();
    /// ```
    #[inline]
    pub fn with_type(mut self) -> Self {
        self.trace.include_type();
        self
    }

    /// Generate a line-separated report of all reported errors.
    ///
    /// This can be useful if you want a quick human-readable overview of
    /// errors. The line separator used will be platform dependent.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context::{self, ErrorMarker};
    /// use musli::value::Value;
    /// use musli::json::Encoding;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// let cx = context::new().with_trace();
    ///
    /// let ErrorMarker = ENCODING.from_str_with::<_, Value<_>>(&cx, "not json").unwrap_err();
    /// let report = cx.report();
    /// println!("{report}");
    /// ```
    #[inline]
    pub fn report(&self) -> Report<'_, A> {
        self.trace.report()
    }

    /// Iterate over all reported errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context::{self, ErrorMarker};
    /// use musli::value::Value;
    /// use musli::json::Encoding;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// let cx = context::new().with_trace();
    ///
    /// let ErrorMarker = ENCODING.from_str_with::<_, Value<_>>(&cx, "not json").unwrap_err();
    /// assert!(cx.errors().count() > 0);
    /// ```
    #[inline]
    pub fn errors(&self) -> Errors<'_, A> {
        self.trace.errors()
    }
}

impl<A, T, E> DefaultContext<A, T, Capture<E>>
where
    A: Allocator,
    T: TraceMode,
    E: ContextError<A>,
{
    /// Unwrap the error marker or panic if there is no error.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use musli::context;
    ///
    /// let cx = context::new().with_capture::<String>();
    /// // This will panic since no error has been captured
    /// let error = cx.unwrap();
    /// ```
    #[inline]
    pub fn unwrap(&self) -> E {
        self.capture.unwrap()
    }

    /// Coerce a captured error into a result.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context;
    ///
    /// let cx = context::new().with_capture::<String>();
    /// let result = cx.result();
    /// assert!(result.is_ok());
    /// ```
    #[inline]
    pub fn result(&self) -> Result<(), E> {
        self.capture.result()
    }
}

impl<A, T, C> Context for &DefaultContext<A, T, C>
where
    A: Allocator,
    T: TraceMode,
    C: ErrorMode<A>,
{
    type Error = C::Error;
    type Mark = <<T as TraceMode>::Impl<A> as TraceImpl<A>>::Mark;
    type Allocator = A;

    #[inline]
    fn clear(self) {
        self.trace.clear();
        self.capture.clear();
        self.depth.set(0);
    }

    #[inline]
    fn alloc(self) -> Self::Allocator {
        self.alloc
    }

    #[inline]
    fn custom<E>(self, message: E) -> Self::Error
    where
        E: 'static + Send + Sync + Error,
    {
        self.trace.custom(self.alloc, &message);
        self.capture.custom(self.alloc, message)
    }

    #[inline]
    fn message<M>(self, message: M) -> Self::Error
    where
        M: fmt::Display,
    {
        self.trace.message(self.alloc, &message);
        self.capture.message(self.alloc, message)
    }

    #[inline]
    fn message_at<M>(self, mark: &Self::Mark, message: M) -> Self::Error
    where
        M: fmt::Display,
    {
        self.trace.message_at(self.alloc, mark, &message);
        self.capture.message(self.alloc, message)
    }

    #[inline]
    fn custom_at<E>(self, mark: &Self::Mark, message: E) -> Self::Error
    where
        E: 'static + Send + Sync + Error,
    {
        self.trace.custom_at(self.alloc, mark, &message);
        self.capture.custom(self.alloc, message)
    }

    #[inline]
    fn mark(self) -> Self::Mark {
        self.trace.mark()
    }

    #[inline]
    fn restore(self, mark: &Self::Mark) {
        self.trace.restore(mark);
    }

    #[inline]
    fn advance(self, n: usize) {
        self.trace.advance(n);
    }

    #[inline]
    fn enter_nesting(self) -> Result<(), Self::Error> {
        let Some(limit) = self.nesting_limit else {
            return Ok(());
        };

        let depth = self.depth.get();

        if depth >= limit {
            return Err(self.message(NestingLimitExceeded { limit }));
        }

        self.depth.set(depth + 1);
        Ok(())
    }

    #[inline]
    fn leave_nesting(self) {
        if self.nesting_limit.is_some() {
            self.depth.set(self.depth.get().saturating_sub(1));
        }
    }

    #[inline]
    fn enter_named_field<F>(self, name: &'static str, field: F)
    where
        F: fmt::Display,
    {
        self.trace.enter_named_field(name, &field);
    }

    #[inline]
    fn enter_unnamed_field<F>(self, index: u32, name: F)
    where
        F: fmt::Display,
    {
        self.trace.enter_unnamed_field(index, &name);
    }

    #[inline]
    fn leave_field(self) {
        self.trace.leave_field();
    }

    #[inline]
    fn enter_struct(self, name: &'static str) {
        self.trace.enter_struct(name);
    }

    #[inline]
    fn leave_struct(self) {
        self.trace.leave_struct();
    }

    #[inline]
    fn enter_enum(self, name: &'static str) {
        self.trace.enter_enum(name);
    }

    #[inline]
    fn leave_enum(self) {
        self.trace.leave_enum();
    }

    #[inline]
    fn enter_variant<V>(self, name: &'static str, tag: V)
    where
        V: fmt::Display,
    {
        self.trace.enter_variant(name, &tag);
    }

    #[inline]
    fn leave_variant(self) {
        self.trace.leave_variant();
    }

    #[inline]
    fn enter_sequence_index(self, index: usize) {
        self.trace.enter_sequence_index(index);
    }

    #[inline]
    fn leave_sequence_index(self) {
        self.trace.leave_sequence_index();
    }

    #[inline]
    fn enter_map_key<F>(self, field: F)
    where
        F: fmt::Display,
    {
        self.trace.enter_map_key(self.alloc, &field);
    }

    #[inline]
    fn leave_map_key(self) {
        self.trace.leave_map_key();
    }
}

struct NestingLimitExceeded {
    limit: usize,
}

impl fmt::Display for NestingLimitExceeded {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Recursion limit exceeded, input may be nested at most {} levels deep",
            self.limit
        )
    }
}
