#[macro_export]
macro_rules! check_length {
    ({@nolen $($tail:tt)*}, $item:item) => {
    };

    ({$head:tt $($tail:tt)*}, $item:item) => {
        check_length!({$($tail)*}, $item);
    };

    ({}, $item:item) => {
        $item
    };
}

/// Super macro to help build benchmarkers without too much of a fuzz.
///
/// Benchmarking might juggle a fair bit of state around depending on the exact implementation, so implementing the API manually is also an option.
#[allow(unused)]
#[macro_export]
macro_rules! benchmarker {
    (
        $lt:lifetime $({$($options:tt)*})?

        $(#[$($buffer_meta:meta)*])*
        $buffer_vis:vis fn buffer() -> $buffer:ty $build_buffer:block

        $(#[$($reset_meta:meta)*])*
        $reset_vis:vis fn reset<T>(&mut $reset_self:ident, $size_hint:tt: usize, $reset_value:tt: &T)
        $(where
            T: $reset_bound:path $(,)?)?
        $reset:block

        $(#[$($encode_meta:meta)*])*
        $encode_vis:vis fn encode<T>(&mut $encode_self:ident, $encode_value:ident: &T) -> Result<$encode_return:ty, $encode_error:ty>
        $(where
            T: $encode_bound:path $(,)?)?
        $encode:block

        $(#[$($decode_meta:meta)*])*
        $decode_vis:vis fn decode<T>(&$decode_self:ident) -> Result<$decode_return:ty, $decode_error:ty>
        $(where
            $(for<$for_lt:lifetime>)? T: $decode_bound:path $(,)?)*
        $decode:block
    ) => {
        pub struct Benchmarker {
            buffer: $buffer,
        }

        #[inline(always)]
        $buffer_vis fn new() -> Benchmarker {
            Benchmarker {
                buffer: $build_buffer
            }
        }

        #[inline(always)]
        $(#[$($decode_meta)*])*
        $decode_vis fn decode<$lt, T>(buffer: $encode_return) -> Result<$decode_return, $decode_error>
        $(where
            $(for<$for_lt>)* T: $decode_bound,)*
        {
            pub struct DecodeState<$lt> {
                buffer: $encode_return,
                _marker: ::core::marker::PhantomData<&$lt ()>,
            }

            impl<$lt> DecodeState<$lt> {
                #[inline(always)]
                pub fn decode<T>(&$decode_self) -> Result<T, $decode_error>
                $(where
                    $(for<$for_lt>)* T: $decode_bound,)*
                $decode
            }

            let state = DecodeState {
                buffer,
                _marker: ::core::marker::PhantomData,
            };

            state.decode()
        }

        impl Benchmarker {
            #[inline(always)]
            pub fn with<I, O>(&mut self, inner: I) -> O
            where
                I: FnOnce(State<'_>) -> O
            {
                inner(State {
                    buffer: &mut self.buffer,
                })
            }
        }

        pub struct State<$lt> {
            #[allow(unused)]
            buffer: &$lt mut $buffer,
        }

        impl<$lt> State<$lt> {
            #[inline(always)]
            $(#[$($reset_meta)*])*
            $reset_vis fn reset<T>(&mut $reset_self, $size_hint: usize, $reset_value: &T)
            $(where
                T: $reset_bound,)*
            $reset

            #[inline(always)]
            $(#[$($encode_meta)*])*
            $encode_vis fn encode<T>(&mut $encode_self, $encode_value: &T) -> Result<EncodeState, $encode_error>
            $(where
                T: $encode_bound,)*
            {
                let value: Result<_, $encode_error> = $encode;
                let buffer = value?;

                Ok(EncodeState {
                    buffer,
                    _marker: ::core::marker::PhantomData,
                })
            }
        }

        pub struct EncodeState<$lt> {
            buffer: $encode_return,
            _marker: ::core::marker::PhantomData<&$lt ()>,
        }

        impl<$lt> EncodeState<$lt> {
            check_length! {
                {$($($options)*)*},

                #[inline(always)]
                pub fn len(&self) -> usize {
                    self.buffer.len()
                }
            }

            #[inline(always)]
            $decode_vis fn decode<T>(&$decode_self) -> Result<$decode_return, $decode_error>
            $(where
                $(for<$for_lt>)* T: $decode_bound,)*
            $decode
        }
    }
}
