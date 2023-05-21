use core::fmt;
use core::mem::take;
use core::ops::Range;

/// A collected error which has been context decorated.
pub struct RichError<'a, S, E> {
    path: &'a [Step<S>],
    path_cap: usize,
    range: Range<usize>,
    error: &'a E,
}

impl<'a, S, E> RichError<'a, S, E> {
    pub(crate) fn new(
        path: &'a [Step<S>],
        path_cap: usize,
        range: Range<usize>,
        error: &'a E,
    ) -> Self {
        Self {
            path,
            path_cap,
            range,
            error,
        }
    }
}

impl<'buf, S, E> fmt::Display for RichError<'buf, S, E>
where
    S: AsRef<str>,
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = format_path(self.path, self.path_cap);

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
#[derive(Debug, Clone)]
pub(crate) enum Step<S> {
    Struct(&'static str),
    Enum(&'static str),
    Variant(&'static str),
    Named(&'static str),
    Unnamed(u32),
    Index(usize),
    Key(S),
}

fn format_path<S>(path: &[Step<S>], path_cap: usize) -> impl fmt::Display + '_
where
    S: AsRef<str>,
{
    FormatPath { path, path_cap }
}

struct FormatPath<'a, S> {
    path: &'a [Step<S>],
    path_cap: usize,
}

impl<'a, S> fmt::Display for FormatPath<'a, S>
where
    S: AsRef<str>,
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

                    write!(f, "[{}]", key.as_ref())?;
                    has_field = true;
                }
            }
        }

        for _ in 0..level {
            write!(f, " }}")?;
        }

        match self.path_cap {
            0 => {}
            1 => write!(f, " .. *one capped step*")?,
            n => write!(f, " .. *{n} capped steps*")?,
        }

        Ok(())
    }
}
