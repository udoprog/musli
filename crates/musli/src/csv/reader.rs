use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use crate::alloc::Global;
#[cfg(feature = "alloc")]
use crate::mode::Text;
use crate::{Allocator, Context, Decode};

use super::de::{DocState, Headers, RecordDecoder};
use super::encoding::Config;
#[cfg(feature = "alloc")]
use super::error::Error;
use super::parser::{self, Parser, SliceParser};

/// A record-oriented CSV reader.
///
/// This is modelled after the `Reader` of the `csv` crate, and is constructed
/// using [`Encoding::reader`] or [`Encoding::reader_in`].
///
/// If headers are enabled, which is the default, the first row of the input is
/// treated as a header row and is not returned as a record. Fields in structs
/// are matched by name against it.
///
/// [`Encoding::reader`]: super::Encoding::reader
/// [`Encoding::reader_in`]: super::Encoding::reader_in
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::csv::Encoding;
/// # use musli::csv::Error;
///
/// #[derive(Debug, PartialEq, Decode, Encode)]
/// struct City<'a> {
///     city: &'a str,
///     population: Option<u64>,
/// }
///
/// let input = "\
/// city,country,population
/// Boston,United States,4628910
/// Concord,United States,
/// ";
///
/// let mut reader = Encoding::new().reader(input);
///
/// let headers = reader.headers()?.expect("missing header row");
/// assert!(headers.iter().eq(["city", "country", "population"]));
///
/// let cities = reader.deserialize::<City<'_>>().collect::<Result<Vec<_>, _>>()?;
///
/// assert_eq!(cities, [
///     City { city: "Boston", population: Some(4628910) },
///     City { city: "Concord", population: None },
/// ]);
/// # Ok::<_, Error>(())
/// ```
#[cfg(feature = "alloc")]
pub struct Reader<'de, M = Text, A = Global>
where
    A: Allocator,
{
    cfg: Config,
    parser: SliceParser<'de>,
    doc: DocState<A>,
    _marker: PhantomData<M>,
}

/// A record-oriented CSV reader.
///
/// This is modelled after the `Reader` of the `csv` crate, and is constructed
/// using [`Encoding::reader_in`].
///
/// [`Encoding::reader_in`]: super::Encoding::reader_in
#[cfg(not(feature = "alloc"))]
pub struct Reader<'de, M, A>
where
    A: Allocator,
{
    cfg: Config,
    parser: SliceParser<'de>,
    doc: DocState<A>,
    _marker: PhantomData<M>,
}

