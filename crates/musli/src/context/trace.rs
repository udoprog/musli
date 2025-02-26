#![allow(clippy::type_complexity)]

use core::cell::{Cell, UnsafeCell};
use core::fmt;
use core::mem::take;
use core::ops::Range;
use core::slice;

use crate::alloc::{Allocator, String, Vec};

use super::{Access, Shared};

mod sealed {
    use crate::alloc::Allocator;

    pub trait Sealed {}
    impl<A> Sealed for super::WithTraceImpl<A> where A: Allocator {}
    impl Sealed for super::NoTraceImpl {}
    impl Sealed for super::Trace {}
    impl Sealed for super::NoTrace {}
}

/// Trait for marker types indicating the tracing mode to use.
pub trait TraceMode: self::sealed::Sealed {
    #[doc(hidden)]
    type Impl<A>: TraceImpl<A>
    where
        A: Allocator;

    #[doc(hidden)]
    fn new_in<A>(alloc: A) -> Self::Impl<A>
    where
        A: Allocator;
}

/// The trait governing how tracing works in a [`DefaultContext`].
///
/// [`DefaultContext`]: super::DefaultContext
pub trait TraceImpl<A>: self::sealed::Sealed {
    #[doc(hidden)]
    type Mark;

    #[doc(hidden)]
    fn clear(&self);

    #[doc(hidden)]
    fn advance(&self, n: usize);

    #[doc(hidden)]
    fn mark(&self) -> Self::Mark;

    #[doc(hidden)]
    fn restore(&self, mark: &Self::Mark);

    #[doc(hidden)]
    fn custom<T>(&self, alloc: A, message: &T)
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug;

    #[doc(hidden)]
    fn message<T>(&self, alloc: A, message: &T)
    where
        T: fmt::Display;

    #[doc(hidden)]
    fn message_at<T>(&self, alloc: A, mark: &Self::Mark, message: &T)
    where
        T: fmt::Display;

    #[doc(hidden)]
    fn custom_at<T>(&self, alloc: A, mark: &Self::Mark, message: &T)
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug;

