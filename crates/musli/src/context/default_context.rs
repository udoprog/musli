#![allow(clippy::type_complexity)]

use core::cell::{Cell, UnsafeCell};
use core::fmt::{self, Write};
use core::marker::PhantomData;
use core::mem::take;
use core::ops::Range;
use core::slice;

#[cfg(feature = "alloc")]
use crate::alloc::System;
use crate::alloc::{self, Allocator, String, Vec};
use crate::Context;

use super::{Access, ErrorMarker, Shared};

/// The default context which uses an allocator to track the location of errors.
///
/// This uses the provided allocator to allocate memory for the collected
/// diagnostics. The allocator to use can be provided using [`with_alloc`].
///
/// The default constructor is only available when the `alloc` feature is
/// enabled, and will use the [`System`] allocator.
///
/// [`with_alloc`]: super::with_alloc
pub struct DefaultContext<'a, A, M>
where
    A: 'a + ?Sized + Allocator,
{
    alloc: &'a A,
    mark: Cell<usize>,
    errors: UnsafeCell<Vec<'a, (Range<usize>, String<'a, A>), A>>,
    path: UnsafeCell<Vec<'a, Step<'a, A>, A>>,
    // How many elements of `path` we've gone over capacity.
    cap: Cell<usize>,
    include_type: bool,
    access: Access,
    _marker: PhantomData<M>,
}

impl<A, M> DefaultContext<'_, A, M> where A: ?Sized + Allocator {}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<M> DefaultContext<'static, System, M> {
    /// Construct a new fully featured context which uses the [`System`]
    /// allocator for memory.
    ///
    /// [`System`]: crate::alloc::System
    #[inline]
    pub fn new() -> Self {
        Self::with_alloc(crate::alloc::system())
    }
}

#[cfg(feature = "alloc")]
impl<M> Default for DefaultContext<'static, System, M> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, A, M> DefaultContext<'a, A, M>
where
    A: 'a + ?Sized + Allocator,
{
    /// Construct a new context which uses allocations to a fixed but
    /// configurable number of diagnostics.
    pub(super) fn with_alloc(alloc: &'a A) -> Self {
        let errors = Vec::new_in(alloc);
        let path = Vec::new_in(alloc);

        Self {
            alloc,
            mark: Cell::new(0),
            errors: UnsafeCell::new(errors),
            path: UnsafeCell::new(path),
            cap: Cell::new(0),
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
            errors: unsafe { (*self.errors.get()).as_slice().iter() },
            cap: self.cap.get(),
            _access: access,
        }
    }

    /// Push an error into the collection.
    fn push_error(&self, range: Range<usize>, error: String<'a, A>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        unsafe {
            _ = (*self.errors.get()).push((range, error));
        }
    }

    /// Push a path.
    fn push_path(&self, step: Step<'a, A>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        let path = unsafe { &mut (*self.path.get()) };

        if !path.push(step) {
            self.cap.set(self.cap.get() + 1);
        }
    }

    /// Pop the last path.
    fn pop_path(&self) {
        let cap = self.cap.get();

        if cap > 0 {
            self.cap.set(cap - 1);
            return;
        }

        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        unsafe {
            (*self.path.get()).pop();
        }
    }

    fn format_string<T>(&self, value: T) -> Option<String<'a, A>>
    where
        T: fmt::Display,
    {
        let mut string = String::new_in(self.alloc);
        write!(string, "{value}").ok()?;
        Some(string)
    }
}

impl<'a, A, M> Context for DefaultContext<'a, A, M>
where
    A: 'a + ?Sized + Allocator,
    M: 'static,
{
    type Mode = M;
    type Error = ErrorMarker;
    type Mark = usize;
    type Allocator = A;
    type String<'this>
        = String<'this, A>
    where
        Self: 'this;

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
        self.alloc
    }

    #[inline]
    fn collect_string<T>(&self, value: &T) -> Result<Self::String<'_>, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        alloc::collect_string(self, value)
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
    fn marked_message<T>(&self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        if let Some(string) = self.format_string(message) {
            self.push_error(*mark..self.mark.get(), string);
        }

        ErrorMarker
    }

    #[inline]
    fn marked_custom<T>(&self, mark: &Self::Mark, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        if let Some(string) = self.format_string(message) {
            self.push_error(*mark..self.mark.get(), string);
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
pub struct Report<'b, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    errors: Errors<'b, 'a, A>,
}

impl<'a, A> fmt::Display for Report<'_, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in self.errors.clone() {
            writeln!(f, "{error}")?;
        }

        Ok(())
    }
}

