use crate::alloc::Vec;
use crate::{Context, Writer};

/// A chunk of spaces used to write indentation without looping a byte at a
/// time.
const SPACES: [u8; 32] = [b' '; 32];

/// Configuration for pretty printing.
///
/// This is handed to [`Encoding::with_pretty`] and decides what the whitespace
/// inserted around structural elements looks like.
///
/// [`Encoding::with_pretty`]: crate::json::Encoding::with_pretty
///
/// # Examples
///
/// ```
/// use musli::json::Pretty;
///
/// const PRETTY: Pretty = Pretty::new().with_indent(4);
/// ```
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Pretty {
    indent: usize,
}

impl Pretty {
    /// Construct a new pretty printing configuration using two spaces per
    /// level of nesting.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::json::Pretty;
    ///
    /// const PRETTY: Pretty = Pretty::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self { indent: 2 }
    }

    /// Indent each level of nesting by `indent` spaces.
    ///
    /// An `indent` of `0` still breaks the document over multiple lines, it
    /// just does not indent them. A document without any incidental whitespace
    /// at all is written by a compact encoding instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::json::Pretty;
    ///
    /// const PRETTY: Pretty = Pretty::new().with_indent(4);
    /// ```
    #[inline]
    pub const fn with_indent(self, indent: usize) -> Self {
        Self { indent, ..self }
    }
}

impl Default for Pretty {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// A writer which indents structured output as it is being written.
///
/// This decorates another [`Writer`] and makes use of the structural signals
/// emitted by an encoder, such as [`begin_object`] and [`begin_array_element`],
/// to insert newlines and indentation. Everything else is passed through
/// verbatim.
///
/// Compact output does not use this writer at all, it writes to the underlying
/// writer directly where every structural signal is the default no-op.
///
/// [`begin_object`]: Writer::begin_object
/// [`begin_array_element`]: Writer::begin_array_element
pub(super) struct PrettyWriter<W> {
    writer: W,
    depth: usize,
    indent: usize,
}

impl<W> PrettyWriter<W>
where
    W: Writer,
{
    /// Construct a new pretty writer wrapping `writer`.
    #[inline]
    pub(super) const fn new(writer: W, pretty: Pretty) -> Self {
        Self {
            writer,
            depth: 0,
            indent: pretty.indent,
        }
    }

    /// Write a newline followed by the indentation of the current depth.
    #[inline]
    fn newline<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.writer.write_byte(cx, b'\n')?;

        let mut remaining = self.depth * self.indent;

        while remaining > 0 {
            let n = remaining.min(SPACES.len());
            self.writer.write_bytes(cx, &SPACES[..n])?;
            remaining -= n;
        }

        Ok(())
    }
}

impl<W> Writer for PrettyWriter<W>
where
    W: Writer,
{
    type Ok = W::Ok;
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn finish<C>(&mut self, cx: C) -> Result<Self::Ok, C::Error>
    where
        C: Context,
    {
        self.writer.finish(cx)
    }

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.writer.extend(cx, buffer)
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.writer.write_bytes(cx, bytes)
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: C, b: u8) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.writer.write_byte(cx, b)
    }

    #[inline]
    fn begin_object<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        _ = cx;
        self.depth += 1;
        Ok(())
    }

    #[inline]
    fn end_object<C>(&mut self, cx: C, empty: bool) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.depth -= 1;

        if !empty {
            self.newline(cx)?;
        }

        Ok(())
    }

    #[inline]
    fn begin_object_key<C>(&mut self, cx: C, first: bool) -> Result<(), C::Error>
    where
        C: Context,
    {
        _ = first;
        self.newline(cx)
    }

    #[inline]
    fn begin_object_value<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.writer.write_byte(cx, b' ')
    }

    #[inline]
    fn begin_array<C>(&mut self, cx: C) -> Result<(), C::Error>
    where
        C: Context,
    {
        _ = cx;
        self.depth += 1;
        Ok(())
    }

    #[inline]
    fn end_array<C>(&mut self, cx: C, empty: bool) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.depth -= 1;

        if !empty {
            self.newline(cx)?;
        }

        Ok(())
    }

    #[inline]
    fn begin_array_element<C>(&mut self, cx: C, first: bool) -> Result<(), C::Error>
    where
        C: Context,
    {
        _ = first;
        self.newline(cx)
    }
}
