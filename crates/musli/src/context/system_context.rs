use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(not(loom))]
use crate::allocator::System;
use crate::buf::{self, BufString, BufVec};
use crate::{Allocator, Context};

use super::access::{self, Access};
use super::rich_error::{RichError, Step};
use super::ErrorMarker;

type BufTriplet<E> = (Vec<Step<String>>, Range<usize>, E);

/// A rich context dynamically allocating space using the system allocator.
pub struct SystemContext<A, M> {
    access: Access,
    mark: Cell<usize>,
    alloc: A,
    errors: UnsafeCell<Vec<BufTriplet<String>>>,
    path: UnsafeCell<Vec<Step<String>>>,
    include_type: bool,
    _marker: PhantomData<M>,
}

#[cfg(not(loom))]
impl<M> SystemContext<&'static System, M> {
    /// Construct a new context which uses the system allocator for memory.
    #[inline]
    pub fn new() -> Self {
        Self::with_alloc(&crate::allocator::SYSTEM)
    }
}

#[cfg(not(loom))]
impl<M> Default for SystemContext<&'static System, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A, M> SystemContext<A, M> {
    /// Construct a new context which uses allocations to store arbitrary
    /// amounts of diagnostics about decoding.
    #[inline]
    pub fn with_alloc(alloc: A) -> Self {
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

    /// Generate a line-separated report of all collected errors.
    pub fn report(&self) -> Report<'_> {
        Report {
            errors: self.errors(),
        }
    }

    /// Iterate over all collected errors.
    pub fn errors(&self) -> Errors<'_> {
        let access = self.access.shared();

        // SAFETY: We've checked above that we have shared access.
        Errors {
            errors: unsafe { &*self.errors.get() },
            index: 0,
            _access: access,
        }
    }
}

impl<A, M> SystemContext<A, M>
where
    A: Allocator,
{
    fn push_error(&self, range: Range<usize>, message: String) {
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

impl<A, M> Context for SystemContext<A, M>
where
    A: Allocator,
    M: 'static,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = usize;
    type Buf<'this> = A::Buf<'this, u8> where Self: 'this;
    type BufString<'this> = BufString<A::Buf<'this, u8>> where Self: 'this;

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
    fn alloc(&self) -> Option<BufVec<Self::Buf<'_>>> {
        Some(BufVec::new(self.alloc.alloc()?))
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
        self.push_error(self.mark.get()..self.mark.get(), message.to_string());
        ErrorMarker
    }

    #[inline]
    fn message<T>(&self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.push_error(self.mark.get()..self.mark.get(), message.to_string());
        ErrorMarker
    }

    #[inline]
    fn marked_message<T>(&self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.push_error(mark..self.mark.get(), message.to_string());
        ErrorMarker
    }

    #[inline]
    fn marked_custom<T>(&self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.push_error(mark..self.mark.get(), message.to_string());
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
        self.push_path(Step::Key(field.to_string()));
    }

    #[inline]
    fn leave_map_key(&self) {
        self.pop_path();
    }
}

/// A line-separated report of all errors.
pub struct Report<'a> {
    errors: Errors<'a>,
}

impl fmt::Display for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in self.errors.clone() {
            writeln!(f, "{error}")?;
        }

        Ok(())
    }
}

/// An iterator over collected errors.
#[derive(Clone)]
pub struct Errors<'a> {
    errors: &'a [BufTriplet<String>],
    index: usize,
    // NB: Drop order is significant, drop the shared access last.
    _access: access::Shared<'a>,
}

impl<'a> Iterator for Errors<'a> {
    type Item = RichError<'a, String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let (path, range, error) = self.errors.get(self.index)?;
        self.index += 1;
        Some(RichError::new(path, 0, range.clone(), error))
    }
}
