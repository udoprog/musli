//! Helper macros for use with Musli.

macro_rules! bare_encoding {
    ($mode:ident, $default:ident, $what:ident, $reader_trait:ident, $writer_trait:ident) => {
        /// Encode the given value to the given [`Writer`] using the default
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
        /// # Ok::<_, Error>(())
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

        /// Encode the given value to the given slice using the default
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
        /// # Ok::<_, Error>(())
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

        /// Encode the given value to a [`Vec`] using the default [`Encoding`].
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
        /// # Ok::<_, Error>(())
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

        /// Encode the given value to a fixed-size bytes using the default
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
        /// # Ok::<_, Error>(())
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

        /// Encode the given value to the given [`Write`] using the default
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
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "std")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
        #[inline]
        pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
        where
            W: std::io::Write,
            T: ?Sized + Encode<crate::mode::$mode>,
        {
            $default.to_writer(writer, value)
        }

        /// Decode the given type `T` from the given [`Reader`] using the
        /// default [`Encoding`].
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
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn decode<'de, R, T>(reader: R) -> Result<T, Error>
        where
            R: $reader_trait<'de>,
            T: Decode<'de, $mode, Global>,
        {
            $default.decode(reader)
        }

        /// Decode the given type `T` from the given slice using the default
        /// [`Encoding`].
        ///
        /// The whole slice has to be consumed by the value being decoded, and
        /// anything left over is reported as an error. Use [`decode`] with a
        /// [`Reader`] to instead stop at the end of the value, which is what
        /// allows several values to be read out of one buffer.
        ///
        /// [`decode`]: self::decode
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
        #[doc = concat!("let data = ", stringify!($what), "::to_vec(&Person {")]
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, $mode, Global>,
        {
            $default.from_slice(bytes)
        }

        /// Decode the given type `T` from the given slice using the default
        /// [`Encoding`], recording where in the input an error is raised.
        ///
        /// This is the same as [`from_slice`], except that the returned error
        /// includes the path to the value being decoded and the position in
        /// the input, such as `.field[2].name: Expected string (at byte 14)`.
        /// See [`Encoding::from_slice_traced`].
        ///
        /// [`from_slice`]: self::from_slice
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        #[doc = concat!("use musli::", stringify!($what), ";")]
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
        /// // Leave out the last byte.
        #[doc = concat!("if let Err(error) = ", stringify!($what), "::from_slice_traced::<Person>(&data[..data.len() - 1]) {")]
        ///     assert!(error.range().is_some());
        ///     println!("{error}");
        /// }
        #[doc = concat!("# Ok::<_, musli::", stringify!($what), "::Error>(())")]
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_slice_traced<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, $mode, Global>,
        {
            $default.from_slice_traced(bytes)
        }
    };
}

pub(crate) use bare_encoding;

