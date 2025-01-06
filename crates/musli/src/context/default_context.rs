use core::fmt;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::{Allocator, Context};

use super::{ErrorMarker, Errors, NoTrace, Report, Trace, TraceConfig, TraceImpl};

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
pub struct DefaultContext<A, B>
where
    A: Allocator,
    B: TraceConfig,
{
    alloc: A,
    trace: B::Impl<A>,
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl DefaultContext<System, NoTrace> {
    /// Construct the default context which uses the [`System`] allocator for
    /// memory.
    #[inline]
    pub fn new() -> Self {
        Self::new_in(System::new())
    }
}

#[cfg(feature = "alloc")]
impl Default for DefaultContext<System, NoTrace> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A> DefaultContext<A, NoTrace>
where
    A: Allocator,
{
    /// Construct a new context which uses allocations to a fixed but
    /// configurable number of diagnostics.
    #[inline]
    pub(super) fn new_in(alloc: A) -> Self {
        let trace = NoTrace::new_in(alloc);
        Self { alloc, trace }
    }

    /// Unwrap the error marker or panic if there is no error.
    #[inline]
    pub fn unwrap(self) -> ErrorMarker {
        self.trace.unwrap()
    }
}

impl<A> DefaultContext<A, Trace>
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

impl<A, B> DefaultContext<A, B>
where
    A: Allocator,
    B: TraceConfig,
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
    pub fn with_trace(self) -> DefaultContext<A, Trace> {
        let trace = Trace::new_in(self.alloc);

        DefaultContext {
            alloc: self.alloc,
            trace,
        }
    }
}

impl<A, B> Context for &DefaultContext<A, B>
where
    A: Allocator,
    B: TraceConfig,
{
    type Error = ErrorMarker;
    type Mark = <<B as TraceConfig>::Impl<A> as TraceImpl>::Mark;
    type Allocator = A;

    #[inline]
    fn clear(self) {
        self.trace.clear();
    }

    #[inline]
    fn alloc(self) -> Self::Allocator {
        self.alloc
    }

    #[inline]
    fn custom<T>(self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.trace.custom(self.alloc, &message);
        ErrorMarker
    }

    #[inline]
    fn message<T>(self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.trace.message(self.alloc, &message);
        ErrorMarker
    }

    #[inline]
    fn marked_message<T>(self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.trace.marked_message(self.alloc, mark, &message);
        ErrorMarker
    }

    #[inline]
    fn marked_custom<T>(self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.trace.marked_custom(self.alloc, mark, &message);
        ErrorMarker
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
