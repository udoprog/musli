//! Helper macros for use with Musli.

macro_rules! bare_encoding {
    ($mode:ident, $default:ident, $what:ident, $reader_trait:ident, $writer_trait:ident) => {
        /// Encode the given value to the given [`Writer`] using the [`DEFAULT`]
        /// [`Encoding`].
        ///
        /// [`Writer`]: crate::Writer
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), ";")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let mut data = Vec::new();
        ///
        #[doc = concat!(stringify!($what), "::encode(&mut data, &Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn encode<W, T>(writer: W, value: &T) -> Result<W::Ok, Error>
        where
            W: $writer_trait,
            T: ?Sized + Encode<crate::mode::$mode>,
        {
            $default.encode(writer, value)
        }

        /// Encode the given value to the given slice using the [`DEFAULT`]
        /// [`Encoding`] and return the number of bytes encoded.
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), ";")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let mut data = Vec::new();
        /// data.resize(128, 0);
        ///
        #[doc = concat!("let w = ", stringify!($what), "::to_slice(&mut data[..], &Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// assert!(w > 0);
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..w])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_slice<T>(out: &mut [u8], value: &T) -> Result<usize, Error>
        where
            T: ?Sized + Encode<crate::mode::$mode>,
        {
            $default.to_slice(out, value)
        }

        /// Encode the given value to a [`Vec`] using the [`DEFAULT`]
        /// [`Encoding`].
        ///
        /// [`Vec`]: rust_alloc::vec::Vec
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), ";")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        #[doc = concat!("let data = ", stringify!($what), "::to_vec(&Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_vec<T>(value: &T) -> Result<rust_alloc::vec::Vec<u8>, Error>
        where
            T: ?Sized + Encode<crate::mode::$mode>,
        {
            $default.to_vec(value)
        }

        /// Encode the given value to a fixed-size bytes using the [`DEFAULT`]
        /// [`Encoding`].
        ///
        /// ```
        /// use musli::{Decode, Encode, FixedBytes};
        #[doc = concat!("use musli::", stringify!($what), ";")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        #[doc = concat!("let data: FixedBytes<128> = ", stringify!($what), "::to_fixed_bytes(&Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<$crate::FixedBytes<N>, Error>
        where
            T: ?Sized + Encode<crate::mode::$mode>,
        {
            $default.to_fixed_bytes::<N, _>(value)
        }

        /// Encode the given value to the given [`Write`] using the [`DEFAULT`]
        /// [`Encoding`].
        ///
        /// [`Write`]: std::io::Write
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), ";")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let mut data = Vec::new();
        ///
        #[doc = concat!(stringify!($what), "::to_writer(&mut data, &Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(all(feature = "std", feature = "alloc"))]
        #[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", feature = "alloc"))))]
        #[inline]
        pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
        where
            W: std::io::Write,
            T: ?Sized + Encode<crate::mode::$mode>,
        {
            $default.to_writer(writer, value)
        }

        /// Decode the given type `T` from the given [`Reader`] using the [`DEFAULT`]
        /// [`Encoding`].
        ///
        /// [`Reader`]: crate::Reader
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), ";")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        #[doc = concat!("let mut data = ", stringify!($what), "::to_vec(&Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// // Add some extra data which will be ignored during decoding.
        /// data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        ///
        /// // Note: A slice implements `musli::Reader`.
        /// let mut slice = &data[..];
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::decode(&mut slice)?;")]
        /// assert_eq!(slice, &[0xde, 0xad, 0xbe, 0xef]);
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn decode<'de, R, T>(reader: R) -> Result<T, Error>
        where
            R: $reader_trait<'de>,
            T: Decode<'de, $mode, System>,
        {
            $default.decode(reader)
        }

        /// Decode the given type `T` from the given slice using the [`DEFAULT`]
        /// [`Encoding`].
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), ";")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        #[doc = concat!("let data = ", stringify!($what), "::to_vec(&Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, $mode, System>,
        {
            $default.from_slice(bytes)
        }
    };
}

pub(crate) use bare_encoding;

