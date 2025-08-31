macro_rules! __slice_sequence {
    (
        $(#[$($meta:meta)*])*
        $cx:ident,
        $ty:ident <T $(, $alloc:ident)?>,
        || $new:expr,
        |$vec:ident, $value:ident| $insert:expr,
        |$reserve_vec:ident, $reserve_capacity:ident| $reserve:expr,
        |$capacity:ident| $with_capacity:expr,
    ) => {
        $(#[$($meta)*])*
        impl<M, T $(, $alloc)*> $crate::en::Encode<M> for $ty<T $(, $alloc)*>
        where
            T: $crate::en::Encode<M>,
            $($alloc: Allocator,)*
        {
            const IS_BITWISE_ENCODE: bool = false;

            type Encode = Self;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: $crate::en::Encoder<Mode = M>,
            {
                encoder.encode_slice(self)
            }

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        $(#[$($meta)*])*
        impl<'de, M, A, T> $crate::de::Decode<'de, M, A> for $ty<T $(, $alloc)*>
        where
            A: $crate::alloc::Allocator,
            T: $crate::de::Decode<'de, M, A>,
        {
            const IS_BITWISE_DECODE: bool = false;

            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de, Mode = M, Allocator = A>,
            {
                struct Builder<'de, M, A, T>
                where
                    $($alloc: $crate::alloc::Allocator,)*
                {
                    vec: $ty<T $(, $alloc)*>,
                    _marker: core::marker::PhantomData<(M, A, &'de ())>
                }

                #[allow(unused_variables)]
                impl<'de, M, A, T> $crate::de::DecodeSliceBuilder<T, A> for Builder<'de, M, A, T>
                where
                    T: $crate::de::Decode<'de, M, A>,
                    A: $crate::alloc::Allocator,
                {
                    #[inline]
                    fn new<C>($cx: C) -> Result<Self, C::Error>
                    where
                        C: $crate::Context<Allocator = A>,
                    {
                        Ok(Builder {
                            vec: $new,
                            _marker: core::marker::PhantomData
                        })
                    }

                    #[inline]
                    fn with_capacity<C>($cx: C, $capacity: usize) -> Result<Self, C::Error>
                    where
                        C: $crate::Context<Allocator = A>,
                    {
                        Ok(Builder {
                            vec: $with_capacity,
                            _marker: core::marker::PhantomData
                        })
                    }

                    #[inline]
                    fn push<C>(&mut self, $cx: C, $value: T) -> Result<(), C::Error>
                    where
                        C: $crate::Context<Allocator = A>,
                    {
                        let $vec = &mut self.vec;
                        $insert;
                        Ok(())
                    }

                    #[inline]
                    fn reserve<C>( &mut self, $cx: C, $reserve_capacity: usize) -> Result<(), C::Error>
                    where
                        C: $crate::Context<Allocator = A>,
                    {
                        let $reserve_vec = &mut self.vec;
                        $reserve;
                        Ok(())
                    }

                    #[inline]
                    unsafe fn set_len(&mut self, len: usize) {
                        unsafe {
                            self.vec.set_len(len);
                        }
                    }

                    #[inline]
                    fn as_mut_ptr(&mut self) -> *mut T {
                        self.vec.as_mut_ptr()
                    }
                }

                let Builder { vec, _marker: core::marker::PhantomData } = decoder.decode_slice()?;
                Ok(vec)
            }
        }

        $(#[$($meta)*])*
        impl<M, T $(, $alloc)*> $crate::en::EncodePacked<M> for $ty<T $(, $alloc)*>
        where
            T: $crate::en::Encode<M>,
            $($alloc: $crate::alloc::Allocator,)*
        {
            #[inline]
            fn encode_packed<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: $crate::en::Encoder<Mode = M>,
            {
                encoder.encode_pack_fn(|pack| {
                    $crate::en::SequenceEncoder::encode_slice(pack, self)
                })
            }
        }
    }
}

pub(crate) use __slice_sequence as slice_sequence;