    #[doc(hidden)]
    fn enter_named_field<T>(&self, name: &'static str, field: &T)
    where
        T: fmt::Display;

    #[doc(hidden)]
    fn enter_unnamed_field<T>(&self, index: u32, name: &T)
    where
        T: fmt::Display;

    #[doc(hidden)]
    fn leave_field(&self);

    #[doc(hidden)]
    fn enter_struct(&self, name: &'static str);

    #[doc(hidden)]
    fn leave_struct(&self);

    #[doc(hidden)]
    fn enter_enum(&self, name: &'static str);

    #[doc(hidden)]
    fn leave_enum(&self);

    #[doc(hidden)]
    fn enter_variant<T>(&self, name: &'static str, _: &T)
    where
        T: fmt::Display;

    #[doc(hidden)]
    fn leave_variant(&self);

    #[doc(hidden)]
    fn enter_sequence_index(&self, index: usize);

    #[doc(hidden)]
    fn leave_sequence_index(&self);

    #[doc(hidden)]
    fn enter_map_key<T>(&self, alloc: A, field: &T)
    where
        T: fmt::Display;

    #[doc(hidden)]
    fn leave_map_key(&self);
}

/// Marker type indicating that tracing is enabled.
///
/// See [`DefaultContext::with_trace`] for more information.
///
/// [`DefaultContext::with_trace`]: super::DefaultContext::with_trace
#[non_exhaustive]
pub struct Trace;

impl TraceMode for Trace {
    type Impl<A>
        = WithTraceImpl<A>
    where
        A: Allocator;

    #[inline]
    fn new_in<A>(alloc: A) -> Self::Impl<A>
    where
        A: Allocator,
    {
        WithTraceImpl::new_in(alloc)
    }
}

/// Trace configuration indicating that tracing is enabled through the allocator
/// `A`.
pub struct WithTraceImpl<A>
where
    A: Allocator,
{
    mark: Cell<usize>,
    errors: UnsafeCell<Vec<(Range<usize>, String<A>), A>>,
    path: UnsafeCell<Vec<Step<A>, A>>,
    // How many elements of `path` we've gone over capacity.
    cap: Cell<usize>,
    include_type: bool,
    access: Access,
}

impl<A> WithTraceImpl<A>
where
    A: Allocator,
{
    /// Construct a new tracing context inside of the given allocator.
    #[inline]
    pub(super) fn new_in(alloc: A) -> Self {
        let errors = Vec::new_in(alloc);
        let path = Vec::new_in(alloc);

        Self {
            mark: Cell::new(0),
            errors: UnsafeCell::new(errors),
            path: UnsafeCell::new(path),
            cap: Cell::new(0),
            include_type: false,
            access: Access::new(),
        }
    }

    #[inline]
    pub(super) fn include_type(&mut self) {
        self.include_type = true;
    }

    /// Generate a line-separated report of all collected errors.
    #[inline]
    pub fn report(&self) -> Report<'_, A> {
        Report {
            errors: self.errors(),
        }
    }

    #[inline]
    pub(super) fn errors(&self) -> Errors<'_, A> {
        let access = self.access.shared();

        Errors {
            path: unsafe { (*self.path.get()).as_slice() },
            errors: unsafe { (*self.errors.get()).as_slice().iter() },
            cap: self.cap.get(),
            _access: access,
        }
    }

    /// Push a path.
    #[inline]
    fn push_path(&self, step: Step<A>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        let path = unsafe { &mut (*self.path.get()) };

        if path.push(step).is_err() {
            self.cap.set(&self.cap.get() + 1);
        }
    }

    /// Pop the last path.
    #[inline]
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

    #[inline]
    fn format_string<T>(&self, alloc: A, value: T) -> Option<String<A>>
    where
        T: fmt::Display,
    {
        use core::fmt::Write;

        let mut string = String::new_in(alloc);
        write!(string, "{value}").ok()?;
        Some(string)
    }

    /// Push an error into the collection.
    #[inline]
    fn push_error(&self, range: Range<usize>, error: String<A>) {
        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        unsafe {
            _ = (*self.errors.get()).push((range, error));
        }
    }
}

impl<A> TraceImpl<A> for WithTraceImpl<A>
where
    A: Allocator,
{
    type Mark = usize;

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
    fn advance(&self, n: usize) {
        self.mark.set(self.mark.get().wrapping_add(n));
    }

    #[inline]
    fn mark(&self) -> Self::Mark {
        self.mark.get()
    }

    #[inline]
    fn restore(&self, mark: &Self::Mark) {
        self.mark.set(*mark);
    }

    #[inline]
    fn custom<T>(&self, alloc: A, message: &T)
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        if let Some(string) = self.format_string(alloc, message) {
            self.push_error(self.mark.get()..self.mark.get(), string);
        }
    }

    #[inline]
    fn message<T>(&self, alloc: A, message: &T)
    where
        T: fmt::Display,
    {
        if let Some(string) = self.format_string(alloc, message) {
            self.push_error(self.mark.get()..self.mark.get(), string);
        }
    }

    #[inline]
    fn message_at<T>(&self, alloc: A, mark: &Self::Mark, message: &T)
    where
        T: fmt::Display,
    {
        if let Some(string) = self.format_string(alloc, message) {
            self.push_error(*mark..self.mark.get(), string);
        }
    }

    #[inline]
    fn custom_at<T>(&self, alloc: A, mark: &Self::Mark, message: &T)
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        if let Some(string) = self.format_string(alloc, message) {
            self.push_error(*mark..self.mark.get(), string);
        }
    }

    #[inline]
    fn enter_named_field<T>(&self, name: &'static str, _: &T)
    where
        T: fmt::Display,
    {
        self.push_path(Step::Named(name));
    }

    #[inline]
    fn enter_unnamed_field<T>(&self, index: u32, _: &T)
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
    fn enter_variant<T>(&self, name: &'static str, _: &T) {
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
    fn enter_map_key<T>(&self, alloc: A, field: &T)
    where
        T: fmt::Display,
    {
        if let Some(string) = self.format_string(alloc, field) {
            self.push_path(Step::Key(string));
        }
    }

    #[inline]
    fn leave_map_key(&self) {
        self.pop_path();
    }
}

#[non_exhaustive]
pub struct NoTraceImpl;

/// Trace configuration indicating that tracing is fully disabled.
///
/// This is the default behavior you get when calling [`new`] or [`new_in`].
///
/// [`new`]: super::new
/// [`new_in`]: super::new_in
#[non_exhaustive]
pub struct NoTrace;

impl TraceMode for NoTrace {
    type Impl<A>
        = NoTraceImpl
    where
        A: Allocator;

    #[inline]
    fn new_in<A>(_: A) -> Self::Impl<A>
    where
        A: Allocator,
    {
        NoTraceImpl
    }
}

impl<A> TraceImpl<A> for NoTraceImpl
where
    A: Allocator,
{
    type Mark = ();

    #[inline]
    fn clear(&self) {}

    #[inline]
    fn mark(&self) -> Self::Mark {}

    #[inline]
    fn restore(&self, mark: &Self::Mark) -> Self::Mark {
        _ = mark;
    }

    #[inline]
    fn advance(&self, n: usize) {
        _ = n;
    }

    #[inline]
    fn custom<T>(&self, alloc: A, message: &T)
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        _ = alloc;
        _ = message;
    }

    #[inline]
    fn message<T>(&self, alloc: A, message: &T)
    where
        T: fmt::Display,
    {
        _ = alloc;
        _ = message;
    }

    #[inline]
    fn message_at<T>(&self, alloc: A, mark: &Self::Mark, message: &T)
    where
        T: fmt::Display,
    {
        _ = alloc;
        _ = mark;
        _ = message;
    }

    #[inline]
    fn custom_at<T>(&self, alloc: A, mark: &Self::Mark, message: &T)
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        _ = alloc;
        _ = mark;
        _ = message;
    }