/// Generate all public encoding helpers.
macro_rules! encoding_impls {
    (
        $mode:ident,
        $what:ident,
        $encoder_new:path,
        $decoder_new:path,
        $reader_trait:ident :: $into_reader:ident,
        $writer_trait:ident :: $into_writer:ident $(,)?
    ) => {
        /// Encode the given value to the given [`Writer`] using the current
        /// [`Encoding`].
        ///
        /// [`Writer`]: crate::Writer
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let mut data = Vec::new();
        ///
        /// ENCODING.encode(&mut data, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn encode<W, T>(self, writer: W, value: &T) -> Result<W::Ok, Error>
        where
            W: $writer_trait,
            T: ?Sized + Encode<$mode>,
        {
            let cx = $crate::context::Same::with_alloc(System::new());
            self.encode_with(&cx, writer, value)
        }

        /// Encode the given value to the given slice using the current
        /// [`Encoding`] and return the number of bytes encoded.
        ///
        /// [`Writer`]: crate::Writer
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let mut data = Vec::new();
        /// data.resize(128, 0);
        ///
        /// let w = ENCODING.to_slice(&mut data, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// assert!(w > 0);
        ///
        /// let person: Person = ENCODING.from_slice(&data[..w])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_slice<T>(self, out: &mut [u8], value: &T) -> Result<usize, Error>
        where
            T: ?Sized + Encode<$mode>,
        {
            let cx = $crate::context::Same::with_alloc(System::new());
            self.to_slice_with(&cx, out, value)
        }

        /// Encode the given value to a [`Vec`] using the current [`Encoding`].
        ///
        /// [`Vec`]: rust_alloc::vec::Vec
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let data = ENCODING.to_vec(&Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_vec<T>(self, value: &T) -> Result<rust_alloc::vec::Vec<u8>, Error>
        where
            T: ?Sized + Encode<$mode>,
        {
            let mut vec = rust_alloc::vec::Vec::new();
            self.encode(&mut vec, value)?;
            Ok(vec)
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// [`Encoding`].
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode, FixedBytes};
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let data: FixedBytes<128> = ENCODING.to_fixed_bytes(&Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_fixed_bytes<const N: usize, T>(
            self,
            value: &T,
        ) -> Result<$crate::FixedBytes<N>, Error>
        where
            T: ?Sized + Encode<$mode>,
        {
            let cx = $crate::context::Same::with_alloc(System::new());
            self.to_fixed_bytes_with(&cx, value)
        }

        /// Encode the given value to the given [`Write`] using the current
        /// [`Encoding`].
        ///
        /// [`Write`]: std::io::Write
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let mut data = Vec::new();
        ///
        /// ENCODING.to_writer(&mut data, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(all(feature = "std", feature = "alloc"))]
        #[cfg_attr(doc_cfg, doc(cfg(all(feature = "std", feature = "alloc"))))]
        #[inline]
        pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), Error>
        where
            W: std::io::Write,
            T: ?Sized + Encode<$mode>,
        {
            let writer = $crate::wrap::wrap(write);
            self.encode(writer, value)
        }

        /// Decode the given type `T` from the given [`Reader`] using the
        /// current [`Encoding`].
        ///
        /// [`Reader`]: crate::Reader
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let mut data = ENCODING.to_vec(&Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// // Add some extra data which will be ignored during decoding.
        /// data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        ///
        /// // Note: A slice implements `musli::Reader`.
        /// let mut slice = &data[..];
        /// let person: Person = ENCODING.decode(&mut slice)?;
        ///
        /// assert_eq!(slice, &[0xde, 0xad, 0xbe, 0xef]);
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn decode<'de, R, T>(self, reader: R) -> Result<T, Error>
        where
            R: $reader_trait<'de>,
            T: Decode<'de, $mode, System>,
        {
            let cx = $crate::context::Same::with_alloc(System::new());
            self.decode_with(&cx, reader)
        }

        /// Decode the given type `T` from the given slice using the current
        /// [`Encoding`].
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let data = ENCODING.to_vec(&Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, $mode, System>,
        {
            let cx = $crate::context::Same::with_alloc(System::new());
            self.from_slice_with(&cx, bytes)
        }

        /// Decode the given type `T` from the given string using the current
        /// [`Encoding`].
        ///
        /// This is an alias over [`Encoding::from_slice`] for convenience. See
        /// its documentation for more.
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_str<'de, T>(self, string: &'de str) -> Result<T, Error>
        where
            T: Decode<'de, M, System>,
        {
            self.from_slice(string.as_bytes())
        }

        /// Encode the given value to the given [`Writer`] using the current
        /// [`Encoding`].
        ///
        /// This is the same as [`Encoding::encode`] but allows for using a
        /// configurable [`Context`].
        ///
        /// [`Writer`]: crate::Writer
        /// [`Context`]: crate::Context
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        /// use musli::alloc::System;
        /// use musli::context::Same;
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let cx = Same::new();
        ///
        /// let mut data = Vec::new();
        ///
        /// ENCODING.encode_with(&cx, &mut data, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn encode_with<C, W, T>(self, cx: C, writer: W, value: &T) -> Result<W::Ok, C::Error>
        where
            C: Context<Mode = $mode>,
            W: $writer_trait,
            T: ?Sized + Encode<C::Mode>,
        {
            cx.clear();
            let mut writer = $writer_trait::$into_writer(writer);
            let encoder = $encoder_new(cx, $crate::writer::Writer::borrow_mut(&mut writer));
            T::encode(value, encoder)?;
            $crate::writer::Writer::finish(&mut writer, cx)
        }

        /// Encode the given value to the given slice using the current
        /// [`Encoding`] and return the number of bytes encoded.
        ///
        /// This is the same as [`Encoding::to_slice`] but allows for using a
        /// configurable [`Context`].
        ///
        /// [`Context`]: crate::Context
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        /// use musli::alloc::System;
        /// use musli::context::Same;
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let cx = Same::new();
        ///
        /// let mut data = Vec::new();
        /// data.resize(128, 0);
        ///
        /// let w = ENCODING.to_slice_with(&cx, &mut data[..], &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// assert!(w > 0);
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..w])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn to_slice_with<C, T>(
            self,
            cx: C,
            out: &mut [u8],
            value: &T,
        ) -> Result<usize, C::Error>
        where
            C: Context<Mode = $mode>,
            T: ?Sized + Encode<C::Mode>,
        {
            let len = out.len();
            let remaining = self.encode_with(cx, out, value)?;
            Ok(len - remaining)
        }

        /// Encode the given value to a [`Vec`] using the current [`Encoding`].
        ///
        /// This is the same as [`Encoding::to_vec`], but allows for using a
        /// configurable [`Context`].
        ///
        /// [`Context`]: crate::Context
        /// [`Vec`]: rust_alloc::vec::Vec
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        /// use musli::alloc::System;
        /// use musli::context::Same;
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let cx = Same::new();
        ///
        /// let data = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_vec_with<C, T>(
            self,
            cx: C,
            value: &T,
        ) -> Result<rust_alloc::vec::Vec<u8>, C::Error>
        where
            C: Context<Mode = $mode>,
            T: ?Sized + Encode<C::Mode>,
        {
            let mut vec = rust_alloc::vec::Vec::new();
            self.encode_with(cx, &mut vec, value)?;
            Ok(vec)
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// [`Encoding`].
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode, FixedBytes};
        /// use musli::alloc::System;
        /// use musli::context::Same;
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let cx = Same::new();
        ///
        /// let data: FixedBytes<128> = ENCODING.to_fixed_bytes_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn to_fixed_bytes_with<C, const N: usize, T>(
            self,
            cx: C,
            value: &T,
        ) -> Result<$crate::FixedBytes<N>, C::Error>
        where
            C: Context<Mode = $mode>,
            T: ?Sized + Encode<C::Mode>,
        {
            let mut bytes = $crate::FixedBytes::new();
            self.encode_with(cx, &mut bytes, value)?;
            Ok(bytes)
        }

        /// Encode the given value to the given [`Write`] using the current
        /// [`Encoding`] and context `C`.
        ///
        /// [`Write`]: std::io::Write
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        /// use musli::alloc::System;
        /// use musli::context::Same;
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let cx = Same::new();
        ///
        /// let mut data = Vec::new();
        ///
        /// ENCODING.to_writer_with(&cx, &mut data, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer_with<C, W, T>(self, cx: C, write: W, value: &T) -> Result<(), C::Error>
        where
            C: Context<Mode = $mode>,
            W: std::io::Write,
            T: ?Sized + Encode<C::Mode>,
        {
            let writer = $crate::wrap::wrap(write);
            self.encode_with(cx, writer, value)
        }

        /// Decode the given type `T` from the given [`Reader`] using the
        /// current [`Encoding`].
        ///
        /// This is the same as [`Encoding::decode`] but allows for using a
        /// configurable [`Context`].
        ///
        /// [`Reader`]: crate::Reader
        /// [`Context`]: crate::Context
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        /// use musli::alloc::System;
        /// use musli::context::Same;
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let cx = Same::new();
        ///
        /// let buf = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let mut slice = &buf[..];
        /// let person: Person = ENCODING.decode_with(&cx, &mut slice)?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn decode_with<'de, C, R, T>(self, cx: C, reader: R) -> Result<T, C::Error>
        where
            C: Context<Mode = $mode>,
            R: $reader_trait<'de>,
            T: Decode<'de, C::Mode, C::Allocator>,
        {
            cx.clear();
            let reader = $reader_trait::$into_reader(reader);
            T::decode($decoder_new(cx, reader))
        }

        /// Decode the given type `T` from the given slice using the current
        /// [`Encoding`].
        ///
        /// This is the same as [`Encoding::from_slice`], but allows for using a
        /// configurable [`Context`].
        ///
        /// [`Context`]: crate::Context
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        /// use musli::alloc::System;
        /// use musli::context::Same;
        #[doc = concat!("use musli::", stringify!($what), "::Encoding;")]
        #[doc = concat!("# use musli::", stringify!($what), "::Error;")]
        ///
        /// const ENCODING: Encoding = Encoding::new();
        ///
        /// #[derive(Decode, Encode)]
        /// struct Person {
        ///     name: String,
        ///     age: u32,
        /// }
        ///
        /// let cx = Same::new();
        ///
        /// let buf = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &buf[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn from_slice_with<'de, C, T>(self, cx: C, bytes: &'de [u8]) -> Result<T, C::Error>
        where
            C: Context<Mode = $mode>,
            T: Decode<'de, $mode, C::Allocator>,
        {
            self.decode_with(cx, bytes)
        }

        /// Decode the given type `T` from the given string using the current
        /// [`Encoding`].
        ///
        /// This is the same as [`Encoding::from_str`] but allows for using a
        /// configurable [`Context`].
        ///
        /// This is an alias over [`Encoding::from_slice_with`] for convenience.
        /// See its documentation for more.
        ///
        /// [`Context`]: crate::Context
        #[inline]
        pub fn from_str_with<'de, C, T>(self, cx: C, string: &'de str) -> Result<T, C::Error>
        where
            C: Context<Mode = M>,
            T: Decode<'de, M, C::Allocator>,
        {
            self.from_slice_with(cx, string.as_bytes())
        }
    };
}

