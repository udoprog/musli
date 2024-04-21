use core::cell::{Cell, UnsafeCell};
use core::fmt::{self, Write};
use core::marker::PhantomData;
use core::ops::Range;

use musli_core::{Allocator, Context};

use crate::buf::{self, BufString};
use crate::fixed::FixedVec;

use super::access::{Access, Shared};
use super::rich_error::{RichError, Step};
use super::ErrorMarker;

type BufPair<'a, A> = (Range<usize>, BufString<<A as Allocator>::Buf<'a>>);

/// A rich context which uses allocations and tracks the exact location of
/// errors.
///
/// This will only store 4 errors by default, and support a path up to 16. To
/// control this, use the [`new_with`][StackContext::new_with] constructor.
pub struct StackContext<'a, const E: usize, const P: usize, A, M>
where
    A: ?Sized + Allocator,
{
    alloc: &'a A,
    mark: Cell<usize>,
    errors: UnsafeCell<FixedVec<BufPair<'a, A>, E>>,
    path: UnsafeCell<FixedVec<Step<BufString<A::Buf<'a>>>, P>>,
    // How many elements of `path` we've gone over capacity.
    path_cap: Cell<usize>,
    include_type: bool,
    access: Access,
    _marker: PhantomData<M>,
}

impl<'a, A, M> StackContext<'a, 16, 4, A, M>
where
    A: ?Sized + Allocator,
{
    /// Construct a new context which uses allocations to a fixed number of
    /// diagnostics.
    ///
    /// This uses the default values of:
    /// * The first 4 errors.
    /// * 16 path elements stored when tracing.
    pub fn new(alloc: &'a A) -> Self {
        Self::new_with(alloc)
    }
}

impl<'a, const E: usize, const P: usize, A, M> StackContext<'a, E, P, A, M>
where
    A: ?Sized + Allocator,
{
    /// Construct a new context which uses allocations to a fixed but
    /// configurable number of diagnostics.
    pub fn new_with(alloc: &'a A) -> Self {
        Self {
            alloc,
            mark: Cell::new(0),
            errors: UnsafeCell::new(FixedVec::new()),
            path: UnsafeCell::new(FixedVec::new()),
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
            path: unsafe { &*self.path.get() },
            errors: unsafe { &*self.errors.get() },
            index: 0,
            path_cap: self.path_cap.get(),
            _access: access,
        }
    }

    /// Push an error into the collection.
    fn push_error(&self, range: Range<usize>, error: BufString<A::Buf<'a>>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        unsafe {
            _ = (*self.errors.get()).try_push((range, error));
        }
    }

    /// Push a path.
    fn push_path(&self, step: Step<BufString<A::Buf<'a>>>) {
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

    fn format_string<T>(&self, value: T) -> Option<BufString<A::Buf<'a>>>
    where
        T: fmt::Display,
    {
        let buf = self.alloc.alloc()?;
        let mut string = BufString::new(buf);
        write!(string, "{value}").ok()?;
        Some(string)
    }
}

impl<'a, const E: usize, const P: usize, A, M> Context for StackContext<'a, E, P, A, M>
where
    A: ?Sized + Allocator,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = usize;
    type Buf<'this> = A::Buf<'this> where Self: 'this;
    type BufString<'this> = BufString<A::Buf<'this>> where Self: 'this;

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
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        self.alloc.alloc()
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
    path: &'a [Step<BufString<A::Buf<'buf>>>],
    errors: &'a [(Range<usize>, BufString<A::Buf<'buf>>)],
    index: usize,
    path_cap: usize,
    _access: Shared<'a>,
}

impl<'a, 'buf, A> Iterator for Errors<'a, 'buf, A>
where
    A: ?Sized + Allocator,
{
    type Item = RichError<'a, BufString<A::Buf<'buf>>, BufString<A::Buf<'buf>>>;

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
