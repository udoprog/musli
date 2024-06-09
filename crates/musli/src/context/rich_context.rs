#![allow(clippy::type_complexity)]

use core::cell::{Cell, UnsafeCell};
use core::fmt::{self, Write};
use core::marker::PhantomData;
use core::ops::Range;

use crate::buf::{self, BufString, BufVec};
use crate::{Allocator, Context};

use super::access::{Access, Shared};
use super::rich_error::{RichError, Step};
use super::ErrorMarker;

#[cfg(all(not(loom), feature = "alloc"))]
use crate::allocator::System;

type BufPair<'a, A> = (Range<usize>, BufString<'a, A>);

/// A rich context which uses allocations and tracks the exact location of
/// errors.
///
/// This uses the provided allocator to allocate memory for the collected
/// diagnostics. The allocator to use can be provided using
/// [`RichContext::with_alloc`].
///
/// The default constructor is only available when the `alloc` feature is
/// enabled, and will use the [`System`] allocator.
pub struct RichContext<'a, A, M>
where
    A: 'a + ?Sized + Allocator,
{
    alloc: &'a A,
    mark: Cell<usize>,
    errors: UnsafeCell<BufVec<'a, BufPair<'a, A>, A>>,
    path: UnsafeCell<BufVec<'a, Step<BufString<'a, A>>, A>>,
    // How many elements of `path` we've gone over capacity.
    path_cap: Cell<usize>,
    include_type: bool,
    access: Access,
    _marker: PhantomData<M>,
}

impl<'a, A, M> RichContext<'a, A, M> where A: ?Sized + Allocator {}

#[cfg(all(not(loom), feature = "alloc"))]
impl<M> RichContext<'static, System, M> {
    /// Construct a new context which uses the system allocator for memory.
    #[inline]
    pub fn new() -> Self {
        Self::with_alloc(&crate::allocator::SYSTEM)
    }
}

#[cfg(all(not(loom), feature = "alloc"))]
impl<M> Default for RichContext<'static, System, M> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, A, M> RichContext<'a, A, M>
where
    A: 'a + ?Sized + Allocator,
{
    /// Construct a new context which uses allocations to a fixed but
    /// configurable number of diagnostics.
    pub fn with_alloc(alloc: &'a A) -> Self {
        let errors = BufVec::new_in(alloc);
        let path = BufVec::new_in(alloc);

        Self {
            alloc,
            mark: Cell::new(0),
            errors: UnsafeCell::new(errors),
            path: UnsafeCell::new(path),
            path_cap: Cell::new(0),
            include_type: false,
            access: Access::new(),
            _marker: PhantomData,
        }
    }

    /// Configure the context to visualize type information, and not just
    /// variant and fields.
    pub fn include_type(&mut self) -> &mut Self {
        self.include_type = true;
        self
    }

    /// Generate a line-separated report of all collected errors.
    pub fn report(&self) -> Report<'_, 'a, A> {
        Report {
            errors: self.errors(),
        }
    }

    /// Iterate over all collected errors.
    pub fn errors(&self) -> Errors<'_, 'a, A> {
        let access = self.access.shared();

        Errors {
            path: unsafe { (*self.path.get()).as_slice() },
            errors: unsafe { (*self.errors.get()).as_slice() },
            index: 0,
            path_cap: self.path_cap.get(),
            _access: access,
        }
    }

    /// Push an error into the collection.
    fn push_error(&self, range: Range<usize>, error: BufString<'a, A>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        unsafe {
            _ = (*self.errors.get()).push((range, error));
        }
    }

    /// Push a path.
    fn push_path(&self, step: Step<BufString<'a, A>>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        let path = unsafe { &mut (*self.path.get()) };

        if !path.push(step) {
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

    fn format_string<T>(&self, value: T) -> Option<BufString<'a, A>>
    where
        T: fmt::Display,
    {
        let mut string = BufString::new_in(self.alloc);
        write!(string, "{value}").ok()?;
        Some(string)
    }
}

impl<'a, A, M> Context for RichContext<'a, A, M>
where
    A: 'a + ?Sized + Allocator,
    M: 'static,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = usize;
    type Allocator = A;
    type BufString<'this> = BufString<'this, A> where Self: 'this;

    #[inline]
    fn clear(&self) {
        self.mark.set(0);
        let _access = self.access.exclusive();

        // SAFETY: We have acquired exclusive access just above.
        unsafe {
            (*self.errors.get()).clear();
            (*self.path.get()).clear();
        }
    }

    #[inline]
    fn alloc(&self) -> &Self::Allocator {
        &self.alloc
    }

    #[inline]
    fn collect_string<T>(&self, value: &T) -> Result<Self::BufString<'_>, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        buf::collect_string(self, value)
    }

    #[inline]
    fn custom<T>(&self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        if let Some(string) = self.format_string(message) {
            self.push_error(self.mark.get()..self.mark.get(), string);
        }

        ErrorMarker
    }

    #[inline]
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        if let Some(string) = self.format_string(message) {
            self.push_error(self.mark.get()..self.mark.get(), string);
        }

        ErrorMarker
    }

    #[inline]
    fn marked_message<T>(&self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        if let Some(string) = self.format_string(message) {
            self.push_error(mark..self.mark.get(), string);
        }

        ErrorMarker
    }

    #[inline]
    fn marked_custom<T>(&self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        if let Some(string) = self.format_string(message) {
            self.push_error(mark..self.mark.get(), string);
        }

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
    fn enter_named_field<T>(&self, name: &'static str, _: &T)
    where
        T: ?Sized + fmt::Display,
    {
        self.push_path(Step::Named(name));
    }

    #[inline]
    fn enter_unnamed_field<T>(&self, index: u32, _: &T)
    where
        T: ?Sized + fmt::Display,
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
        if let Some(string) = self.format_string(field) {
            self.push_path(Step::Key(string));
        }
    }

    #[inline]
    fn leave_map_key(&self) {
        self.pop_path();
    }
}

/// A line-separated report of all errors.
pub struct Report<'a, 'buf, A>
where
    A: 'buf + ?Sized + Allocator,
{
    errors: Errors<'a, 'buf, A>,
}

impl<'a, 'buf, A> fmt::Display for Report<'a, 'buf, A>
where
    A: 'buf + ?Sized + Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in self.errors.clone() {
            writeln!(f, "{error}")?;
        }

        Ok(())
    }
}

/// An iterator over available errors.
pub struct Errors<'a, 'buf, A>
where
    A: 'buf + ?Sized + Allocator,
{
    path: &'a [Step<BufString<'buf, A>>],
    errors: &'a [(Range<usize>, BufString<'buf, A>)],
    index: usize,
    path_cap: usize,
    _access: Shared<'a>,
}

impl<'a, 'buf, A> Iterator for Errors<'a, 'buf, A>
where
    A: 'buf + ?Sized + Allocator,
{
    type Item = RichError<'a, BufString<'buf, A>, BufString<'buf, A>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (range, error) = self.errors.get(self.index)?;
        self.index += 1;

        Some(RichError::new(
            self.path,
            self.path_cap,
            range.clone(),
            error,
        ))
    }
}

impl<'a, 'buf, A> Clone for Errors<'a, 'buf, A>
where
    A: ?Sized + Allocator,
{
    fn clone(&self) -> Self {
        Self {
            path: self.path,
            errors: self.errors,
            index: self.index,
            path_cap: self.path_cap,
            _access: self._access.clone(),
        }
    }
}
