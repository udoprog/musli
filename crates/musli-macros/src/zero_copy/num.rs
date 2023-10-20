use std::fmt;
use std::mem::take;
use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Token;

#[derive(Clone, Copy)]
pub(super) enum NumericalRepr {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    Usize,
}

impl NumericalRepr {
    pub(super) fn size(self, mem: &syn::Path) -> NumericalSize<'_> {
        match self {
            NumericalRepr::U8 => NumericalSize::Fixed(1),
            NumericalRepr::U16 => NumericalSize::Fixed(2),
            NumericalRepr::U32 => NumericalSize::Fixed(4),
            NumericalRepr::U64 => NumericalSize::Fixed(8),
            NumericalRepr::U128 => NumericalSize::Fixed(16),
            NumericalRepr::I8 => NumericalSize::Fixed(1),
            NumericalRepr::I16 => NumericalSize::Fixed(2),
            NumericalRepr::I32 => NumericalSize::Fixed(4),
            NumericalRepr::I64 => NumericalSize::Fixed(8),
            NumericalRepr::I128 => NumericalSize::Fixed(16),
            NumericalRepr::Isize => NumericalSize::Pointer(mem),
            NumericalRepr::Usize => NumericalSize::Pointer(mem),
        }
    }

    pub(super) fn as_ty(self) -> &'static str {
        match self {
            NumericalRepr::U8 => "u8",
            NumericalRepr::U16 => "u16",
            NumericalRepr::U32 => "u32",
            NumericalRepr::U64 => "u64",
            NumericalRepr::U128 => "u128",
            NumericalRepr::I8 => "i8",
            NumericalRepr::I16 => "i16",
            NumericalRepr::I32 => "i32",
            NumericalRepr::I64 => "i64",
            NumericalRepr::I128 => "i128",
            NumericalRepr::Isize => "isize",
            NumericalRepr::Usize => "isize",
        }
    }
}

pub(super) enum NumericalSize<'a> {
    Fixed(usize),
    Pointer(&'a syn::Path),
}

impl ToTokens for NumericalSize<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NumericalSize::Fixed(size) => size.to_tokens(tokens),
            NumericalSize::Pointer(mem) => {
                tokens.extend(quote!(#mem::size_of::<usize>()));
            }
        }
    }
}

/// Helper to enumerate discriminants of an enum.
pub(super) struct Enumerator {
    current: Option<syn::Expr>,
    num: NumericalRepr,
    span: Span,
    zero: syn::LitInt,
    one: syn::LitInt,
}

impl Enumerator {
    pub(super) fn new(num: NumericalRepr, span: Span) -> Self {
        Self {
            current: None,
            num,
            span,
            zero: syn::LitInt::new(&format!("0{}", num.as_ty()), span),
            one: syn::LitInt::new(&format!("1{}", num.as_ty()), span),
        }
    }