    #[inline]
    fn enter_named_field<T>(&self, name: &'static str, field: &T)
    where
        T: fmt::Display,
    {
        _ = name;
        _ = field;
    }

    #[inline]
    fn enter_unnamed_field<T>(&self, index: u32, field: &T)
    where
        T: fmt::Display,
    {
        _ = index;
        _ = field;
    }

    #[inline]
    fn leave_field(&self) {}

    #[inline]
    fn enter_struct(&self, name: &'static str) {
        _ = name;
    }

    #[inline]
    fn leave_struct(&self) {}

    #[inline]
    fn enter_enum(&self, name: &'static str) {
        _ = name;
    }

    #[inline]
    fn leave_enum(&self) {}

    #[inline]
    fn enter_variant<T>(&self, name: &'static str, variant: &T)
    where
        T: fmt::Display,
    {
        _ = name;
        _ = variant;
    }

    #[inline]
    fn leave_variant(&self) {}

    #[inline]
    fn enter_sequence_index(&self, index: usize) {
        _ = index;
    }

    #[inline]
    fn leave_sequence_index(&self) {}

    #[inline]
    fn enter_map_key<T>(&self, alloc: A, field: &T)
    where
        T: fmt::Display,
    {
        _ = alloc;
        _ = field;
    }

    #[inline]
    fn leave_map_key(&self) {}
}

/// A line-separated report of all errors.
///
/// See [`DefaultContext::report`].
///
/// [`DefaultContext::report`]: super::DefaultContext::report
pub struct Report<'a, A>
where
    A: Allocator,
{
    errors: Errors<'a, A>,
}

impl<A> fmt::Display for Report<'_, A>
where
    A: Allocator,
{
    #[inline]
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
///
/// [`DefaultContext::errors`]: super::DefaultContext::errors
pub struct Errors<'a, A>
where
    A: Allocator,
{
    path: &'a [Step<A>],
    cap: usize,
    errors: slice::Iter<'a, (Range<usize>, String<A>)>,
    _access: Shared<'a>,
}

impl<'a, A> Iterator for Errors<'a, A>
where
    A: Allocator,
{
    type Item = Error<'a, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (range, error) = self.errors.next()?;
        Some(Error::new(self.path, self.cap, range.clone(), error))
    }
}

impl<A> Clone for Errors<'_, A>
where
    A: Allocator,
{
    #[inline]
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
pub struct Error<'a, A>
where
    A: Allocator,
{
    path: &'a [Step<A>],
    cap: usize,
    range: Range<usize>,
    error: &'a str,
}

impl<'a, A> Error<'a, A>
where
    A: Allocator,
{
    #[inline]
    fn new(path: &'a [Step<A>], cap: usize, range: Range<usize>, error: &'a str) -> Self {
        Self {
            path,
            cap,
            range,
            error,
        }
    }
}

impl<A> fmt::Display for Error<'_, A>
where
    A: Allocator,
{
    #[inline]
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
enum Step<A>
where
    A: Allocator,
{
    Struct(&'static str),
    Enum(&'static str),
    Variant(&'static str),
    Named(&'static str),
    Unnamed(u32),
    Index(usize),
    Key(String<A>),
}

struct FormatPath<'a, A>
where
    A: Allocator,
{
    path: &'a [Step<A>],
    cap: usize,
}

impl<'a, A> FormatPath<'a, A>
where
    A: Allocator,
{
    #[inline]
    fn new(path: &'a [Step<A>], cap: usize) -> Self {
        Self { path, cap }
    }
}

impl<A> fmt::Display for FormatPath<'_, A>
where
    A: Allocator,
{
    #[inline]
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
