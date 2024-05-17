//! Helper macros for use with Musli.

macro_rules! bare_encoding {
    ($mode:ident, $default:ident, $what:ident, $reader_trait:ident) => {
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
        ///     age: 62,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn encode<W, T>(writer: W, value: &T) -> Result<(), Error>
        where
            W: $crate::Writer,
            T: ?Sized + $crate::Encode<crate::mode::$mode>,
        {
            $default.encode(writer, value)
        }

        /// Encode the given value to a [`Vec`] using the [`DEFAULT`]
        /// [`Encoding`].
        ///
        /// [`Vec`]: alloc::vec::Vec
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
        ///     age: 62,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_vec<T>(value: &T) -> Result<alloc::vec::Vec<u8>, Error>
        where
            T: ?Sized + $crate::Encode<crate::mode::$mode>,
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
        ///     age: 62,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn to_fixed_bytes<const N: usize, T>(value: &T) -> Result<$crate::FixedBytes<N>, Error>
        where
            T: ?Sized + $crate::Encode<crate::mode::$mode>,
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
        ///     age: 62,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "std")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
        #[inline]
        pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), Error>
        where
            W: std::io::Write,
            T: ?Sized + $crate::Encode<crate::mode::$mode>,
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
        ///     age: 62,
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
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn decode<'de, R, T>(reader: R) -> Result<T, Error>
        where
            R: $reader_trait<'de>,
            T: $crate::Decode<'de, $mode>,
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
        ///     age: 62,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
        where
            T: $crate::Decode<'de, $mode>,
        {
            $default.from_slice(bytes)
        }
    };
}

pub(crate) use bare_encoding;