/// Generate all public encoding helpers.
macro_rules! encoding_impls {
    // A format which also builds its decoder from the configuration of the
    // encoding passes the name of a method on the encoding which takes
    // `(self, cx, reader)` and constructs the decoder.
    (
        custom,
        $mode:ident,
        $what:ident,
        self.$decoder_method:ident,
        $reader_trait:ident :: $into_reader:ident,
        $reader_kind:path,
        $writer_trait:ident :: $into_writer:ident $(,)?
    ) => {
        $crate::macros::encoding_impls!(
            @body
            $mode,
            $what,
            [method $decoder_method],
            $reader_trait::$into_reader,
            $reader_kind,
            $writer_trait::$into_writer,
        );
    };

    // A format which builds its encoder differently per configuration defines
    // `encode_into` itself and passes `custom` in place of a constructor.
    (
        custom,
        $mode:ident,
        $what:ident,
        $decoder_new:path,
        $reader_trait:ident :: $into_reader:ident,
        $reader_kind:path,
        $writer_trait:ident :: $into_writer:ident $(,)?
    ) => {
        $crate::macros::encoding_impls!(
            @body
            $mode,
            $what,
            [new $decoder_new],
            $reader_trait::$into_reader,
            $reader_kind,
            $writer_trait::$into_writer,
        );
    };

    (
        $mode:ident,
        $what:ident,
        $encoder_new:path,
        $decoder_new:path,
        $reader_trait:ident :: $into_reader:ident,
        $reader_kind:path,
        $writer_trait:ident :: $into_writer:ident $(,)?
    ) => {
        /// Encode `value` into `writer` using the encoder this encoding is
        /// configured with.
        #[inline]
        fn encode_into<C, W, T>(self, cx: C, writer: W, value: &T) -> Result<(), C::Error>
        where
            C: Context,
            W: $crate::Writer,
            T: ?Sized + Encode<$mode>,
        {
            T::encode(value, $encoder_new(cx, writer))
        }

        $crate::macros::encoding_impls!(
            @body
            $mode,
            $what,
            [new $decoder_new],
            $reader_trait::$into_reader,
            $reader_kind,
            $writer_trait::$into_writer,
        );
    };

    (@new_decoder $this:tt, $cx:ident, $reader:expr, new $decoder_new:path) => {
        $decoder_new($cx, $reader)
    };

    (@new_decoder $this:tt, $cx:ident, $reader:expr, method $decoder_method:ident) => {
        $this.$decoder_method($cx, $reader)
    };

    (
        @body
        $mode:ident,
        $what:ident,
        [$($decoder:tt)*],
        $reader_trait:ident :: $into_reader:ident,
        $reader_kind:path,
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
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn encode<W, T>(self, writer: W, value: &T) -> Result<W::Ok, Error>
        where
            W: $writer_trait,
            T: ?Sized + Encode<$mode>,
        {
            let cx = $crate::context::new().with_error();
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
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_slice<T>(self, out: &mut [u8], value: &T) -> Result<usize, Error>
        where
            T: ?Sized + Encode<$mode>,
        {
            let cx = $crate::context::new().with_error();
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
        /// # Ok::<_, Error>(())
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
        /// # Ok::<_, Error>(())
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
            let cx = $crate::context::new().with_error();
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
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "std")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
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
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn decode<'de, R, T>(self, reader: R) -> Result<T, Error>
        where
            R: $reader_trait<'de>,
            T: Decode<'de, $mode, Global>,
        {
            let cx = $crate::context::new().with_error();
            self.decode_with(&cx, reader)
        }

        /// Decode the given type `T` from the given slice using the current
        /// [`Encoding`].
        ///
        /// The whole slice has to be consumed by the value being decoded, and
        /// anything left over is reported as an error. Use
        /// [`Encoding::decode`] with a [`Reader`] to instead stop at the end of
        /// the value, which is what allows several values to be read out of one
        /// buffer.
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
        /// let data = ENCODING.to_vec(&Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, $mode, Global>,
        {
            let cx = $crate::context::new().with_error();
            self.from_slice_with(&cx, bytes)
        }

        /// Decode the given type `T` from the given string using the current
        /// [`Encoding`].
        ///
        /// The whole string has to be consumed by the value being decoded, and
        /// anything left over is reported as an error.
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_str<'de, T>(self, string: &'de str) -> Result<T, Error>
        where
            T: Decode<'de, M, Global>,
        {
            let cx = $crate::context::new().with_error();
            self.from_str_with(&cx, string)
        }

        /// Decode the given type `T` from the given [`Reader`] using the
        /// current [`Encoding`], recording where in the input an error is
        /// raised.
        ///
        /// This is the same as [`Encoding::decode`], except that the returned
        /// error includes the path to the value being decoded and the position
        /// in the input, such as `.field[2].name: Expected string (at byte 14)`.
        /// These are also available through [`Error::path`] and
        /// [`Error::range`].
        ///
        /// Tracing is not free, so the plain methods leave it out. To configure
        /// the tracing context further, build one with
        /// [`DefaultContext::with_trace`] and [`DefaultContext::with_error`] and
        /// pass it to [`Encoding::decode_with`].
        ///
        /// [`Reader`]: crate::Reader
        /// [`DefaultContext::with_trace`]: crate::context::DefaultContext::with_trace
        /// [`DefaultContext::with_error`]: crate::context::DefaultContext::with_error
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn decode_traced<'de, R, T>(self, reader: R) -> Result<T, Error>
        where
            R: $reader_trait<'de>,
            T: Decode<'de, $mode, Global>,
        {
            let cx = $crate::context::new().with_trace().with_error();
            self.decode_with(&cx, reader)
        }

        /// Decode the given type `T` from the given slice using the current
        /// [`Encoding`], recording where in the input an error is raised.
        ///
        /// This is the same as [`Encoding::from_slice`], except that the
        /// returned error includes the path to the value being decoded and the
        /// position in the input, such as
        /// `.field[2].name: Expected string (at byte 14)`. These are also
        /// available through [`Error::path`] and [`Error::range`].
        ///
        /// Tracing is not free, so the plain methods leave it out. To configure
        /// the tracing context further, build one with
        /// [`DefaultContext::with_trace`] and [`DefaultContext::with_error`] and
        /// pass it to [`Encoding::from_slice_with`].
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
        /// // Leave out the last byte.
        /// if let Err(error) = ENCODING.from_slice_traced::<Person>(&data[..data.len() - 1]) {
        ///     assert!(error.path().is_some());
        ///     assert!(error.range().is_some());
        ///     println!("{error}");
        /// }
        /// # Ok::<_, Error>(())
        /// ```
        ///
        /// [`DefaultContext::with_trace`]: crate::context::DefaultContext::with_trace
        /// [`DefaultContext::with_error`]: crate::context::DefaultContext::with_error
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_slice_traced<'de, T>(self, bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, $mode, Global>,
        {
            let cx = $crate::context::new().with_trace().with_error();
            self.from_slice_with(&cx, bytes)
        }

        /// Decode the given type `T` from the given string using the current
        /// [`Encoding`], recording where in the input an error is raised.
        ///
        /// This is the same as [`Encoding::from_str`], except that the
        /// returned error includes the path to the value being decoded and the
        /// position in the input, such as
        /// `.field[2].name: Expected string (at byte 14)`. These are also
        /// available through [`Error::path`] and [`Error::range`].
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn from_str_traced<'de, T>(self, string: &'de str) -> Result<T, Error>
        where
            T: Decode<'de, M, Global>,
        {
            let cx = $crate::context::new().with_trace().with_error();
            self.from_str_with(&cx, string)
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
        /// use musli::alloc::Global;
        /// use musli::context;
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
        /// let cx = context::new().with_error();
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
        /// # Ok::<_, Error>(())
        /// ```
        #[inline]
        pub fn encode_with<C, W, T>(self, cx: C, writer: W, value: &T) -> Result<W::Ok, C::Error>
        where
            C: Context,
            W: $writer_trait,
            T: ?Sized + Encode<$mode>,
        {
            cx.clear();
            let mut writer = $writer_trait::$into_writer(writer);
            self.encode_into(cx, $crate::writer::Writer::borrow_mut(&mut writer), value)?;
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
        /// use musli::alloc::Global;
        /// use musli::context;
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
        /// let cx = context::new().with_error();
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
        /// # Ok::<_, Error>(())
        /// ```
        #[inline]
        pub fn to_slice_with<C, T>(
            self,
            cx: C,
            out: &mut [u8],
            value: &T,
        ) -> Result<usize, C::Error>
        where
            C: Context,
            T: ?Sized + Encode<$mode>,
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
        /// use musli::alloc::Global;
        /// use musli::context;
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
        /// let cx = context::new().with_error();
        ///
        /// let data = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<_, Error>(())
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
            C: Context,
            T: ?Sized + Encode<$mode>,
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
        /// use musli::alloc::Global;
        /// use musli::context;
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
        /// let cx = context::new().with_error();
        ///
        /// let data: FixedBytes<128> = ENCODING.to_fixed_bytes_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<_, Error>(())
        /// ```
        #[inline]
        pub fn to_fixed_bytes_with<C, const N: usize, T>(
            self,
            cx: C,
            value: &T,
        ) -> Result<$crate::FixedBytes<N>, C::Error>
        where
            C: Context,
            T: ?Sized + Encode<$mode>,
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
        /// use musli::alloc::Global;
        /// use musli::context;
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
        /// let cx = context::new().with_error();
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
        /// # Ok::<_, Error>(())
        /// ```
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer_with<C, W, T>(self, cx: C, write: W, value: &T) -> Result<(), C::Error>
        where
            C: Context,
            W: std::io::Write,
            T: ?Sized + Encode<$mode>,
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
        /// use musli::alloc::Global;
        /// use musli::context;
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
        /// let cx = context::new().with_error();
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
        /// # Ok::<_, Error>(())
        /// ```
        #[inline]
        pub fn decode_with<'de, C, R, T>(self, cx: C, reader: R) -> Result<T, C::Error>
        where
            C: Context,
            R: $reader_trait<'de>,
            T: Decode<'de, $mode, C::Allocator>,
        {
            cx.clear();
            let reader = $reader_trait::$into_reader(reader);
            T::decode($crate::macros::encoding_impls!(@new_decoder self, cx, reader, $($decoder)*))
        }

        /// Decode the given type `T` from the given slice using the current
        /// [`Encoding`].
        ///
        /// The whole slice has to be consumed by the value being decoded, and
        /// anything left over is reported as an error.
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
        /// use musli::alloc::Global;
        /// use musli::context;
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
        /// let cx = context::new().with_error();
        ///
        /// let buf = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &buf[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
        /// # Ok::<_, Error>(())
        /// ```
        #[inline]
        pub fn from_slice_with<'de, C, T>(self, cx: C, bytes: &'de [u8]) -> Result<T, C::Error>
        where
            C: Context,
            T: Decode<'de, $mode, C::Allocator>,
        {
            cx.clear();
            let mut reader = $reader_trait::$into_reader(bytes);
            let value = T::decode($crate::macros::encoding_impls!(
                @new_decoder
                self,
                cx,
                <_ as $reader_kind>::borrow_mut(&mut reader),
                $($decoder)*
            ))?;

            if !<_ as $reader_kind>::is_exhausted(&mut reader, cx) {
                return Err($crate::Context::message(cx, "Trailing input"));
            }

            Ok(value)
        }

        /// Decode the given type `T` from the given string using the current
        /// [`Encoding`].
        ///
        /// This is the same as [`Encoding::from_str`] but allows for using a
        /// configurable [`Context`].
        ///
        /// [`Context`]: crate::Context
        #[inline]
        pub fn from_str_with<'de, C, T>(self, cx: C, string: &'de str) -> Result<T, C::Error>
        where
            C: Context,
            T: Decode<'de, $mode, C::Allocator>,
        {
            cx.clear();
            let mut reader = $reader_trait::$into_reader(string);
            let value = T::decode($crate::macros::encoding_impls!(
                @new_decoder
                self,
                cx,
                <_ as $reader_kind>::borrow_mut(&mut reader),
                $($decoder)*
            ))?;

            if !<_ as $reader_kind>::is_exhausted(&mut reader, cx) {
                return Err($crate::Context::message(cx, "Trailing input"));
            }

            Ok(value)
        }
    };
}

