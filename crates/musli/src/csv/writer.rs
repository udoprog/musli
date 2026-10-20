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
/// let mut writer = Encoding::new().writer(Vec::new());
/// writer.serialize(&City { city: "Boston", population: Some(4628910) })?;
/// writer.serialize(&City { city: "Concord, MA", population: None })?;
///
/// assert_eq!(writer.into_inner(), b"city,population\nBoston,4628910\n\"Concord, MA\",\n");
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
    /// let mut writer = Encoding::new().with_headers(false).writer(Vec::new());
    ///
    /// assert!(writer.serialize_with(&cx, &(1, "a,b")).is_ok());
    /// assert!(writer.serialize_with(&cx, &(2, "c")).is_ok());
    /// assert_eq!(writer.into_inner(), b"1,\"a,b\"\n2,c\n");
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
    /// # Examples
    ///
    /// ```
    /// use musli::csv::Encoding;
    /// # use musli::csv::Error;
    ///
    /// let mut writer = Encoding::new().writer(Vec::new());
    /// writer.serialize(&(1, "a"))?;
    /// writer.serialize(&(2, "b"))?;
    ///
    /// // Every record must have the same number of fields.
    /// assert!(writer.serialize(&(3, "c", true)).is_err());
    ///
    /// assert_eq!(writer.into_inner(), b"0,1\n1,a\n2,b\n");
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
}
