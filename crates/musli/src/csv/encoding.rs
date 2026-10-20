use core::marker;

#[cfg(feature = "alloc")]
use rust_alloc::string::String;
#[cfg(feature = "alloc")]
use rust_alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::alloc::Global;
use crate::mode::Text;
use crate::{Allocator, Context, Decode, Encode, IntoWriter, Writer};

use super::de::CsvDecoder;
use super::en::CsvEncoder;
#[cfg(feature = "alloc")]
use super::error::Error;
use super::parser::{IntoParser, Parser};
use super::reader::Reader;
use super::writer::Writer as CsvWriter;

#[allow(unused)]
const DEFAULT: Encoding = Encoding::new();

crate::macros::bare_encoding!(Text, DEFAULT, csv, IntoParser, IntoWriter);

/// Encode the given value to a [`String`] using the default [`Encoding`].
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::csv;
/// # use musli::csv::Error;
///
/// #[derive(Debug, PartialEq, Decode, Encode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let people = vec![
///     Person { name: "Aristotle".to_string(), age: 61 },
///     Person { name: "Plato".to_string(), age: 80 },
/// ];
///
/// let data = csv::to_string(&people)?;
/// assert_eq!(data, "name,age\nAristotle,61\nPlato,80\n");
///
/// let decoded: Vec<Person> = csv::from_str(&data)?;
/// assert_eq!(decoded, people);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Encode<Text>,
{
    DEFAULT.to_string(value)
}

/// Decode the given type `T` from the given string using the default
/// [`Encoding`].
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::csv;
/// # use musli::csv::Error;
///
/// #[derive(Debug, PartialEq, Decode, Encode)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let people: Vec<Person> = csv::from_str("age,name\n61,Aristotle\n80,Plato\n")?;
///
/// assert_eq!(people, [
///     Person { name: "Aristotle".to_string(), age: 61 },
///     Person { name: "Plato".to_string(), age: 80 },
/// ]);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[inline]
pub fn from_str<'de, T>(string: &'de str) -> Result<T, Error>
where
    T: Decode<'de, Text, Global>,
{
    DEFAULT.from_str(string)
}

/// The record terminator used by an [`Encoding`].
///
/// This is modelled after the `Terminator` of the `csv` crate.
///
/// # Examples
///
/// ```
/// use musli::csv::{Encoding, Terminator};
/// # use musli::csv::Error;
///
/// const CRLF: Encoding = Encoding::new().with_terminator(Terminator::CRLF);
///
/// let data = CRLF.to_string(&[(1, 2), (3, 4)])?;
/// assert_eq!(data, "0,1\r\n1,2\r\n3,4\r\n");
/// # Ok::<_, Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Terminator {
    /// Records are terminated by `\r\n` when writing.
    ///
    /// When reading, any of `\r\n`, `\n` or `\r` terminate a record.
    CRLF,
    /// Records are terminated by the given byte.
    ///
    /// When reading with `Terminator::Any(b'\n')`, which is the default, any
    /// of `\r\n`, `\n` or `\r` also terminate a record. For any other byte,
    /// only that byte terminates a record.
    Any(u8),
}

impl Terminator {
    /// The single byte which terminates a record when reading, if any. If this
    /// is `None` then any of `\r\n`, `\n` or `\r` terminates a record.
    #[inline]
    pub(crate) const fn read_byte(self) -> Option<u8> {
        match self {
            Terminator::CRLF | Terminator::Any(b'\n') => None,
            Terminator::Any(b) => Some(b),
        }
    }

    /// The bytes written to terminate a record.
    #[inline]
    pub(crate) const fn write_bytes(&self) -> &[u8] {
        match self {
            Terminator::CRLF => b"\r\n",
            Terminator::Any(b) => core::slice::from_ref(b),
        }
    }
}

/// Configuration shared by everything which encodes or decodes CSV.
#[derive(Clone, Copy)]
pub(crate) struct Config {
    pub(crate) delimiter: u8,
    pub(crate) quote: u8,
    pub(crate) terminator: Terminator,
    pub(crate) has_headers: bool,
    pub(crate) flexible: bool,
}

