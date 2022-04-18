/// Declare an error representation.
#[macro_export]
macro_rules! decl_message_repr {
    ($vis:vis $ident:ident, $fallback:literal) => {
        #[cfg(feature = "std")]
        $vis struct $ident(Box<str>);

        #[cfg(feature = "std")]
        impl $ident {
            $vis fn collect<T>(message: T) -> Self where T: core::fmt::Display {
                Self(message.to_string().into())
            }
        }

        #[cfg(feature = "std")]
        impl core::fmt::Debug for $ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }

        #[cfg(feature = "std")]
        impl core::fmt::Display for $ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }

        #[cfg(not(feature = "std"))]
        $vis struct $ident;

        #[cfg(not(feature = "std"))]
        impl $ident {
            $vis fn collect<T>(_: T) -> Self where T: core::fmt::Display {
                Self
            }
        }

        #[cfg(not(feature = "std"))]
        impl core::fmt::Debug for $ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                $fallback.fmt(f)
            }
        }

        #[cfg(not(feature = "std"))]
        impl core::fmt::Display for $ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                $fallback.fmt(f)
            }
        }
    }
}
