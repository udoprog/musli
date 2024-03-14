use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::ops::Range;

use musli::{Allocator, Context};

use crate::fixed::{FixedString, FixedVec};

use super::access::{Access, Shared};
use super::rich_error::{RichError, Step};
use super::{Error, ErrorMarker};

/// A rich context which uses allocations and tracks the exact location of every
/// error.
///
/// * This only stores the latest error raised.
/// * The `P` param indicates the maximum number of path steps recorded. If
///   another step is added it will simply be ignored and an incomplete
///   indicator is used instead.
/// * The `S` parameter indicates the maximum size in bytes (UTF-8) of a stored
///   map key.
pub struct NoStdContext<const P: usize, const S: usize, A, E> {
    access: Access,
    mark: Cell<usize>,
    alloc: A,
    error: UnsafeCell<Option<(Range<usize>, E)>>,
    path: UnsafeCell<FixedVec<Step<FixedString<S>>, P>>,
    path_cap: Cell<usize>,
    include_type: bool,
}

impl<A, E> NoStdContext<16, 32, A, E> {
    /// Construct a new context which uses allocations to a fixed number of
    /// diagnostics.
    ///
    /// This uses the default values of:
    /// * 16 path elements stored.
    /// * A maximum map key of 32 bytes (UTF-8).
    pub fn new(alloc: A) -> Self {
        Self::new_with(alloc)
    }
}

impl<const P: usize, const S: usize, A, E> NoStdContext<P, S, A, E> {
    /// Construct a new context which uses allocations to a fixed but
    /// configurable number of diagnostics.
    pub fn new_with(alloc: A) -> Self {
        Self {
            access: Access::new(),
            mark: Cell::new(0),
            alloc,
            error: UnsafeCell::new(None),
            path: UnsafeCell::new(FixedVec::new()),
            path_cap: Cell::new(0),
            include_type: false,
        }
    }

    /// Configure the context to visualize type information, and not just
    /// variant and fields.
    pub fn include_type(&mut self) -> &mut Self {
        self.include_type = true;
        self
    }

    /// Iterate over all collected errors.
    pub fn errors(&self) -> Errors<'_, S, E> {
        let access = self.access.shared();

        Errors {
            path: unsafe { &*self.path.get() },
            error: unsafe { (*self.error.get()).as_ref() },
            path_cap: self.path_cap.get(),
            _access: access,
        }
    }

    /// Push an error into the collection.
    fn push_error(&self, range: Range<usize>, error: E) {
        // SAFETY: We've restricted access to the context, so this is safe.
        unsafe {
            self.error.get().replace(Some((range, error)));
        }
    }

    /// Push a path.
    fn push_path(&self, step: Step<FixedString<S>>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        let path = unsafe { &mut (*self.path.get()) };

        if path.try_push(step).is_err() {
            self.path_cap.set(self.path_cap.get() + 1);
        }
    }

    /// Pop the last path.
    fn pop_path(&self) {
        let cap = self.path_cap.get();

        if cap > 0 {
            self.path_cap.set(cap - 1);
            return;
        }

        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        unsafe {
            (*self.path.get()).pop();
        }
    }
}

impl<const V: usize, const S: usize, A, E> Context for NoStdContext<V, S, A, E>
where
    A: Allocator,
    E: Error,
{
    type Input = E;
    type Error = ErrorMarker;
    type Mark = usize;
    type Buf<'this> = A::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
    }

    #[inline(always)]
    fn report<T>(&self, error: T) -> Self::Error
    where
        E: From<T>,
    {
        self.push_error(self.mark.get()..self.mark.get(), E::from(error));
        ErrorMarker
    }

    #[inline]
    fn marked_report<T>(&self, mark: Self::Mark, message: T) -> Self::Error
    where
        E: From<T>,
    {
        self.push_error(mark..self.mark.get(), E::from(message));
        ErrorMarker
    }

    #[inline(always)]
    fn custom<T>(&self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.push_error(self.mark.get()..self.mark.get(), E::custom(message));
        ErrorMarker
    }

    #[inline(always)]
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.push_error(self.mark.get()..self.mark.get(), E::message(message));
        ErrorMarker
    }

    #[inline]
    fn marked_message<T>(&self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.push_error(mark..self.mark.get(), E::message(message));
        ErrorMarker
    }

    #[inline]
    fn mark(&self) -> Self::Mark {
        self.mark.get()
    }

    #[inline]
    fn advance(&self, n: usize) {
        self.mark.set(self.mark.get().wrapping_add(n));
    }

    #[inline]
    fn enter_named_field<T>(&self, name: &'static str, _: T)
    where
        T: fmt::Display,
    {
        self.push_path(Step::Named(name));
    }

    #[inline]
    fn enter_unnamed_field<T>(&self, index: u32, _: T)
    where
        T: fmt::Display,
    {
        self.push_path(Step::Unnamed(index));
    }

    #[inline]
    fn leave_field(&self) {
        self.pop_path();
    }

    #[inline]
    fn enter_struct(&self, name: &'static str) {
        if self.include_type {
            self.push_path(Step::Struct(name));
        }
    }

    #[inline]
    fn leave_struct(&self) {
        if self.include_type {
            self.pop_path();
        }
    }

    #[inline]
    fn enter_enum(&self, name: &'static str) {
        if self.include_type {
            self.push_path(Step::Enum(name));
        }
    }

    #[inline]
    fn leave_enum(&self) {
        if self.include_type {
            self.pop_path();
        }
    }

    #[inline]
    fn enter_variant<T>(&self, name: &'static str, _: T) {
        self.push_path(Step::Variant(name));
    }

    #[inline]
    fn leave_variant(&self) {
        self.pop_path();
    }

    #[inline]
    fn enter_sequence_index(&self, index: usize) {
        self.push_path(Step::Index(index));
    }

    #[inline]
    fn leave_sequence_index(&self) {
        self.pop_path();
    }

    #[inline]
    fn enter_map_key<T>(&self, field: T)
    where
        T: fmt::Display,
    {
        use core::fmt::Write;
        let mut string = FixedString::new();
        let _ = write!(string, "{}", field);
        self.push_path(Step::Key(string));
    }

    #[inline]
    fn leave_map_key(&self) {
        self.pop_path();
    }
}

/// An iterator over available errors.
pub struct Errors<'a, const S: usize, E> {
    path: &'a [Step<FixedString<S>>],
    error: Option<&'a (Range<usize>, E)>,
    path_cap: usize,
    _access: Shared<'a>,
}

impl<'a, const S: usize, E: 'a> Iterator for Errors<'a, S, E> {
    type Item = RichError<'a, FixedString<S>, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (range, error) = self.error.take()?;

        Some(RichError::new(
            self.path,
            self.path_cap,
            range.clone(),
            error,
        ))
    }
}