macro_rules! implement_error {
    (
        $(#[$($meta:meta)*])*
        $vis:vis struct $id:ident;
    ) => {
        $(#[$($meta)*])*
        #[derive(PartialEq, Eq)]
        #[cfg(feature = "alloc")]
        pub struct $id<A = $crate::alloc::Global>
        where
            A: $crate::Allocator,
        {
            err: Impl<A>,
        }

        $(#[$($meta)*])*
        #[derive(PartialEq, Eq)]
        #[cfg(not(feature = "alloc"))]
        pub struct $id<A>
        where
            A: $crate::Allocator,
        {
            err: Impl<A>,
        }

        impl<A> $id<A>
        where
            A: $crate::Allocator,
        {
            /// The path to the value which was being decoded when the error
            /// was raised, such as `.field[2].name`.
            ///
            /// This is only recorded if the error was produced through a
            /// context which traces, such as the one used by the `*_traced`
            /// methods on `Encoding` or one built with
            /// [`DefaultContext::with_trace`]. The path is empty if the error
            /// was raised outside of any field.
            ///
            /// [`DefaultContext::with_trace`]: crate::context::DefaultContext::with_trace
            #[inline]
            pub fn path(&self) -> Option<&str> {
                match &self.err {
                    Impl::Traced(Some(traced)) => Some(traced.path()),
                    _ => None,
                }
            }

            /// The range of bytes in the input the error was raised at.
            ///
            /// Like [`path`], this is only recorded if the error was produced
            /// through a context which traces. The range is empty if the error
            /// refers to a single position.
            ///
            #[doc = concat!("[`path`]: ", stringify!($id), "::path")]
            #[inline]
            pub fn range(&self) -> Option<core::ops::Range<usize>> {
                match &self.err {
                    Impl::Traced(Some(traced)) => Some(traced.range.clone()),
                    _ => None,
                }
            }
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

        #[derive(PartialEq, Eq)]
        enum Impl<A>
        where
            A: $crate::Allocator,
        {
            Message($crate::alloc::String<A>),
            // A traced error, or `None` if allocating the error failed.
            //
            // This is boxed and doubles as the allocation error so that the
            // error stays as small as a string, since it is returned through
            // every level of decoding.
            Traced(Option<$crate::alloc::Box<Traced<A>, A>>),
        }

        /// An error decorated with the location it was raised at.
        #[derive(PartialEq, Eq)]
        struct Traced<A>
        where
            A: $crate::Allocator,
        {
            /// The path followed by the message, split at `split`.
            text: $crate::alloc::String<A>,
            split: usize,
            range: core::ops::Range<usize>,
        }

        impl<A> Traced<A>
        where
            A: $crate::Allocator,
        {
            #[inline]
            fn path(&self) -> &str {
                &self.text.as_str()[..self.split]
            }

            #[inline]
            fn message(&self) -> &str {
                &self.text.as_str()[self.split..]
            }
        }

        impl<A> core::fmt::Display for Impl<A>
        where
            A: $crate::Allocator,
        {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Impl::Message(message) => message.fmt(f),
                    Impl::Traced(Some(traced)) => {
                        let path = traced.path();

                        if !path.is_empty() {
                            write!(f, "{path}: ")?;
                        }

                        traced.message().fmt(f)?;

                        let range = &traced.range;

                        if range.start != 0 || range.end != 0 {
                            if range.start == range.end {
                                write!(f, " (at byte {})", range.start)?;
                            } else {
                                write!(f, " (at bytes {}-{})", range.start, range.end)?;
                            }
                        }

                        Ok(())
                    }
                    Impl::Traced(None) => $crate::alloc::AllocError.fmt(f),
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
                    Impl::Traced(Some(traced)) => f
                        .debug_struct("Traced")
                        .field("path", &traced.path())
                        .field("message", &traced.message())
                        .field("range", &traced.range)
                        .finish(),
                    Impl::Traced(None) => {
                        f.debug_tuple("Alloc").field(&$crate::alloc::AllocError).finish()
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
                use core::fmt::Write;

                let mut s = $crate::alloc::String::new_in(alloc);

                let err = if core::write!(s, "{message}").is_ok() {
                    Impl::Message(s)
                } else {
                    Impl::Traced(None)
                };

                Self { err }
            }

            #[inline]
            fn traced_message<T>(
                alloc: A,
                message: T,
                location: &$crate::context::Location<'_, A>,
            ) -> Self
            where
                T: core::fmt::Display,
            {
                use core::fmt::Write;

                let mut text = $crate::alloc::String::new_in(alloc);

                if core::write!(text, "{}", location.path()).is_err() {
                    return Self::message(alloc, location.with_message(message));
                }

                let split = text.len();

                if core::write!(text, "{message}").is_err() {
                    return Self {
                        err: Impl::Traced(None),
                    };
                }

                let traced = Traced {
                    text,
                    split,
                    range: location.range(),
                };

                let err = match $crate::alloc::Box::new_in(traced, alloc) {
                    Ok(traced) => Impl::Traced(Some(traced)),
                    Err(..) => return Self::message(alloc, location.with_message(message)),
                };

                Self { err }
            }

            #[inline]
            fn traced_custom<T>(
                alloc: A,
                error: T,
                location: &$crate::context::Location<'_, A>,
            ) -> Self
            where
                T: 'static + Send + Sync + core::error::Error,
            {
                Self::traced_message(alloc, error, location)
            }
        }

        #[cfg(feature = "alloc")]
        const _: () = {
            const fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<$id<$crate::alloc::Global>>();
        };
    };
}

pub(crate) use encoding_impls;
pub(crate) use implement_error;