    pub(super) fn next_index(&self, negative: bool, lit: &syn::LitInt) -> syn::Result<syn::Expr> {
        // Helper trait to process base numeral arithmetics.
        trait Numeral: Copy + FromStr {
            fn is_zero(self) -> bool;
            fn inc(self) -> Option<Self>;
            fn dec(self) -> Option<Self>;
        }

        macro_rules! impl_numeral {
            ($($ty:ty),*) => {
                $(impl Numeral for $ty {
                    #[inline] fn is_zero(self) -> bool { self == 0 }
                    #[inline] fn inc(self) -> Option<Self> { self.checked_add(1) }
                    #[inline] fn dec(self) -> Option<Self> { self.checked_sub(1) }
                })*
            }
        }

        impl_numeral!(u8, u16, u32, u64, u128, usize);

        fn signed<T>(neg: bool, lit: &syn::LitInt) -> syn::Result<(bool, T)>
        where
            T: Numeral,
            T::Err: fmt::Display,
        {
            let number = lit.base10_parse::<T>()?;

            // We still need to zero-check, because numerical literals can be
            // `-0`.
            if neg && !number.is_zero() {
                // NB: negative numbers are numerically decremented, as in -4
                // becomes -3.
                let Some(number) = number.dec() else {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "Discriminant overflow for representation",
                    ));
                };

                // If we reached 0, then the sign flips.
                Ok((!number.is_zero(), number))
            } else {
                let Some(number) = number.inc() else {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "Discriminant overflow for representation",
                    ));
                };

                Ok((false, number))
            }
        }

        fn unsigned<T>(neg: bool, lit: &syn::LitInt) -> syn::Result<(bool, T)>
        where
            T: Numeral,
            T::Err: fmt::Display,
        {
            let (neg, number) = signed(neg, lit)?;

            if neg {
                return Err(syn::Error::new_spanned(
                    lit,
                    "Unsigned discriminants cannot be negative",
                ));
            }

            Ok((false, number))
        }

        macro_rules! arm {
            ($kind:ident, $parse:ty, $ty:ty) => {{
                let (negative, lit) = $kind::<$parse>(negative, lit)?;
                let lit = syn::LitInt::new(&format!("{lit}{}", stringify!($ty)), self.span);
                (negative, lit)
            }};
        }

        let (negative, lit) = match self.num {
            NumericalRepr::U8 => arm!(unsigned, u8, u8),
            NumericalRepr::U16 => arm!(unsigned, u16, u16),
            NumericalRepr::U32 => arm!(unsigned, u32, u32),
            NumericalRepr::U64 => arm!(unsigned, u64, u64),
            NumericalRepr::U128 => arm!(unsigned, u128, u128),
            NumericalRepr::Usize => arm!(unsigned, usize, usize),
            NumericalRepr::I8 => arm!(signed, u8, i8),
            NumericalRepr::I16 => arm!(signed, u16, i16),
            NumericalRepr::I32 => arm!(signed, u32, i32),
            NumericalRepr::I64 => arm!(signed, u64, i64),
            NumericalRepr::I128 => arm!(signed, u128, i128),
            NumericalRepr::Isize => arm!(signed, usize, isize),
        };

        Ok(if negative {
            syn::Expr::Unary(syn::ExprUnary {
                attrs: Vec::new(),
                op: syn::UnOp::Neg(<Token![-]>::default()),
                expr: Box::new(int_expr(lit)),
            })
        } else {
            int_expr(lit)
        })
    }

    /// Get the next discriminant based on the provided expression.
    pub(super) fn next(&mut self, discriminant: Option<&syn::Expr>) -> syn::Result<syn::Expr> {
        let current = if let Some(expr) = discriminant {
            self.current = Some(expr.clone());
            expr.clone()
        } else {
            let current = match take(&mut self.current) {
                Some(existing) => {
                    if let Some((negative, lit)) = as_int(&existing) {
                        // We keep as_int, while it slightly bloats the macro it
                        // simplifies the resulting outgoing expression, which
                        // improves diagnostics.

                        self.next_index(negative, lit)?
                    } else {
                        // Prior expression + 1
                        syn::Expr::Binary(syn::ExprBinary {
                            attrs: Vec::new(),
                            left: Box::new(existing),
                            op: syn::BinOp::Add(<Token![+]>::default()),
                            right: Box::new(int_expr(self.one.clone())),
                        })
                    }
                }
                None => int_expr(self.zero.clone()),
            };

            self.current = Some(current.clone());
            current
        };

        Ok(current)
    }
}

fn int_expr(lit: syn::LitInt) -> syn::Expr {
    syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit: syn::Lit::Int(lit),
    })
}

fn as_int(expr: &syn::Expr) -> Option<(bool, &syn::LitInt)> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) => Some((false, int)),
        syn::Expr::Group(syn::ExprGroup { expr, .. }) => as_int(expr),
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(..),
            expr,
            ..
        }) => as_int(expr).map(|(neg, int)| (!neg, int)),
        _ => None,
    }
}
