/// Generate extensions assuming an encoding has implemented encode_with.
#[doc(hidden)]
#[macro_export]
macro_rules! encode_with_extensions {
    () => {
        /// Encode the given value to the given [Writer] using the current
        /// configuration.
        #[inline]
        pub fn encode<W, T>(self, writer: W, value: &T) -> Result<(), Error>
        where
            W: Writer,
            Error: From<W::Error>,
            T: ?Sized + Encode<M>,
        {
            let alloc = musli_common::allocator::Default::default();
            let mut cx = musli_common::context::Same::new(&alloc);
            self.encode_with(&mut cx, writer, value)
        }

        /// Encode the given value to the given [Write][io::Write] using the current
        /// configuration.
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), Error>
        where
            W: io::Write,
            Error: From<io::Error>,
            T: ?Sized + Encode<M>,
        {
            let mut writer = $crate::wrap::wrap(write);
            self.encode(&mut writer, value)
        }

        /// Encode the given value to a [`Vec`] using the current configuration.
        #[cfg(feature = "alloc")]
        #[inline]
        pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, Error>
        where
            T: ?Sized + Encode<M>,
        {
            let mut vec = Vec::new();
            self.encode(&mut vec, value)?;
            Ok(vec)
        }

        /// Encode the given value to a [`Vec`] using the current configuration.
        ///
        /// This is the same as [`Encoding::to_vec`], but allows for using a
        /// configurable [`Context`].
        #[cfg(feature = "alloc")]
        #[inline]
        pub fn to_vec_with<C, T>(self, cx: &mut C, value: &T) -> Result<Vec<u8>, C::Error>
        where
            C: Context<Input = Error>,
            T: ?Sized + Encode<M>,
        {
            let mut vec = Vec::new();
            self.encode_with(cx, &mut vec, value)?;
            Ok(vec)
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// configuration.
        #[inline]
        pub fn to_fixed_bytes<const N: usize, T>(self, value: &T) -> Result<FixedBytes<N>, Error>
        where
            T: ?Sized + Encode<M>,
        {
            let alloc = musli_common::allocator::Default::default();
            let mut cx = musli_common::context::Same::new(&alloc);
            self.to_fixed_bytes_with(&mut cx, value)
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// configuration.
        #[inline]
        pub fn to_fixed_bytes_with<C, const N: usize, T>(
            self,
            cx: &mut C,
            value: &T,
        ) -> Result<FixedBytes<N>, C::Error>
        where
            C: Context<Input = Error>,
            T: ?Sized + Encode<M>,
        {
            let mut bytes = FixedBytes::new();
            self.encode_with(cx, &mut bytes, value)?;
            Ok(bytes)
        }
    };
}

/// Generate all public encoding helpers.
#[doc(hidden)]
#[macro_export]
macro_rules! encoding_from_slice_impls {
    ($encoder_new:path, $decoder_new:path) => {
        /// Decode the given type `T` from the given slice using the current
        /// configuration.
        #[inline]
        pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, Error>
        where
            T: Decode<'de, M>,
        {
            let alloc = musli_common::allocator::Default::default();
            let mut cx = musli_common::context::Same::new(&alloc);
            let reader = SliceReader::new(bytes);
            T::decode(&mut cx, $decoder_new(reader))
        }

        /// Decode the given type `T` from the given slice using the current
        /// configuration.
        ///
        /// This is the same as [`Encoding::from_slice`], but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn from_slice_with<'de, C, T>(self, cx: &mut C, bytes: &'de [u8]) -> Result<T, C::Error>
        where
            C: Context<Input = Error>,
            T: Decode<'de, M>,
        {
            let reader = SliceReader::new(bytes);
            T::decode(cx, $decoder_new(reader))
        }
    };
}

/// Generate all public encoding helpers.
#[doc(hidden)]
#[macro_export]
macro_rules! encoding_impls {
    ($encoder_new:path, $decoder_new:path) => {
        /// Encode the given value to the given [`Writer`] using the current
        /// configuration.
        ///
        /// This is the same as [`Encoding::encode`] but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn encode_with<C, W, T>(self, cx: &mut C, writer: W, value: &T) -> Result<(), C::Error>
        where
            C: Context<Input = Error>,
            W: Writer,
            Error: From<W::Error>,
            T: ?Sized + Encode<M>,
        {
            T::encode(value, cx, $encoder_new(writer))
        }

        /// Decode the given type `T` from the given [Reader] using the current
        /// configuration.
        ///
        /// This is the same as [`Encoding::decode`] but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn decode_with<'de, C, R, T>(self, cx: &mut C, reader: R) -> Result<T, C::Error>
        where
            C: Context<Input = Error>,
            R: Reader<'de>,
            Error: From<R::Error>,
            T: Decode<'de, M>,
        {
            T::decode(cx, $decoder_new(reader))
        }

        /// Decode the given type `T` from the given [Reader] using the current
        /// configuration.
        #[inline]
        pub fn decode<'de, R, T>(self, reader: R) -> Result<T, Error>
        where
            R: Reader<'de>,
            Error: From<R::Error>,
            T: Decode<'de, M>,
        {
            let alloc = musli_common::allocator::Default::default();
            let mut cx = musli_common::context::Same::new(&alloc);
            self.decode_with(&mut cx, reader)
        }

        $crate::encode_with_extensions!();
    };
}
