use core::fmt;
use core::ops::Range;

use arrayvec::{ArrayString, ArrayVec};
use musli::context::Error;
use musli::Context;

use crate::allocator::Allocator;
use crate::context::rich_error::{RichError, Step};

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
    mark: usize,
    alloc: A,
    error: Option<(Range<usize>, E)>,
    path: ArrayVec<Step<ArrayString<S>>, P>,
    path_cap: usize,
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
            mark: 0,
            alloc,
            error: None,
            path: ArrayVec::new(),
            path_cap: 0,
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
    pub fn iter(&self) -> impl Iterator<Item = RichError<'_, ArrayString<S>, E>> {
        self.error
            .iter()
            .map(|(range, error)| RichError::new(&self.path, self.path_cap, range.clone(), error))
    }

    /// Push an error into the collection.
    fn push_error(&mut self, range: Range<usize>, error: E) {
        self.error = Some((range, error));
    }

    /// Push a path.
    fn push_path(&mut self, step: Step<ArrayString<S>>) {
        if self.path.try_push(step).is_err() {
            self.path_cap += 1;
        }
    }

    /// Pop the last path.
    fn pop_path(&mut self) {
        if self.path_cap > 0 {
            self.path_cap -= 1;
            return;
        }

        self.path.pop();
    }
}

impl<const V: usize, const S: usize, A, E> Context for NoStdContext<V, S, A, E>
where
    A: Allocator,
    E: musli::error::Error,
{
    type Input = E;
    type Error = Error;
    type Mark = usize;
    type Buf = A::Buf;

    #[inline(always)]
    fn alloc(&mut self) -> Self::Buf {
        self.alloc.alloc()
    }

    #[inline(always)]
    fn report<T>(&mut self, error: T) -> Self::Error
    where
        E: From<T>,
    {
        self.push_error(self.mark..self.mark, E::from(error));
        Error
    }

    #[inline]
    fn marked_report<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        E: From<T>,
    {
        self.push_error(mark..self.mark, E::from(message));
        Error
    }

    #[inline(always)]
    fn custom<T>(&mut self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.push_error(self.mark..self.mark, E::custom(message));
        Error
    }

    #[inline(always)]
    fn message<T>(&mut self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.push_error(self.mark..self.mark, E::message(message));
        Error
    }

    #[inline]
    fn marked_message<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.push_error(mark..self.mark, E::message(message));
        Error
    }

    #[inline]
    fn mark(&mut self) -> Self::Mark {
        self.mark
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.mark = self.mark.wrapping_add(n);
    }

    #[inline]
    fn enter_named_field<T>(&mut self, name: &'static str, _: T)
    where
        T: fmt::Display,
    {
        self.push_path(Step::Named(name));
    }

    #[inline]
    fn enter_unnamed_field<T>(&mut self, index: u32, _: T)
    where
        T: fmt::Display,
    {
        self.push_path(Step::Unnamed(index));
    }

    #[inline]
    fn leave_field(&mut self) {
        self.pop_path();
    }

    #[inline]
    fn enter_struct(&mut self, name: &'static str) {
        if self.include_type {
            self.push_path(Step::Struct(name));
        }
    }

    #[inline]
    fn leave_struct(&mut self) {
        if self.include_type {
            self.pop_path();
        }
    }

    #[inline]
    fn enter_enum(&mut self, name: &'static str) {
        if self.include_type {
            self.push_path(Step::Enum(name));
        }
    }

    #[inline]
    fn leave_enum(&mut self) {
        if self.include_type {
            self.pop_path();
        }
    }

    #[inline]
    fn enter_variant<T>(&mut self, name: &'static str, _: T) {
        self.push_path(Step::Variant(name));
    }

    #[inline]
    fn leave_variant(&mut self) {
        self.pop_path();
    }

    #[inline]
    fn enter_sequence_index(&mut self, index: usize) {
        self.push_path(Step::Index(index));
    }

    #[inline]
    fn leave_sequence_index(&mut self) {
        self.pop_path();
    }

    #[inline]
    fn enter_map_key<T>(&mut self, field: T)
    where
        T: fmt::Display,
    {
        use core::fmt::Write;
        let mut string = ArrayString::new();
        let _ = write!(string, "{}", field);
        self.push_path(Step::Key(string));
    }

    #[inline]
    fn leave_map_key(&mut self) {
        self.pop_path();
    }
}