/// Generate all public encoding helpers.
macro_rules! encoding_impls {
    ($mode:ident, $what:ident, $encoder_new:path, $decoder_new:path, $reader_trait:ident :: $into_reader:ident $(,)?) => {
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
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn encode<W, T>(self, writer: W, value: &T) -> Result<(), Error>
        where
            W: $crate::Writer,
            T: ?Sized + $crate::Encode<$mode>,
        {
            $crate::allocator::default!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.encode_with(&cx, writer, value)
            })
        }

        /// Encode the given value to a [`Vec`] using the current [`Encoding`].
        ///
        /// [`Vec`]: alloc::vec::Vec
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
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_vec<T>(self, value: &T) -> Result<alloc::vec::Vec<u8>, Error>
        where
            T: ?Sized + $crate::Encode<$mode>,
        {
            let mut vec = alloc::vec::Vec::new();
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
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn to_fixed_bytes<const N: usize, T>(
            self,
            value: &T,
        ) -> Result<$crate::FixedBytes<N>, Error>
        where
            T: ?Sized + $crate::Encode<$mode>,
        {
            $crate::allocator::default!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.to_fixed_bytes_with(&cx, value)
            })
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
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), Error>
        where
            W: std::io::Write,
            T: ?Sized + $crate::Encode<$mode>,
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
        ///     age: 62,
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
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn decode<'de, R, T>(self, reader: R) -> Result<T, Error>
        where
            R: $reader_trait<'de>,
            T: $crate::Decode<'de, $mode>,
        {
            $crate::allocator::default!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.decode_with(&cx, reader)
            })
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
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, Error>
        where
            T: $crate::Decode<'de, $mode>,
        {
            $crate::allocator::default!(|alloc| {
                let cx = $crate::context::Same::new(alloc);
                self.from_slice_with(&cx, bytes)
            })
        }

        /// Decode the given type `T` from the given string using the current
        /// [`Encoding`].
        ///
        /// This is an alias over [`Encoding::from_slice`] for convenience. See
        /// its documentation for more.
        #[inline]
        pub fn from_str<'de, T>(self, string: &'de str) -> Result<T, Error>
        where
            T: $crate::Decode<'de, M>,
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
        /// use musli::allocator::System;
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
        /// let alloc = System::new();
        /// let cx = Same::new(&alloc);
        ///
        /// let mut data = Vec::new();
        ///
        /// ENCODING.encode_with(&cx, &mut data, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn encode_with<C, W, T>(self, cx: &C, writer: W, value: &T) -> Result<(), C::Error>
        where
            C: ?Sized + $crate::Context<Mode = $mode>,
            W: $crate::Writer,
            T: ?Sized + $crate::Encode<C::Mode>,
        {
            cx.clear();
            T::encode(value, cx, $encoder_new(cx, writer))
        }

        /// Encode the given value to a [`Vec`] using the current [`Encoding`].
        ///
        /// This is the same as [`Encoding::to_vec`], but allows for using a
        /// configurable [`Context`].
        ///
        /// [`Context`]: crate::Context
        /// [`Vec`]: alloc::vec::Vec
        ///
        /// # Examples
        ///
        /// ```
        /// use musli::{Decode, Encode};
        /// use musli::allocator::System;
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
        /// let alloc = System::new();
        /// let cx = Same::new(&alloc);
        ///
        /// let data = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "alloc")]
        #[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
        #[inline]
        pub fn to_vec_with<C, T>(self, cx: &C, value: &T) -> Result<alloc::vec::Vec<u8>, C::Error>
        where
            C: ?Sized + $crate::Context<Mode = $mode>,
            T: ?Sized + $crate::Encode<C::Mode>,
        {
            let mut vec = alloc::vec::Vec::new();
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
        /// use musli::allocator::System;
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
        /// let alloc = System::new();
        /// let cx = Same::new(&alloc);
        ///
        /// let data: FixedBytes<128> = ENCODING.to_fixed_bytes_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn to_fixed_bytes_with<C, const N: usize, T>(
            self,
            cx: &C,
            value: &T,
        ) -> Result<$crate::FixedBytes<N>, C::Error>
        where
            C: ?Sized + $crate::Context<Mode = $mode>,
            T: ?Sized + $crate::Encode<C::Mode>,
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
        /// use musli::allocator::System;
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
        /// let alloc = System::new();
        /// let cx = Same::new(&alloc);
        ///
        /// let mut data = Vec::new();
        ///
        /// ENCODING.to_writer_with(&cx, &mut data, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer_with<C, W, T>(self, cx: &C, write: W, value: &T) -> Result<(), C::Error>
        where
            C: ?Sized + $crate::Context<Mode = $mode>,
            W: std::io::Write,
            T: ?Sized + $crate::Encode<C::Mode>,
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
        /// use musli::allocator::System;
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
        /// let alloc = System::new();
        /// let cx = Same::new(&alloc);
        ///
        /// let buf = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 62,
        /// })?;
        ///
        /// let mut slice = &buf[..];
        /// let person: Person = ENCODING.decode_with(&cx, &mut slice)?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn decode_with<'de, C, R, T>(self, cx: &C, reader: R) -> Result<T, C::Error>
        where
            C: ?Sized + $crate::Context<Mode = $mode>,
            R: $reader_trait<'de>,
            T: $crate::Decode<'de, C::Mode>,
        {
            cx.clear();
            let reader = $reader_trait::$into_reader(reader);
            T::decode(cx, $decoder_new(cx, reader))
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
        /// use musli::allocator::System;
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
        /// let alloc = System::new();
        /// let cx = Same::new(&alloc);
        ///
        /// let buf = ENCODING.to_vec_with(&cx, &Person {
        ///     name: "Aristotle".to_string(),
        ///     age: 62,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &buf[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 62);
        /// # Ok::<(), Error>(())
        /// ```
        #[inline]
        pub fn from_slice_with<'de, C, T>(self, cx: &C, bytes: &'de [u8]) -> Result<T, C::Error>
        where
            C: ?Sized + $crate::Context<Mode = $mode>,
            T: $crate::Decode<'de, $mode>,
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
        pub fn from_str_with<'de, C, T>(self, cx: &C, string: &'de str) -> Result<T, C::Error>
        where
            C: ?Sized + $crate::Context<Mode = M>,
            T: $crate::Decode<'de, M>,
        {
            self.from_slice_with(cx, string.as_bytes())
        }
    };
}

pub(crate) use encoding_impls;