/// Setting up encoding with parameters.
///
/// The configuration is modelled after the `ReaderBuilder` and `WriterBuilder`
/// types of the `csv` crate, and applies to both reading and writing:
///
/// * [`Encoding::with_delimiter`] sets the field delimiter, which defaults to
///   `,`.
/// * [`Encoding::with_quote`] sets the quote character, which defaults to `"`.
/// * [`Encoding::with_terminator`] sets the record terminator, which defaults
///   to `\n`. See [`Terminator`].
/// * [`Encoding::with_headers`] sets whether the first row is a header row,
///   which is the default.
/// * [`Encoding::with_flexible`] permits records with differing numbers of
///   fields, which is off by default.
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::csv::Encoding;
/// # use musli::csv::Error;
///
/// const TSV: Encoding = Encoding::new().with_delimiter(b'\t').with_headers(false);
///
/// #[derive(Debug, PartialEq, Decode, Encode)]
/// struct Point(u32, u32);
///
/// let points = vec![Point(1, 2), Point(3, 4)];
///
/// let data = TSV.to_string(&points)?;
/// assert_eq!(data, "1\t2\n3\t4\n");
///
/// let decoded: Vec<Point> = TSV.from_str(&data)?;
/// assert_eq!(decoded, points);
/// # Ok::<_, Error>(())
/// ```
pub struct Encoding<M = Text>
where
    M: 'static,
{
    config: Config,
    _marker: marker::PhantomData<M>,
}

impl Default for Encoding<Text> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Encoding<Text> {
    /// Construct a new [`Encoding`] with the default configuration.
    ///
    /// You can modify this using the available factory methods:
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::{Encoding, Terminator};
    /// # use musli::csv::Error;
    ///
    /// const CONFIG: Encoding<Csv> = Encoding::new()
    ///     .with_mode()
    ///     .with_delimiter(b';')
    ///     .with_terminator(Terminator::CRLF);
    ///
    /// // Mode marker indicating that some attributes should only apply when
    /// // we're encoding in a CSV mode.
    /// enum Csv {}
    ///
    /// #[derive(Debug, PartialEq, Encode, Decode)]
    /// #[musli(mode = Csv, name_all = "name")]
    /// struct Person<'a> {
    ///     #[musli(mode = Csv, name = "Name")]
    ///     name: &'a str,
    ///     #[musli(mode = Csv, name = "Age")]
    ///     age: u32,
    /// }
    ///
    /// let expected = Person {
    ///     name: "Aristotle",
    ///     age: 61,
    /// };
    ///
    /// let out = CONFIG.to_string(&expected)?;
    /// assert_eq!(out, "Name;Age\r\nAristotle;61\r\n");
    /// let actual = CONFIG.from_str(&out)?;
    /// assert_eq!(expected, actual);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Encoding {
            config: Config {
                delimiter: b',',
                quote: b'"',
                terminator: Terminator::Any(b'\n'),
                has_headers: true,
                flexible: false,
            },
            _marker: marker::PhantomData,
        }
    }
}