/// An iterator over available errors.
///
/// See [`DefaultContext::errors`].
pub struct Errors<'b, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    path: &'b [Step<'a, A>],
    cap: usize,
    errors: slice::Iter<'b, (Range<usize>, String<'a, A>)>,
    _access: Shared<'b>,
}

impl<'b, 'a, A> Iterator for Errors<'b, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    type Item = Error<'b, 'a, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (range, error) = self.errors.next()?;
        Some(Error::new(self.path, self.cap, range.clone(), error))
    }
}

impl<A> Clone for Errors<'_, '_, A>
where
    A: ?Sized + Allocator,
{
    fn clone(&self) -> Self {
        Self {
            path: self.path,
            cap: self.cap,
            errors: self.errors.clone(),
            _access: self._access.clone(),
        }
    }
}

/// A collected error which has been context decorated.
pub struct Error<'b, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    path: &'b [Step<'a, A>],
    cap: usize,
    range: Range<usize>,
    error: &'b str,
}

impl<'b, 'a, A> Error<'b, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    fn new(path: &'b [Step<'a, A>], cap: usize, range: Range<usize>, error: &'b str) -> Self {
        Self {
            path,
            cap,
            range,
            error,
        }
    }
}

impl<'a, A> fmt::Display for Error<'_, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = FormatPath::new(self.path, self.cap);

        if self.range.start != 0 || self.range.end != 0 {
            if self.range.start == self.range.end {
                write!(f, "{path}: {} (at byte {})", self.error, self.range.start)?;
            } else {
                write!(
                    f,
                    "{path}: {} (at bytes {}-{})",
                    self.error, self.range.start, self.range.end
                )?;
            }
        } else {
            write!(f, "{path}: {}", self.error)?;
        }

        Ok(())
    }
}

/// A single traced step.
#[derive(Debug)]
pub(crate) enum Step<'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    Struct(&'static str),
    Enum(&'static str),
    Variant(&'static str),
    Named(&'static str),
    Unnamed(u32),
    Index(usize),
    Key(String<'a, A>),
}

struct FormatPath<'b, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    path: &'b [Step<'a, A>],
    cap: usize,
}

impl<'b, 'a, A> FormatPath<'b, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    pub(crate) fn new(path: &'b [Step<'a, A>], cap: usize) -> Self {
        Self { path, cap }
    }
}

impl<'a, A> fmt::Display for FormatPath<'_, 'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_type = false;
        let mut has_field = false;
        let mut level = 0;

        for step in self.path {
            match step {
                Step::Struct(name) => {
                    if take(&mut has_field) {
                        write!(f, " = ")?;
                    }

                    write!(f, "{name}")?;
                    has_type = true;
                }
                Step::Enum(name) => {
                    if take(&mut has_field) {
                        write!(f, " = ")?;
                    }

                    write!(f, "{name}::")?;
                }
                Step::Variant(name) => {
                    if take(&mut has_field) {
                        write!(f, " = ")?;
                    }

                    write!(f, "{name}")?;
                    has_type = true;
                }
                Step::Named(name) => {
                    if take(&mut has_type) {
                        write!(f, " {{ ")?;
                        level += 1;
                    }

                    write!(f, ".{name}")?;
                    has_field = true;
                }
                Step::Unnamed(index) => {
                    if take(&mut has_type) {
                        write!(f, " {{ ")?;
                        level += 1;
                    }

                    write!(f, ".{index}")?;
                    has_field = true;
                }
                Step::Index(index) => {
                    if take(&mut has_type) {
                        write!(f, " {{ ")?;
                        level += 1;
                    }

                    write!(f, "[{index}]")?;
                    has_field = true;
                }
                Step::Key(key) => {
                    if take(&mut has_type) {
                        write!(f, " {{ ")?;
                        level += 1;
                    }

                    write!(f, "[{}]", key)?;
                    has_field = true;
                }
            }
        }

        for _ in 0..level {
            write!(f, " }}")?;
        }

        match self.cap {
            0 => {}
            1 => write!(f, " .. *one capped step*")?,
            n => write!(f, " .. *{n} capped steps*")?,
        }

        Ok(())
    }
}
