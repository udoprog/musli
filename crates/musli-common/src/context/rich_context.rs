use core::fmt;
use core::marker::PhantomData;
use core::mem::take;
use core::ops::Range;
use core::ptr;

use alloc::string::String;
use alloc::vec::Vec;

use musli::de;
use musli::error::Error;
use musli::Context;

/// A collected error which has been context decorated.
pub struct RichError<'a, E> {
    path: &'a [Step],
    range: Range<usize>,
    error: &'a E,
}

/// A rich context which uses allocations and tracks the exact location of every
/// error.
pub struct RichContext<'buf, E> {
    mark: usize,
    string: Option<ptr::NonNull<String>>,
    errors: Vec<(Vec<Step>, Range<usize>, E)>,
    path: Vec<Step>,
    include_type: bool,
    _marker: PhantomData<(&'buf mut String, E)>,
}

impl<'buf, E> RichContext<'buf, E> {
    /// Construct a new context which uses allocations to store arbitrary
    /// amounts of diagnostics about decoding.
    ///
    /// Or at least until we run out of memory.
    pub fn new(string: &'buf mut String) -> Self {
        Self {
            mark: 0,
            string: Some(string.into()),
            errors: Vec::new(),
            path: Vec::new(),
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
    pub fn iter(&self) -> impl Iterator<Item = RichError<'_, E>> {
        self.errors.iter().map(|(path, range, error)| RichError {
            path,
            range: range.clone(),
            error,
        })
    }
}

impl<'buf, E> Context<'buf> for RichContext<'buf, E>
where
    E: Error,
{
    type Input = E;
    type Error = de::Error;
    type TraceField = ();
    type TraceVariant = ();
    type Mark = usize;

    #[inline(always)]
    fn report<T>(&mut self, error: T) -> Self::Error
    where
        E: From<T>,
    {
        self.errors
            .push((self.path.clone(), self.mark..self.mark, E::from(error)));
        de::Error
    }

    #[inline]
    fn marked_report<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        E: From<T>,
    {
        self.errors
            .push((self.path.clone(), mark..self.mark, E::from(message)));
        de::Error
    }

    #[inline(always)]
    fn custom<T>(&mut self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.errors
            .push((self.path.clone(), self.mark..self.mark, E::custom(message)));
        de::Error
    }

    #[inline(always)]
    fn message<T>(&mut self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.errors
            .push((self.path.clone(), self.mark..self.mark, E::message(message)));
        de::Error
    }

    #[inline]
    fn marked_message<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        self.errors
            .push((self.path.clone(), mark..self.mark, E::message(message)));
        de::Error
    }

    #[inline]
    fn mark(&mut self) -> Self::Mark {
        self.mark
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.mark = self.mark.wrapping_add(n);
    }

    #[inline(always)]
    fn store_string(&mut self, s: &str) {
        if let Some(mut string) = self.string {
            // SAFETY: we're holding onto a mutable reference to the string so it
            // must be live for the duration of the context.
            let string = unsafe { string.as_mut() };
            string.clear();
            string.push_str(s);
        }
    }

    #[inline(always)]
    fn get_string<'a>(&self) -> Option<&'buf str> {
        let string = self.string?;
        // SAFETY: we're holding onto a mutable reference to the string so it
        // must be live for the duration of the context.
        let string = unsafe { string.as_ref() };
        Some(string)
    }

    #[inline]
    fn trace_enter_named_field<T>(&mut self, name: &'static str, _: T) -> Self::TraceField
    where
        T: fmt::Display,
    {
        self.path.push(Step::NamedField(name));
    }

    #[inline]
    fn trace_enter_unnamed_field<T>(&mut self, index: u32, _: T) -> Self::TraceField
    where
        T: fmt::Display,
    {
        self.path.push(Step::UnnamedField(index));
    }

    #[inline]
    fn trace_leave_field(&mut self, _: Self::TraceField) {
        self.path.pop();
    }

    #[inline]
    fn trace_enter_struct(&mut self, name: &'static str) {
        if self.include_type {
            self.path.push(Step::Struct(name));
        }
    }

    #[inline]
    fn trace_leave_struct(&mut self) {
        if self.include_type {
            self.path.pop();
        }
    }

    #[inline]
    fn trace_enter_enum(&mut self, name: &'static str) {
        if self.include_type {
            self.path.push(Step::Enum(name));
        }
    }

    #[inline]
    fn trace_leave_enum(&mut self) {
        if self.include_type {
            self.path.pop();
        }
    }

    #[inline]
    fn trace_enter_variant<T>(&mut self, name: &'static str, _: T) -> Self::TraceVariant {
        self.path.push(Step::Variant(name));
    }

    #[inline]
    fn trace_leave_variant(&mut self, _: Self::TraceVariant) {
        self.path.pop();
    }

    #[inline]
    fn trace_enter_sequence_index(&mut self, index: usize) {
        self.path.push(Step::Index(index));
    }

    #[inline]
    fn trace_leave_sequence_index(&mut self) {
        self.path.pop();
    }
}

impl<'buf, E> fmt::Display for RichError<'buf, E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = format_path(self.path);

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

#[derive(Debug, Clone)]
enum Step {
    Struct(&'static str),
    Enum(&'static str),
    Variant(&'static str),
    NamedField(&'static str),
    UnnamedField(u32),
    Index(usize),
}

fn format_path(path: &[Step]) -> impl fmt::Display + '_ {
    FormatPath { path }
}

struct FormatPath<'a> {
    path: &'a [Step],
}

impl<'a> fmt::Display for FormatPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_type = false;
        let mut has_prior = false;
        let mut level = 0;

        for step in self.path {
            match *step {
                Step::Struct(name) => {
                    if take(&mut has_prior) {
                        write!(f, " = ")?;
                    }

                    write!(f, "{name}")?;
                    has_type = true;
                }
                Step::Enum(name) => {
                    if take(&mut has_prior) {
                        write!(f, " = ")?;
                    }

                    write!(f, "{name}::")?;
                }
                Step::Variant(name) => {
                    if take(&mut has_prior) {
                        write!(f, " = ")?;
                    }

                    write!(f, "{name}")?;
                    has_type = true;
                }
                Step::NamedField(name) => {
                    if take(&mut has_type) {
                        write!(f, " {{ ")?;
                        level += 1;
                    }

                    write!(f, ".{name}")?;
                    has_prior = true;
                }
                Step::UnnamedField(index) => {
                    if take(&mut has_type) {
                        write!(f, " {{ ")?;
                        level += 1;
                    }

                    write!(f, ".{index}")?;
                    has_prior = true;
                }
                Step::Index(index) => {
                    if take(&mut has_type) {
                        write!(f, " {{ ")?;
                        level += 1;
                    }

                    write!(f, "[{index}]")?;
                    has_prior = true;
                }
            }
        }

        for _ in 0..level {
            write!(f, " }}")?;
        }

        Ok(())
    }
}