impl<M> Encoding<M>
where
    M: 'static,
{
    /// Change the mode of the encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    ///
    /// enum Custom {}
    ///
    /// const CONFIG: Encoding<Custom> = Encoding::new().with_mode();
    /// ```
    #[inline]
    pub const fn with_mode<T>(self) -> Encoding<T>
    where
        T: 'static,
    {
        Encoding {
            config: self.config,
            _marker: marker::PhantomData,
        }
    }

    /// Set the field delimiter, which defaults to `,`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new().with_delimiter(b';');
    ///
    /// let data = ENCODING.to_string(&[(1, "a;b")])?;
    /// assert_eq!(data, "0;1\n1;\"a;b\"\n");
    ///
    /// let decoded: Vec<(u32, String)> = ENCODING.from_str(&data)?;
    /// assert_eq!(decoded, [(1, String::from("a;b"))]);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub const fn with_delimiter(self, delimiter: u8) -> Self {
        Self {
            config: Config {
                delimiter,
                ..self.config
            },
            ..self
        }
    }

    /// Set the quote character, which defaults to `"`.
    ///
    /// Fields which contain the delimiter, the quote character or a line
    /// terminator are surrounded by quotes when written, and quotes inside of
    /// them are escaped by doubling them.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new().with_quote(b'\'').with_headers(false);
    ///
    /// let data = ENCODING.to_string(&[("it's", "a,b")])?;
    /// assert_eq!(data, "'it''s','a,b'\n");
    ///
    /// let decoded: Vec<(String, String)> = ENCODING.from_str(&data)?;
    /// assert_eq!(decoded, [(String::from("it's"), String::from("a,b"))]);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub const fn with_quote(self, quote: u8) -> Self {
        Self {
            config: Config {
                quote,
                ..self.config
            },
            ..self
        }
    }

    /// Set the record terminator, which defaults to `Terminator::Any(b'\n')`.
    ///
    /// See [`Terminator`] for how terminators are treated when reading.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::{Encoding, Terminator};
    /// # use musli::csv::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new()
    ///     .with_terminator(Terminator::CRLF)
    ///     .with_headers(false);
    ///
    /// let data = ENCODING.to_string(&[(1, 2), (3, 4)])?;
    /// assert_eq!(data, "1,2\r\n3,4\r\n");
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub const fn with_terminator(self, terminator: Terminator) -> Self {
        Self {
            config: Config {
                terminator,
                ..self.config
            },
            ..self
        }
    }

    /// Set whether the first row is a header row, which is the default.
    ///
    /// When headers are enabled, a header row is written before the first
    /// record. For structs with named fields it contains the field names, for
    /// other records, like tuples, it contains the position of each field
    /// starting at `0`.
    ///
    /// When reading with headers enabled the first row is always treated as a
    /// header row, and fields in structs are matched by name against it. This
    /// means that fields can be in any order, unknown columns are skipped and
    /// missing columns can be handled with `#[musli(default)]`.
    ///
    /// When headers are disabled, fields of structs are identified by their
    /// position, so this only works for structs whose fields are named by
    /// index, such as tuple structs or structs using `#[musli(name_all =
    /// "index")]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const NO_HEADERS: Encoding = Encoding::new().with_headers(false);
    ///
    /// #[derive(Debug, PartialEq, Decode, Encode)]
    /// #[musli(name_all = "index")]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let people = vec![Person { name: "Aristotle".to_string(), age: 61 }];
    ///
    /// assert_eq!(NO_HEADERS.to_string(&people)?, "Aristotle,61\n");
    /// assert_eq!(musli::csv::to_string(&people)?, "0,1\nAristotle,61\n");
    ///
    /// let decoded: Vec<Person> = NO_HEADERS.from_str("Plato,80\n")?;
    /// assert_eq!(decoded, [Person { name: "Plato".to_string(), age: 80 }]);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub const fn with_headers(self, has_headers: bool) -> Self {
        Self {
            config: Config {
                has_headers,
                ..self.config
            },
            ..self
        }
    }

    /// Set whether records may have a differing number of fields, which is
    /// off by default.
    ///
    /// When this is disabled, every record must have the same number of
    /// fields as the header row, or as the first record if headers are
    /// disabled.
    ///
    /// When this is enabled and a struct is decoded from a record which is
    /// shorter than the header row, the missing trailing fields are treated as
    /// missing and can be handled with `#[musli(default)]`. Extra fields
    /// without a header are skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const FLEXIBLE: Encoding = Encoding::new().with_flexible(true);
    ///
    /// #[derive(Debug, PartialEq, Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     #[musli(default)]
    ///     age: Option<u32>,
    /// }
    ///
    /// let input = "name,age\nAristotle\nPlato,80\n";
    ///
    /// assert!(musli::csv::from_str::<Vec<Person>>(input).is_err());
    ///
    /// let people: Vec<Person> = FLEXIBLE.from_str(input)?;
    ///
    /// assert_eq!(people, [
    ///     Person { name: "Aristotle".to_string(), age: None },
    ///     Person { name: "Plato".to_string(), age: Some(80) },
    /// ]);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub const fn with_flexible(self, flexible: bool) -> Self {
        Self {
            config: Config {
                flexible,
                ..self.config
            },
            ..self
        }
    }

    /// Construct a record-oriented [`Reader`] over the given input using the
    /// global allocator to store the header row.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Decode, Encode)]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let mut reader = ENCODING.reader("name,age\nAristotle,61\nPlato,80\n");
    ///
    /// let mut people = Vec::new();
    ///
    /// for person in reader.deserialize::<Person<'_>>() {
    ///     people.push(person?);
    /// }
    ///
    /// assert_eq!(people, [
    ///     Person { name: "Aristotle", age: 61 },
    ///     Person { name: "Plato", age: 80 },
    /// ]);
    /// # Ok::<_, Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn reader<'de, B>(self, input: &'de B) -> Reader<'de, M, Global>
    where
        B: ?Sized + AsRef<[u8]>,
    {
        Reader::new(self.config, input.as_ref(), Global::new())
    }

    /// Construct a record-oriented [`Reader`] over the given input using the
    /// given allocator to store the header row.
    ///
    /// Records read from the reader have to be decoded using a [`Context`]
    /// which uses the same allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::alloc::{ArrayBuffer, Slice};
    /// use musli::context;
    /// use musli::csv::Encoding;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Decode, Encode)]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let mut buf = ArrayBuffer::new();
    /// let alloc = Slice::new(&mut buf);
    /// let cx = context::new_in(&alloc);
    ///
    /// let mut reader = ENCODING.reader_in("name,age\nAristotle,61\n", &alloc);
    ///
    /// let Ok(Some(person)) = reader.next_record_with::<_, Person<'_>>(&cx) else {
    ///     panic!("failed to read record");
    /// };
    ///
    /// assert_eq!(person, Person { name: "Aristotle", age: 61 });
    /// assert!(matches!(reader.next_record_with::<_, Person<'_>>(&cx), Ok(None)));
    /// ```
    #[inline]
    pub fn reader_in<'de, B, A>(self, input: &'de B, alloc: A) -> Reader<'de, M, A>
    where
        B: ?Sized + AsRef<[u8]>,
        A: Allocator,
    {
        Reader::new(self.config, input.as_ref(), alloc)
    }

    /// Construct a record-oriented [`Writer`][CsvWriter] which writes records
    /// to the given [`Writer`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Debug, PartialEq, Decode, Encode)]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let mut writer = ENCODING.writer(Vec::new());
    /// writer.serialize(&Person { name: "Aristotle", age: 61 })?;
    /// writer.serialize(&Person { name: "Plato", age: 80 })?;
    ///
    /// let data = writer.into_inner();
    /// assert_eq!(data, b"name,age\nAristotle,61\nPlato,80\n");
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn writer<W>(self, writer: W) -> CsvWriter<W, M>
    where
        W: Writer,
    {
        CsvWriter::new(self.config, writer)
    }

    /// Construct the decoder used for the given parser.
    #[inline]
    fn decoder<'de, C, P>(self, cx: C, parser: P) -> CsvDecoder<P, C, M>
    where
        C: Context,
        P: Parser<'de>,
    {
        CsvDecoder::new(cx, parser, self.config)
    }

    /// Encode `value` into `writer`.
    #[inline]
    fn encode_into<C, W, T>(self, cx: C, writer: W, value: &T) -> Result<(), C::Error>
    where
        C: Context,
        W: Writer,
        T: ?Sized + Encode<M>,
    {
        T::encode(value, CsvEncoder::new(cx, writer, self.config, value))
    }

    crate::macros::encoding_impls!(
        custom,
        M,
        csv,
        self.decoder,
        IntoParser::into_parser,
        crate::csv::parser::Parser,
        IntoWriter::into_writer,
    );

    /// Encode the given value to a [`String`] using the current [`Encoding`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let data = ENCODING.to_string(&Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 61,
    /// })?;
    ///
    /// assert_eq!(data, "name,age\nAristotle,61\n");
    ///
    /// let person: Person = ENCODING.from_str(&data[..])?;
    /// assert_eq!(person.name, "Aristotle");
    /// assert_eq!(person.age, 61);
    /// # Ok::<_, Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn to_string<T>(self, value: &T) -> Result<String, Error>
    where
        T: ?Sized + Encode<M>,
    {
        let cx = crate::context::new().with_error();
        self.to_string_with(&cx, value)
    }

    /// Encode the given value to a [`String`] using the current [`Encoding`].
    ///
    /// This is the same as [`Encoding::to_string`] but allows for using a
    /// configurable [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// const ENCODING: Encoding = Encoding::new();
    ///
    /// #[derive(Decode, Encode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let cx = musli::context::new().with_error();
    ///
    /// let data = ENCODING.to_string_with(&cx, &Person {
    ///     name: "Aristotle".to_string(),
    ///     age: 61,
    /// })?;
    ///
    /// let person: Person = ENCODING.from_str_with(&cx, &data[..])?;
    /// assert_eq!(person.name, "Aristotle");
    /// assert_eq!(person.age, 61);
    /// # Ok::<_, Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn to_string_with<T, C>(self, cx: C, value: &T) -> Result<String, C::Error>
    where
        C: Context,
        T: ?Sized + Encode<M>,
    {
        cx.clear();
        let mut data = Vec::with_capacity(128);
        self.encode_into(cx, &mut data, value)?;

        // NB: Delimiters, quotes and terminators are configurable bytes, so
        // the output is only guaranteed to be valid UTF-8 if they are ASCII.
        match String::from_utf8(data) {
            Ok(string) => Ok(string),
            Err(..) => Err(cx.message("Encoded CSV is not valid UTF-8")),
        }
    }
}

impl<M> Clone for Encoding<M> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<M> Copy for Encoding<M> {}
