use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use musli::{Allocator, Context};

use super::access::{self, Access};
use super::rich_error::{RichError, Step};
use super::{Error, ErrorMarker};

/// A rich context which uses allocations and tracks the exact location of every
/// error.
pub struct AllocContext<A, M, E> {
    access: Access,
    mark: Cell<usize>,
    alloc: A,
    errors: UnsafeCell<Vec<(Vec<Step<String>>, Range<usize>, E)>>,
    path: UnsafeCell<Vec<Step<String>>>,
    include_type: bool,
    _marker: PhantomData<M>,
}

impl<A, M, E> AllocContext<A, M, E> {
    /// Construct a new context which uses allocations to store arbitrary
    /// amounts of diagnostics about decoding.
    ///
    /// Or at least until we run out of memory.
    pub fn new(alloc: A) -> Self {
        Self {
            access: Access::new(),
            mark: Cell::new(0),
            alloc,
            errors: UnsafeCell::new(Vec::new()),
            path: UnsafeCell::new(Vec::new()),
            include_type: false,
            _marker: PhantomData,
        }
    }

    /// Configure the context to visualize type information, and not just
    /// variant and fields.
    pub fn include_type(&mut self) -> &mut Self {
        self.include_type = true;
        self
    }

    /// Iterate over all collected errors.
    pub fn errors(&self) -> Errors<'_, E> {
        let access = self.access.shared();

        // SAFETY: We've checked above that we have shared access.
        Errors {
            errors: unsafe { &*self.errors.get() },
            index: 0,
            _access: access,
        }
    }
}

impl<A, M, E> AllocContext<A, M, E>
where
    A: Allocator,
    E: Error,
{
    fn push_error(&self, range: Range<usize>, message: E) {
        let _access = self.access.exclusive();

        // SAFETY: We've restricted access to the context, so this is safe.
        let path = unsafe { (*self.path.get()).clone() };
        let errors = unsafe { &mut (*self.errors.get()) };

        errors.push((path, range, message));
    }

    fn push_path(&self, step: Step<String>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        let path = unsafe { &mut (*self.path.get()) };

        path.push(step);
    }

    fn pop_path(&self) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        let path = unsafe { &mut (*self.path.get()) };

        path.pop();
    }
}

impl<A, M, E> Context for AllocContext<A, M, E>
where
    A: Allocator,
    E: Error,
{
    type Mode = M;
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
        self.push_path(Step::Key(field.to_string()));
    }

    #[inline]
    fn leave_map_key(&self) {
        self.pop_path();
    }
}

/// An iterator over collected errors.
pub struct Errors<'a, E> {
    errors: &'a [(Vec<Step<String>>, Range<usize>, E)],
    index: usize,
    // NB: Drop order is significant, drop the shared access last.
    _access: access::Shared<'a>,
}

impl<'a, E> Iterator for Errors<'a, E> {
    type Item = RichError<'a, String, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let (path, range, error) = self.errors.get(self.index)?;
        self.index += 1;
        Some(RichError::new(path, 0, range.clone(), error))
    }
}
