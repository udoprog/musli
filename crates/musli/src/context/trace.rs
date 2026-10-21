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

    /// The location in the input that an error raised now is reported at.
    ///
    /// If `start` is specified, the location spans from it to the current
    /// position.
    #[doc(hidden)]
    fn location(&self, start: Option<&Self::Mark>) -> Option<Location<'_, A>>
    where
        A: Allocator;
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
        let cap = self.cap.get();

        // Once a step has been capped, every step entered under it is capped
        // as well. Otherwise popping would remove them in the wrong order.
        if cap > 0 {
            self.cap.set(cap + 1);
            return;
        }

        let _access = self.access.exclusive();

        // SAFETY: We've checked that we have exclusive access just above.
        let path = unsafe { &mut (*self.path.get()) };

        if path.push(step).is_err() {
            self.cap.set(self.cap.get() + 1);
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
        self.cap.set(0);
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
        match self.format_string(alloc, field) {
            Some(string) => self.push_path(Step::Key(string)),
            // The key could not be formatted, so count it as a capped step so
            // that leaving it does not pop its parent.
            None => self.cap.set(self.cap.get() + 1),
        }
    }

    #[inline]
    fn leave_map_key(&self) {
        self.pop_path();
    }

    #[inline]
    fn location(&self, start: Option<&Self::Mark>) -> Option<Location<'_, A>> {
        let access = self.access.shared();
        let end = self.mark.get();
        let start = start.copied().unwrap_or(end);

        Some(Location {
            // SAFETY: We hold shared access to the path while the location is
            // alive.
            path: unsafe { (*self.path.get()).as_slice() },
            cap: self.cap.get(),
            range: start..end,
            _access: access,
        })
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

    #[inline]
    fn location(&self, start: Option<&Self::Mark>) -> Option<Location<'_, A>> {
        _ = start;
        None
    }
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
        format_traced(f, self.path, self.cap, &self.range, &self.error)
    }
}

/// Format `message` as having been raised at the given path and range.
///
/// An empty path is left out, and so is a range of `0..0`, since decoders
/// which do not read from a buffer of input never advance the position.
fn format_traced<A>(
    f: &mut fmt::Formatter<'_>,
    path: &[Step<A>],
    cap: usize,
    range: &Range<usize>,
    message: &dyn fmt::Display,
) -> fmt::Result
where
    A: Allocator,
{
    if !path.is_empty() || cap > 0 {
        write!(f, "{}: ", FormatPath::new(path, cap))?;
    }

    message.fmt(f)?;

    if range.start != 0 || range.end != 0 {
        if range.start == range.end {
            write!(f, " (at byte {})", range.start)?;
        } else {
            write!(f, " (at bytes {}-{})", range.start, range.end)?;
        }
    }

    Ok(())
}

/// Where in the input an error was raised, as recorded by a context which
/// traces.
///
/// A context which traces, such as one built with
/// [`DefaultContext::with_trace`], passes this to
/// [`ContextError::traced_message`] and [`ContextError::traced_custom`] when it
/// constructs an error, so that the error can carry the path of fields,
/// variants and indexes which were being decoded, and the position in the input
/// it was raised at.
///
/// [`DefaultContext::with_trace`]: super::DefaultContext::with_trace
/// [`ContextError::traced_message`]: super::ContextError::traced_message
/// [`ContextError::traced_custom`]: super::ContextError::traced_custom
pub struct Location<'a, A>
where
    A: Allocator,
{
    path: &'a [Step<A>],
    cap: usize,
    range: Range<usize>,
    _access: Shared<'a>,
}

impl<'a, A> Location<'a, A>
where
    A: Allocator,
{
    /// Test if the error was raised outside of any field, variant, sequence
    /// index or map key, in which case [`Location::path`] formats as an empty
    /// string.
    #[inline]
    pub fn is_root(&self) -> bool {
        self.path.is_empty() && self.cap == 0
    }

    /// The path to the value which was being decoded, such as
    /// `.field[2].name`.
    ///
    /// If the context includes types through [`DefaultContext::with_type`],
    /// the path also names the structs and enums involved.
    ///
    /// [`DefaultContext::with_type`]: super::DefaultContext::with_type
    #[inline]
    pub fn path(&self) -> impl fmt::Display + 'a {
        FormatPath::new(self.path, self.cap)
    }

    /// The range of bytes in the input the error was raised at.
    ///
    /// The range is empty when the error refers to a single position. It is
    /// `0..0` when the decoder does not read from a buffer of input, such as
    /// when decoding from a [`Value`].
    ///
    /// [`Value`]: crate::value::Value
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Format `message` decorated with this location, the same way
    /// [`DefaultContext::report`] formats each error.
    ///
    /// This produces `path: message (at byte N)`, leaving out the path if
    /// [`Location::is_root`] and the position if the range is `0..0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context::{self, ErrorMarker};
    /// use musli::json::Encoding;
    /// use musli::{Decode, Encode};
    ///
    /// #[derive(Debug, Decode, Encode)]
    /// struct Person {
    ///     name: String,
    /// }
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// let cx = context::new().with_trace().with_error::<String>();
    /// let error = ENCODING.from_str_with::<_, Person>(&cx, r#"{"name": 42}"#).unwrap_err();
    /// assert_eq!(error, ".name: Expected string, found <number> (at bytes 8-9)");
    /// ```
    ///
    /// [`DefaultContext::report`]: super::DefaultContext::report
    #[inline]
    pub fn with_message<M>(&self, message: M) -> impl fmt::Display
    where
        M: fmt::Display,
    {
        WithMessage {
            location: self,
            message,
        }
    }
}

impl<A> fmt::Display for Location<'_, A>
where
    A: Allocator,
{
    /// Formats the path, followed by the position if there is one.
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FormatPath::new(self.path, self.cap))?;

        if self.range.start != 0 || self.range.end != 0 {
            if !self.is_root() {
                write!(f, " ")?;
            }

            if self.range.start == self.range.end {
                write!(f, "(at byte {})", self.range.start)?;
            } else {
                write!(f, "(at bytes {}-{})", self.range.start, self.range.end)?;
            }
        }

        Ok(())
    }
}

impl<A> fmt::Debug for Location<'_, A>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Location")
            .field("path", &format_args!("{}", self.path()))
            .field("range", &self.range)
            .finish()
    }
}

struct WithMessage<'a, 'b, A, M>
where
    A: Allocator,
{
    location: &'a Location<'b, A>,
    message: M,
}

impl<A, M> fmt::Display for WithMessage<'_, '_, A, M>
where
    A: Allocator,
    M: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let l = self.location;
        format_traced(f, l.path, l.cap, &l.range, &self.message)
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

                    write!(f, "[{key}]")?;
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