macro_rules! test_include_if {
    (#[musli_value] => $($rest:tt)*) => { $($rest)* };
    (=> $($_:tt)*) => {};
}

pub(crate) use test_include_if;

/// Generate test functions which provides rich diagnostics when they fail.
macro_rules! test_fns {
    ($mode:ident, $what:expr $(, $(#[$option:ident])*)?) => {
        /// Roundtrip encode the given value.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "test")]
        pub fn rt<T>(value: T) -> T
        where
            T: $crate::en::Encode<crate::mode::$mode> + $crate::de::DecodeOwned<crate::mode::$mode>,
            T: ::core::fmt::Debug + ::core::cmp::PartialEq,
        {
            const WHAT: &str = $what;
            const ENCODING: super::Encoding = super::Encoding::new();

            use ::core::any::type_name;

            struct FormatBytes<'a>(&'a [u8]);

            impl ::core::fmt::Display for FormatBytes<'_> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "b\"")?;

                    for b in self.0 {
                        if b.is_ascii_graphic() {
                            write!(f, "{}", *b as char)?;
                        } else {
                            write!(f, "\\x{b:02x}")?;
                        }
                    }

                    write!(f, "\" (0-{})", self.0.len())?;
                    Ok(())
                }
            }

            $crate::allocator::default!(|alloc| {
                let mut cx = $crate::context::SystemContext::new(&alloc);
                cx.include_type();

                let out = match ENCODING.to_vec_with(&cx, &value) {
                    Ok(out) => out,
                    Err(..) => {
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                    }
                };

                let decoded: T = match ENCODING.from_slice_with(&cx, out.as_slice()) {
                    Ok(decoded) => decoded,
                    Err(..) => {
                        let out = FormatBytes(&out);
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to decode:\nValue: {value:?}\nBytes: {out}\n{error}", type_name::<T>())
                    }
                };

                assert_eq!(decoded, value, "{WHAT}: {}: roundtrip does not match\nValue: {value:?}", type_name::<T>());

                $crate::macros::test_include_if! {
                    $($(#[$option])*)* =>
                    let value_decode: $crate::value::Value = match ENCODING.from_slice_with(&cx, out.as_slice()) {
                        Ok(decoded) => decoded,
                        Err(..) => {
                            let out = FormatBytes(&out);
                            let error = cx.report();
                            panic!("{WHAT}: {}: failed to decode to value type:\nValue: {value:?}\nBytes:{out}\n{error}", type_name::<T>())
                        }
                    };

                    let value_decoded: T = match $crate::value::decode_with(&cx, &value_decode) {
                        Ok(decoded) => decoded,
                        Err(..) => {
                            let out = FormatBytes(&out);
                            let error = cx.report();
                            panic!("{WHAT}: {}: failed to decode from value type:\nValue: {value:?}\nBytes: {out}\nBuffered value: {value_decode:?}\n{error}", type_name::<T>())
                        }
                    };

                    assert_eq!(value_decoded, value, "{WHAT}: {}: musli-value roundtrip does not match\nValue: {value:?}", type_name::<T>());
                }

                decoded
            })
        }

        /// Encode and then decode the given value once.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "test")]
        pub fn decode<'de, T, U>(value: T, out: &'de mut ::alloc::vec::Vec<u8>, expected: &U) -> U
        where
            T: $crate::en::Encode<crate::mode::$mode>,
            T: ::core::fmt::Debug + ::core::cmp::PartialEq,
            U: $crate::de::Decode<'de, crate::mode::$mode>,
            U: ::core::fmt::Debug + ::core::cmp::PartialEq,
        {
            const WHAT: &str = $what;
            const ENCODING: super::Encoding = super::Encoding::new();

            use ::core::any::type_name;

            struct FormatBytes<'a>(&'a [u8]);

            impl ::core::fmt::Display for FormatBytes<'_> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "b\"")?;

                    for b in self.0 {
                        if b.is_ascii_graphic() {
                            write!(f, "{}", *b as char)?;
                        } else {
                            write!(f, "\\x{b:02x}")?;
                        }
                    }

                    write!(f, "\" (0-{})", self.0.len())?;
                    Ok(())
                }
            }

            $crate::allocator::default!(|alloc| {
                let mut cx = $crate::context::SystemContext::new(&alloc);
                cx.include_type();

                out.clear();

                match ENCODING.to_writer_with(&cx, &mut *out, &value) {
                    Ok(()) => (),
                    Err(..) => {
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                    }
                };

                let actual = match ENCODING.from_slice_with(&cx, &*out) {
                    Ok(decoded) => decoded,
                    Err(..) => {
                        let out = FormatBytes(&*out);
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to decode:\nValue: {value:?}\nBytes: {out}\n{error}", type_name::<U>())
                    }
                };

                assert_eq!(
                    actual,
                    *expected,
                    "{WHAT}: decoded value does not match expected\nBytes: {}",
                    FormatBytes(&*out),
                );

                actual
            })
        }

        /// Encode a value to bytes.
        #[doc(hidden)]
        #[track_caller]
        #[cfg(feature = "test")]
        pub fn to_vec<T>(value: T) -> ::alloc::vec::Vec<u8>
        where
            T: $crate::en::Encode<crate::mode::$mode>,
        {
            const WHAT: &str = $what;
            const ENCODING: super::Encoding = super::Encoding::new();

            use ::core::any::type_name;

            $crate::allocator::default!(|alloc| {
                let mut cx = $crate::context::SystemContext::new(alloc);
                cx.include_type();

                match ENCODING.to_vec_with(&cx, &value) {
                    Ok(out) => out,
                    Err(..) => {
                        let error = cx.report();
                        panic!("{WHAT}: {}: failed to encode:\n{error}", type_name::<T>())
                    }
                }
            })
        }
    }
}