impl<'de, M, A> Reader<'de, M, A>
where
    M: 'static,
    A: Allocator,
{
    #[inline]
    pub(crate) fn new(cfg: Config, input: &'de [u8], alloc: A) -> Self {
        Self {
            cfg,
            parser: SliceParser::new(input),
            doc: DocState::new(alloc),
            _marker: PhantomData,
        }
    }

    /// Read the header row if headers are enabled and it hasn't been read yet,
    /// and return it.
    ///
    /// This returns `None` if headers are disabled or if the input is empty.
    ///
    /// This is the same as [`Reader::headers`] but allows for using a
    /// configurable [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    ///
    /// let cx = musli::context::new();
    /// let mut reader = Encoding::new().reader("name,age\nAristotle,61\n");
    ///
    /// let Ok(Some(headers)) = reader.headers_with(&cx) else {
    ///     panic!("missing header row");
    /// };
    ///
    /// assert!(headers.iter().eq(["name", "age"]));
    /// ```
    #[inline]
    pub fn headers_with<C>(&mut self, cx: C) -> Result<Option<&Headers<A>>, C::Error>
    where
        C: Context,
    {
        self.doc.ensure_header(cx, &mut self.parser, &self.cfg)?;
        Ok(self.doc.headers())
    }

    /// Read and decode the next record.
    ///
    /// Returns `None` once there are no more records.
    ///
    /// This is the same as [`Reader::next_record`] but allows for using a
    /// configurable [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    ///
    /// let cx = musli::context::new();
    /// let mut reader = Encoding::new().with_headers(false).reader("1,2\n3,4\n");
    ///
    /// assert!(matches!(reader.next_record_with::<_, (u32, u32)>(&cx), Ok(Some((1, 2)))));
    /// assert!(matches!(reader.next_record_with::<_, (u32, u32)>(&cx), Ok(Some((3, 4)))));
    /// assert!(matches!(reader.next_record_with::<_, (u32, u32)>(&cx), Ok(None)));
    /// ```
    #[inline]
    pub fn next_record_with<C, T>(&mut self, cx: C) -> Result<Option<T>, C::Error>
    where
        C: Context<Allocator = A>,
        T: Decode<'de, M, A>,
    {
        cx.clear();

        self.doc.ensure_header(cx, &mut self.parser, &self.cfg)?;

        if !parser::begin_record(cx, &mut self.parser, &self.cfg) {
            return Ok(None);
        }

        let value = T::decode(RecordDecoder::<_, _, M, _>::new(
            cx,
            &mut self.parser,
            self.cfg,
            &mut self.doc,
            true,
        ))?;

        Ok(Some(value))
    }

    /// Test if all records have been read.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut reader = Encoding::new().with_headers(false).reader("1\n\n");
    /// assert!(!reader.is_done());
    /// assert_eq!(reader.next_record::<u32>()?, Some(1));
    /// assert!(reader.is_done());
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn is_done(&self) -> bool {
        self.parser
            .remaining()
            .iter()
            .all(|&b| self.cfg.is_terminator(b))
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<'de, M> Reader<'de, M, Global>
where
    M: 'static,
{
    /// Read the header row if headers are enabled and it hasn't been read yet,
    /// and return it.
    ///
    /// This returns `None` if headers are disabled or if the input is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut reader = Encoding::new().reader("name,age\nAristotle,61\n");
    /// let headers = reader.headers()?.expect("missing header row");
    /// assert!(headers.iter().eq(["name", "age"]));
    ///
    /// let mut reader = Encoding::new().with_headers(false).reader("Aristotle,61\n");
    /// assert!(reader.headers()?.is_none());
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn headers(&mut self) -> Result<Option<&Headers<Global>>, Error> {
        let cx = crate::context::new().with_error();
        self.headers_with(&cx)
    }

    /// Read and decode the next record.
    ///
    /// Returns `None` once there are no more records.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// #[derive(Debug, PartialEq, Decode, Encode)]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let mut reader = Encoding::new().reader("age,name\n61,Aristotle\n");
    ///
    /// assert_eq!(reader.next_record()?, Some(Person { name: "Aristotle", age: 61 }));
    /// assert_eq!(reader.next_record::<Person<'_>>()?, None);
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn next_record<T>(&mut self) -> Result<Option<T>, Error>
    where
        T: Decode<'de, M, Global>,
    {
        let cx = crate::context::new().with_error();
        self.next_record_with(&cx)
    }

    /// Iterate over decoded records.
    ///
    /// Once an error has been returned, the iterator stops producing records.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Decode, Encode};
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// #[derive(Debug, PartialEq, Decode, Encode)]
    /// struct Person<'a> {
    ///     name: &'a str,
    ///     age: u32,
    /// }
    ///
    /// let mut reader = Encoding::new().reader("name,age\nAristotle,61\nPlato,eighty\nSocrates,70\n");
    /// let mut it = reader.deserialize::<Person<'_>>();
    ///
    /// assert_eq!(it.next().transpose()?, Some(Person { name: "Aristotle", age: 61 }));
    /// assert!(it.next().expect("expected an error").is_err());
    /// assert!(it.next().is_none());
    /// # Ok::<_, Error>(())
    /// ```
    #[inline]
    pub fn deserialize<T>(&mut self) -> DeserializeRecords<'_, 'de, M, T>
    where
        T: Decode<'de, M, Global>,
    {
        DeserializeRecords {
            reader: self,
            done: false,
            _marker: PhantomData,
        }
    }
}

/// An iterator over decoded records, as returned by [`Reader::deserialize`].
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub struct DeserializeRecords<'a, 'de, M, T> {
    reader: &'a mut Reader<'de, M, Global>,
    done: bool,
    _marker: PhantomData<T>,
}

#[cfg(feature = "alloc")]
impl<'de, M, T> Iterator for DeserializeRecords<'_, 'de, M, T>
where
    M: 'static,
    T: Decode<'de, M, Global>,
{
    type Item = Result<T, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.reader.next_record() {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(error) => {
                self.done = true;
                Some(Err(error))
            }
        }
    }
}
