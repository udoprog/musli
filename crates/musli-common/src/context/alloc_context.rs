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

/// A rich context which uses allocations.
pub struct AllocContext<'buf, E> {
    mark: usize,
    string: ptr::NonNull<String>,
    errors: Vec<(Range<usize>, E)>,
    path: Vec<Step>,
    _marker: PhantomData<(&'buf mut String, E)>,
}

impl<'buf, E> AllocContext<'buf, E> {
    /// Construct a new context which uses allocations to store arbitrary
    /// amounts of diagnostics about decoding.
    ///
    /// Or at least until we run out of memory.
    pub fn new(string: &'buf mut String) -> Self {
        Self {
            mark: 0,
            string: string.into(),
            errors: Vec::new(),
            path: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<'buf, E> Context<'buf> for AllocContext<'buf, E>
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
        self.errors.push((self.mark..self.mark, E::from(error)));
        de::Error
    }

    #[inline(always)]
    fn custom<T>(&mut self, message: T) -> Self::Error
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        self.errors.push((self.mark..self.mark, E::custom(message)));
        de::Error
    }

    #[inline(always)]
    fn message<T>(&mut self, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        let path = format_path(&self.path);
        self.errors.push((
            self.mark..self.mark,
            E::message(format_args!("{path}: {message}")),
        ));
        de::Error
    }

    #[inline]
    fn marked_message<T>(&mut self, mark: Self::Mark, message: T) -> Self::Error
    where
        T: fmt::Display,
    {
        let now = self.mark;
        let path = format_path(&self.path);
        self.errors.push((
            mark..self.mark,
            E::message(format_args!("{path}: {message} (at {mark}..{now})")),
        ));
        de::Error
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.mark = self.mark.wrapping_add(n);
    }

    fn store_string(&mut self, s: &str) {
        // SAFETY: we're holding onto a mutable reference to the string so it
        // must be live for the duration of the context.
        let string = unsafe { self.string.as_mut() };

        string.clear();
        string.push_str(s);
    }

    fn get_string<'a>(&self) -> Option<&'buf str> {
        // SAFETY: we're holding onto a mutable reference to the string so it
        // must be live for the duration of the context.
        let string = unsafe { self.string.as_ref() };

        if string.is_empty() {
            None
        } else {
            Some(string)
        }
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
        self.path.push(Step::Struct(name));
    }

    #[inline]
    fn trace_leave_struct(&mut self) {
        self.path.pop();
    }

    #[inline]
    fn trace_enter_enum(&mut self, name: &'static str) {
        self.path.push(Step::Enum(name));
    }

    #[inline]
    fn trace_leave_enum(&mut self) {
        self.path.pop();
    }

    #[inline]
    fn trace_enter_variant<T>(&mut self, name: &'static str, _: T) -> Self::TraceVariant {
        self.path.push(Step::Variant(name));
    }

    fn trace_leave_variant(&mut self, _: Self::TraceVariant) {
        self.path.pop();
    }
}

impl<'buf, E> fmt::Display for AllocContext<'buf, E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.errors.is_empty() {
            write!(f, "no errors")?;
            return Ok(());
        }

        for (range, error) in &self.errors {
            if range.start != 0 || range.end != 0 {
                if range.start == range.end {
                    writeln!(f, "{error} (at byte {})", range.start)?;
                } else {
                    writeln!(f, "{error} (at bytes {}-{})", range.start, range.end)?;
                }
            } else {
                writeln!(f, "{error}")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Step {
    Struct(&'static str),
    Enum(&'static str),
    Variant(&'static str),
    NamedField(&'static str),
    UnnamedField(u32),
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

        for step in self.path {
            match *step {
                Step::Struct(name) => {
                    if take(&mut has_prior) {
                        write!(f, " / ")?;
                    }

                    write!(f, "{name} {{ ")?;
                    has_type = true;
                }
                Step::Enum(name) => {
                    if take(&mut has_prior) {
                        write!(f, " / ")?;
                    }

                    write!(f, "{name}::")?;
                }
                Step::Variant(name) => {
                    write!(f, "{name} {{ ")?;
                    has_type = true;
                }
                Step::NamedField(name) => {
                    write!(f, ".{name}")?;

                    if take(&mut has_type) {
                        write!(f, " }}")?;
                    }

                    has_prior = true;
                }
                Step::UnnamedField(index) => {
                    write!(f, ".{index}")?;

                    if take(&mut has_type) {
                        write!(f, " }}")?;
                    }

                    has_prior = true;
                }
            }
        }

        if has_type {
            write!(f, " }}")?;
        }

        Ok(())
    }
}
