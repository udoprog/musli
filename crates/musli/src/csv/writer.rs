use core::marker::PhantomData;

use crate::mode::Text;
use crate::{Context, Encode};

use super::en::{EncState, Pass, RecordEncoder};
use super::encoding::Config;
#[cfg(feature = "alloc")]
use super::error::Error;

/// A record-oriented CSV writer.
///
/// This is modelled after the `Writer` of the `csv` crate, and is constructed
/// using [`Encoding::writer`].
///
/// Every call to [`Writer::serialize`] writes one record. If headers are
/// enabled, which is the default, a header row is written before the first
/// record. For structs it contains the names of their fields, and for other
/// records like tuples it contains the position of each field.
///
/// [`Encoding::writer`]: super::Encoding::writer
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli::csv::Encoding;
/// # use musli::csv::Error;
///
/// #[derive(Encode)]
/// struct City<'a> {
///     city: &'a str,
///     population: Option<u64>,
/// }
///
/// let mut data = Vec::new();
///
/// let mut writer = Encoding::new().writer(&mut data);
/// writer.serialize(&City { city: "Boston", population: Some(4628910) })?;
/// writer.serialize(&City { city: "Concord, MA", population: None })?;
///
/// assert_eq!(data, b"city,population\nBoston,4628910\n\"Concord, MA\",\n");
/// # Ok::<_, Error>(())
/// ```
pub struct Writer<W, M = Text> {
    cfg: Config,
    writer: W,
    state: EncState,
    _marker: PhantomData<M>,
}

impl<W, M> Writer<W, M>
where
    W: crate::Writer,
    M: 'static,
{
    #[inline]
    pub(crate) fn new(cfg: Config, writer: W) -> Self {
        Self {
            cfg,
            writer,
            state: EncState::new(),
            _marker: PhantomData,
        }
    }

    /// Encode and write a single record.
    ///
    /// This is the same as [`Writer::serialize`] but allows for using a
    /// configurable [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    ///
    /// let cx = musli::context::new();
    /// let mut data = Vec::new();
    /// let mut writer = Encoding::new().with_headers(false).writer(&mut data);
    ///
    /// assert!(writer.serialize_with(&cx, &(1, "a,b")).is_ok());
    /// assert!(writer.serialize_with(&cx, &(2, "c")).is_ok());
    /// assert_eq!(data, b"1,\"a,b\"\n2,c\n");
    /// ```
    #[inline]
    pub fn serialize_with<C, T>(&mut self, cx: C, value: &T) -> Result<(), C::Error>
    where
        C: Context,
        T: ?Sized + Encode<M>,
    {
        cx.clear();

        T::encode(
            value,
            RecordEncoder::<_, _, M, _, _>::new(
                cx,
                self.writer.borrow_mut(),
                self.cfg,
                &mut self.state,
                Some(value),
                Pass::Value,
            ),
        )
    }

    /// Encode and write a single record.
    ///
    /// Unless [`Encoding::with_flexible`] is enabled, every record must have
    /// the same number of fields as the first one. Since records are written
    /// as they are being encoded, a record which raises an error might have
    /// been partially written.
    ///
    /// [`Encoding::with_flexible`]: super::Encoding::with_flexible
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut data = Vec::new();
    /// let mut writer = Encoding::new().writer(&mut data);
    /// writer.serialize(&(1, "a"))?;
    /// writer.serialize(&(2, "b"))?;
    /// assert_eq!(data, b"0,1\n1,a\n2,b\n");
    ///
    /// let mut writer = Encoding::new().writer(&mut data);
    /// writer.serialize(&(1, "a"))?;
    /// assert!(writer.serialize(&(3, "c", true)).is_err());
    /// # Ok::<_, Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn serialize<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Encode<M>,
    {
        let cx = crate::context::new().with_error();
        self.serialize_with(&cx, value)
    }

    /// Access the underlying writer.
    #[inline]
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Access the underlying writer mutably.
    #[inline]
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Unwrap the underlying writer.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Finish writing and return the output of the underlying writer. When
    /// writing to a `&mut [u8]` this is the number of bytes which remain
    /// unused in the slice.
    ///
    /// This is the same as [`Writer::finish`] but allows for using a
    /// configurable [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    ///
    /// let cx = musli::context::new();
    /// let mut buf = [0u8; 64];
    /// let mut writer = Encoding::new().writer(&mut buf[..]);
    ///
    /// assert!(writer.serialize_with(&cx, &(1, 2)).is_ok());
    ///
    /// let Ok(remaining) = writer.finish_with(&cx) else {
    ///     panic!("failed to finish");
    /// };
    ///
    /// assert_eq!(&buf[..buf.len() - remaining], b"0,1\n1,2\n");
    /// ```
    #[inline]
    pub fn finish_with<C>(mut self, cx: C) -> Result<W::Ok, C::Error>
    where
        C: Context,
    {
        self.writer.finish(cx)
    }

    /// Finish writing and return the output of the underlying writer. When
    /// writing to a `&mut [u8]` this is the number of bytes which remain
    /// unused in the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut buf = [0u8; 64];
    /// let mut writer = Encoding::new().writer(&mut buf[..]);
    /// writer.serialize(&(1, 2))?;
    /// let remaining = writer.finish()?;
    ///
    /// assert_eq!(&buf[..buf.len() - remaining], b"0,1\n1,2\n");
    /// # Ok::<_, Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
    #[inline]
    pub fn finish(self) -> Result<W::Ok, Error> {
        let cx = crate::context::new().with_error();
        self.finish_with(&cx)
    }
}
