/// Generate extensions assuming an encoding has implemented encode_with.
#[doc(hidden)]
#[macro_export]
macro_rules! encode_with_extensions {
    () => {
        /// Encode the given value to the given [Writer] using the current
        /// configuration.
        #[inline]
        pub fn encode<W, T>(self, writer: W, value: &T) -> Result<(), W::Error>
        where
            W: Writer,
            T: ?Sized + Encode<M>,
        {
            let mut cx = musli_common::context::Same::default();
            self.encode_with(&mut cx, writer, value)
        }

        /// Encode the given value to the given [Write][io::Write] using the current
        /// configuration.
        #[cfg(feature = "std")]
        #[inline]
        pub fn to_writer<W, T>(self, write: W, value: &T) -> Result<(), io::Error>
        where
            W: io::Write,
            T: ?Sized + Encode<M>,
        {
            let mut writer = $crate::wrap::wrap(write);
            self.encode(&mut writer, value)
        }

        /// Encode the given value to a [`Buffer`] using the current configuration.
        #[inline]
        pub fn to_buffer<T>(self, value: &T) -> Result<Buffer, BufferError>
        where
            T: ?Sized + Encode<M>,
        {
            let mut data = Buffer::new();
            self.encode(&mut data, value)?;
            Ok(data)
        }

        /// Encode the given value to a [`Buffer`] using the current configuration.
        ///
        /// This is the same as [`Encoding::to_buffer`], but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn to_buffer_with<'buf, C, T>(self, cx: &mut C, value: &T) -> Result<Buffer, C::Error>
        where
            C: Context<'buf, Input = BufferError>,
            T: ?Sized + Encode<M>,
        {
            let mut data = Buffer::new();
            self.encode_with(cx, &mut data, value)?;
            Ok(data)
        }

        /// Encode the given value to a [`Vec`] using the current configuration.
        #[cfg(feature = "alloc")]
        #[inline]
        pub fn to_vec<T>(self, value: &T) -> Result<Vec<u8>, BufferError>
        where
            T: ?Sized + Encode<M>,
        {
            Ok(self.to_buffer(value)?.into_vec())
        }

        /// Encode the given value to a [`Vec`] using the current configuration.
        ///
        /// This is the same as [`Encoding::to_vec`], but allows for using a
        /// configurable [`Context`].
        #[cfg(feature = "alloc")]
        #[inline]
        pub fn to_vec_with<'buf, C, T>(self, cx: &mut C, value: &T) -> Result<Vec<u8>, C::Error>
        where
            C: Context<'buf, Input = BufferError>,
            T: ?Sized + Encode<M>,
        {
            Ok(self.to_buffer_with(cx, value)?.into_vec())
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// configuration.
        #[inline]
        pub fn to_fixed_bytes<const N: usize, T>(
            self,
            value: &T,
        ) -> Result<FixedBytes<N>, BufferError>
        where
            T: ?Sized + Encode<M>,
        {
            let mut cx = musli_common::context::Same::default();
            self.to_fixed_bytes_with(&mut cx, value)
        }

        /// Encode the given value to a fixed-size bytes using the current
        /// configuration.
        #[inline]
        pub fn to_fixed_bytes_with<'buf, C, const N: usize, T>(
            self,
            cx: &mut C,
            value: &T,
        ) -> Result<FixedBytes<N>, C::Error>
        where
            C: Context<'buf, Input = BufferError>,
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
        pub fn from_slice<'de, T>(self, bytes: &'de [u8]) -> Result<T, BufferError>
        where
            T: Decode<'de, M>,
        {
            let mut cx = musli_common::context::Same::default();
            let reader = SliceReader::new(bytes);
            T::decode(&mut cx, $decoder_new(reader))
        }

        /// Decode the given type `T` from the given slice using the current
        /// configuration.
        ///
        /// This is the same as [`Encoding::from_slice`], but allows for using a
        /// configurable [`Context`].
        #[inline]
        pub fn from_slice_with<'de, 'buf, C, T>(
            self,
            cx: &mut C,
            bytes: &'de [u8],
        ) -> Result<T, C::Error>
        where
            C: Context<'buf, Input = BufferError>,
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
        pub fn encode_with<'buf, C, W, T>(
            self,
            cx: &mut C,
            writer: W,
            value: &T,
        ) -> Result<(), C::Error>
        where
            C: Context<'buf, Input = W::Error>,
            W: Writer,
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
        pub fn decode_with<'de, 'buf, C, R, T>(self, cx: &mut C, reader: R) -> Result<T, C::Error>
        where
            C: Context<'buf, Input = R::Error>,
            R: Reader<'de>,
            T: Decode<'de, M>,
        {
            T::decode(cx, $decoder_new(reader))
        }

        /// Decode the given type `T` from the given [Reader] using the current
        /// configuration.
        #[inline]
        pub fn decode<'de, R, T>(self, reader: R) -> Result<T, R::Error>
        where
            R: Reader<'de>,
            T: Decode<'de, M>,
        {
            let mut cx = musli_common::context::Same::default();
            self.decode_with(&mut cx, reader)
        }

        $crate::encode_with_extensions!();
    };
}
