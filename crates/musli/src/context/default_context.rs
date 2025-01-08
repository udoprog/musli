use core::error::Error;
use core::fmt;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::{Allocator, Context};

use super::{
    Capture, CaptureError, ContextError, Errors, NoCapture, NoTrace, Report, SameError, Trace,
    TraceConfig, TraceImpl,
};

/// The default context which uses an allocator to track the location of errors.
///
/// This is typically constructed using [`new`] and by default uses the
/// [`System`] allocator to allocate memory. To customized the allocator to use
/// [`new_in`] can be used during construction.
///
/// The default constructor is only available when the `alloc` feature is
/// enabled, and will use the [`System`] allocator.
///
/// [`new`]: super::new
/// [`new_in`]: super::new_in
pub struct DefaultContext<A, B, C>
where
    A: Allocator,
    B: TraceConfig,
{
    alloc: A,
    trace: B::Impl<A>,
    capture: C,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl DefaultContext<System, NoTrace, NoCapture> {
    /// Construct the default context which uses the [`System`] allocator for
    /// memory.
    #[inline]
    pub fn new() -> Self {
        Self::new_in(System::new())
    }
}

#[cfg(feature = "alloc")]
impl Default for DefaultContext<System, NoTrace, NoCapture> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A> DefaultContext<A, NoTrace, NoCapture>
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
            capture: NoCapture,
        }
    }

    /// Unwrap the error marker or panic if there is no error.
    #[inline]
    pub fn is_error(self) -> bool {
        self.trace.is_error()
    }
}

impl<A, C> DefaultContext<A, Trace, C>
where
    A: Allocator,
{
    /// If tracing is enabled through [`DefaultContext::with_trace`], this
    /// configured the context to visualize type information, and not just
    /// variant and fields.
    #[inline]
    pub fn with_type(mut self) -> Self {
        self.trace.include_type();
        self
    }

    /// Generate a line-separated report of all reported errors.
    ///
    /// This can be useful if you want a quick human-readable overview of
    /// errors. The line separator used will be platform dependent.
    #[inline]
    pub fn report(&self) -> Report<'_, A> {
        self.trace.report()
    }

    /// Iterate over all reported errors.
    #[inline]
    pub fn errors(&self) -> Errors<'_, A> {
        self.trace.errors()
    }
}

impl<A, B, C> DefaultContext<A, B, C>
where
    A: Allocator,
    B: TraceConfig,
    C: Capture<A>,
{
    /// Enable tracing through the current allocator `A`.
    ///
    /// Note that this makes diagnostics methods such as [`report`] and
    /// [`errors`] available on the type.
    ///
    /// Tracing requires the configured allocator to work, if for example the
    /// [`Disabled`] allocator was in use, no diagnostics would be collected.
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
    /// use musli::alloc::System;
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
    pub fn with_capture<E>(self) -> DefaultContext<A, B, CaptureError<E, A>>
    where
        E: ContextError<A>,
    {
        DefaultContext {
            alloc: self.alloc,
            trace: self.trace,
            capture: CaptureError::new(),
        }
    }

    /// Forward the specified error type.
    ///
    /// This causes the method receiving the context to return the specified
    /// error type directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::alloc::System;
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
    /// let cx = context::new().with_same();
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
    pub fn with_same<E>(self) -> DefaultContext<A, B, SameError<E, A>>
    where
        E: ContextError<A>,
    {
        DefaultContext {
            alloc: self.alloc,
            trace: self.trace,
            capture: SameError::new(),
        }
    }
}

impl<A, B, E> DefaultContext<A, B, CaptureError<E, A>>
where
    A: Allocator,
    B: TraceConfig,
    E: ContextError<A>,
{
    /// Unwrap the error marker or panic if there is no error.
    #[inline]
    pub fn unwrap(&self) -> E {
        self.capture.unwrap()
    }

    /// Coerce a captured error into a result.
    #[inline]
    pub fn result(&self) -> Result<(), E> {
        self.capture.result()
    }
}

impl<A, B, C> Context for &DefaultContext<A, B, C>
where
    A: Allocator,
    B: TraceConfig,
    C: Capture<A>,
{
    type Error = C::Error;
    type Mark = <<B as TraceConfig>::Impl<A> as TraceImpl<A>>::Mark;
    type Allocator = A;

    #[inline]
    fn clear(self) {
        self.trace.clear();
        self.capture.clear();
    }

    #[inline]
    fn alloc(self) -> Self::Allocator {
        self.alloc
    }

    #[inline]
    fn custom<T>(self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        self.trace.custom(self.alloc, &message);
        self.capture.custom(self.alloc, message)
    }

    #[inline]
    fn message<T>(self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.trace.message(self.alloc, &message);
        self.capture.message(self.alloc, message)
    }

    #[inline]
    fn marked_message<T>(self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.trace.marked_message(self.alloc, mark, &message);
        self.capture.message(self.alloc, message)
    }

    #[inline]
    fn marked_custom<T>(self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + Error,
    {
        self.trace.marked_custom(self.alloc, mark, &message);
        self.capture.custom(self.alloc, message)
    }

    #[inline]
    fn mark(self) -> Self::Mark {
        self.trace.mark()
    }

    #[inline]
    fn advance(self, n: usize) {
        self.trace.advance(n);
    }

    #[inline]
    fn enter_named_field<T>(self, name: &'static str, field: T)
    where
        T: fmt::Display,
    {
        self.trace.enter_named_field(name, &field);
    }

    #[inline]
    fn enter_unnamed_field<T>(self, index: u32, name: T)
    where
        T: fmt::Display,
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
    fn enter_variant<T>(self, name: &'static str, tag: T)
    where
        T: fmt::Display,
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
    fn enter_map_key<T>(self, field: T)
    where
        T: fmt::Display,
    {
        self.trace.enter_map_key(self.alloc, &field);
    }

    #[inline]
    fn leave_map_key(self) {
        self.trace.leave_map_key();
    }
}