pub(crate) use test_fns;

#[cfg(feature = "test")]
#[macro_export]
#[doc(hidden)]
macro_rules! __assert_roundtrip_eq {
    ($support:ident, $expr:expr $(, $($extra:tt)*)?) => {{
        let expected = $expr;

        macro_rules! inner {
            ($name:ident) => {{
                assert_eq!(
                    $crate::$name::test::rt($expr), expected,
                    "{}: roundtripped value does not match expected",
                    stringify!($name),
                );
            }}
        }

        $crate::macros::test_matrix!($support, inner);
        $crate::macros::support::musli_value_rt($expr);
        $crate::macros::extra!($expr $(, $($extra)*)*);
        expected
    }};
}

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(inline)]
/// Assert that expression `$expr` can be roundtrip encoded using the encodings
/// specified by `$support`.
///
/// This demands that encoding and subsequently decoding `$expr` through any
/// formats results in a value that can be compared using [`PartialEq`] to
/// `$expr`.
///
/// This can be used to test upgrade stable formats.
///
/// The `$support` parameter is one of:
/// * `full` - All formats are tested.
/// * `no_json` - All formats except JSON are tested.
/// * `descriptive` - All fully self-descriptive formats are tested.
/// * `json` - Only JSON is tested.
/// * `upgrade_stable` - Only upgrade-stable formats are tested.
///
/// Extra tests can be specified using the `$extra` parameter:
/// * `json = <expected>` - Assert that the JSON encoding of `$expr` matched
///   exactly `$expected`.
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Version1 {
///     name: String,
/// }
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Person {
///     name: String,
///     #[musli(default)]
///     age: Option<u32>,
/// }
///
/// musli::macros::assert_roundtrip_eq! {
///     full,
///     Person {
///         name: String::from("Aristotle"),
///         age: Some(62),
///     },
///     json = r#"{"name":"Aristotle","age":62}"#,
/// };
/// ```
pub use __assert_roundtrip_eq as assert_roundtrip_eq;

#[cfg(feature = "test")]
#[macro_export]
#[doc(hidden)]
macro_rules! __assert_decode_eq {
    ($support:ident, $expr:expr, $expected:expr $(, $($extra:tt)*)?) => {{
        let mut bytes = $crate::macros::support::Vec::<u8>::new();

        macro_rules! decode {
            ($name:ident) => {{
                $crate::$name::test::decode($expr, &mut bytes, &$expected);
            }}
        }

        $crate::macros::test_matrix!($support, decode);
        $crate::macros::extra!($expr $(, $($extra)*)*);
    }};
}

#[cfg(feature = "test")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "test")))]
#[doc(inline)]
/// Assert that expression `$expr` can decode to expression `$expected` using
/// the encodings specified by `$support`.
///
/// This can be used to test upgrade stable formats.
///
/// The `$support` parameter is one of:
/// * `full` - All formats are tested.
/// * `no_json` - All formats except JSON are tested.
/// * `descriptive` - All fully self-descriptive formats are tested.
/// * `json` - Only JSON is tested.
/// * `upgrade_stable` - Only upgrade-stable formats are tested.
///
/// Extra tests can be specified using the `$extra` parameter:
/// * `json = <expected>` - Assert that the JSON encoding of `$expr` matched
///   exactly `$expected`.
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Version1 {
///     name: String,
/// }
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct Version2 {
///     name: String,
///     #[musli(default)]
///     age: Option<u32>,
/// }
///
/// // Only upgrade stable formats can remove an existing field since they need
/// // to know how to skip it over.
/// musli::macros::assert_decode_eq! {
///     upgrade_stable,
///     Version2 {
///         name: String::from("Aristotle"),
///         age: Some(62),
///     },
///     Version1 {
///         name: String::from("Aristotle"),
///     },
///     json = r#"{"name":"Aristotle","age":62}"#,
/// };
///
/// // Every supported format can add a new field which has a default value.
/// musli::macros::assert_decode_eq! {
///     full,
///     Version1 {
///         name: String::from("Aristotle"),
///     },
///     Version2 {
///         name: String::from("Aristotle"),
///         age: None,
///     },
///     json = r#"{"name":"Aristotle"}"#,
/// };
/// ```
pub use __assert_decode_eq as assert_decode_eq;

#[cfg(feature = "test")]
#[macro_export]
#[doc(hidden)]
macro_rules! __extra {
    ($expr:expr $(,)?) => {};

    ($expr:expr, json = $json_expected:expr $(, $($extra:tt)*)?) => {{
        let json = $crate::json::test::to_vec($expr);
        let string = ::std::string::String::from_utf8(json).expect("Encoded JSON is not valid utf-8");

        assert_eq!(
            string, $json_expected,
            "json: encoded json does not match expected value"
        );

        $crate::macros::extra!($expr $(, $($extra)*)*);
    }};
}

#[cfg(feature = "test")]
#[doc(hidden)]
pub use __extra as extra;

#[cfg(feature = "test")]
#[macro_export]
#[doc(hidden)]
macro_rules! __test_matrix {
    (full, $call:path) => {
        $call!(storage);
        $call!(wire);
        $call!(descriptive);
        $call!(json);
    };

    (no_json, $call:path) => {
        $call!(storage);
        $call!(wire);
        $call!(descriptive);
    };

    (descriptive, $call:path) => {
        $call!(descriptive);
        $call!(json);
    };

    (json, $call:path) => {
        $call!(json);
    };

    (upgrade_stable, $call:path) => {
        $call!(wire);
        $call!(descriptive);
        $call!(json);
    };
}

#[cfg(feature = "test")]
#[doc(hidden)]
pub use __test_matrix as test_matrix;

#[cfg(all(feature = "test", feature = "alloc"))]
#[doc(hidden)]
pub mod support {
    pub use alloc::vec::Vec;

    use crate::mode::Binary;
    use crate::value::{self, Value};
    use crate::{Decode, Encode};

    #[track_caller]
    pub fn musli_value_rt<T>(expected: T)
    where
        T: Encode<Binary> + for<'de> Decode<'de, Binary>,
        T: PartialEq + core::fmt::Debug,
    {
        let value: Value = value::encode(&expected).expect("value: Encoding should succeed");
        let actual: T = value::decode(&value).expect("value: Decoding should succeed");
        assert_eq!(
            actual, expected,
            "value: roundtripped value does not match expected"
        );
    }
}
