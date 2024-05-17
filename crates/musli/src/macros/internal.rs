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
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        #[doc = concat!("let person: Person = ", stringify!($what), "::from_slice(&data[..])?;")]
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice(&data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &data[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
        ///     age: 61,
        /// })?;
        ///
        /// let person: Person = ENCODING.from_slice_with(&cx, &buf[..])?;
        /// assert_eq!(person.name, "Aristotle");
        /// assert_eq!(person.age, 61);
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