macro_rules! implement_error {
    (
        $(#[$($meta:meta)*])*
        $vis:vis struct $id:ident;
    ) => {
        $(#[$($meta)*])*
        #[cfg(feature = "alloc")]
        pub struct $id<A = $crate::alloc::System>
        where
            A: $crate::Allocator,
        {
            err: Impl<A>,
        }

        $(#[$($meta)*])*
        #[cfg(not(feature = "alloc"))]
        pub struct $id<A>
        where
            A: $crate::Allocator,
        {
            err: Impl<A>,
        }

        impl<A> core::fmt::Display for $id<A>
        where
            A: $crate::Allocator,
        {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.err.fmt(f)
            }
        }

        impl<A> core::fmt::Debug for $id<A>
        where
            A: $crate::Allocator,
        {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.err.fmt(f)
            }
        }

        enum Impl<A>
        where
            A: $crate::Allocator,
        {
            Message(crate::alloc::String<A>),
            Alloc(crate::alloc::AllocError),
        }

        impl<A> core::fmt::Display for Impl<A>
        where
            A: $crate::Allocator,
        {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Impl::Message(message) => message.fmt(f),
                    Impl::Alloc(error) => error.fmt(f),
                }
            }
        }

        impl<A> core::fmt::Debug for Impl<A>
        where
            A: $crate::Allocator,
        {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Impl::Message(message) => {
                        f.debug_tuple("Message").field(message).finish()
                    }
                    Impl::Alloc(error) => {
                        f.debug_tuple("Alloc").field(error).finish()
                    }
                }
            }
        }

        impl<A> core::error::Error for $id<A>
        where
            A: $crate::Allocator
        {
        }

        impl<A> $crate::context::ContextError<A> for $id<A>
        where
            A: $crate::Allocator,
        {
            #[inline]
            fn custom<T>(alloc: A, error: T) -> Self
            where
                T: core::fmt::Display,
            {
                Self::message(alloc, error)
            }

            #[inline]
            fn message<T>(alloc: A, message: T) -> Self
            where
                T: core::fmt::Display,
            {
                let err = match crate::alloc::collect_string(alloc, &message) {
                    Ok(message) => Impl::Message(message),
                    Err(error) => Impl::Alloc(error),
                };

                Self { err }
            }
        }

        #[cfg(feature = "alloc")]
        const _: () = {
            const fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<$id<$crate::alloc::System>>();
        };
    };
}

pub(crate) use encoding_impls;
pub(crate) use implement_error;
