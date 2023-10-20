use core::mem::take;

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
}

impl Enumerator {
    pub(super) fn new(num: NumericalRepr, span: Span) -> Self {
        Self {
            current: None,
            num,
            span,
        }
    }

    pub(super) fn next_index(
        &self,
        negative: bool,
        lit: &syn::LitInt,
    ) -> syn::Result<(bool, syn::LitInt)> {
        macro_rules! arm {
            ($kind:ident, $parse:ty, $ty:ty) => {{
                macro_rules! handle_neg {
                    (signed, $lit:ident) => {{
                        if negative && $lit != 0 {
                            let Some($lit) = $lit.checked_sub(1) else {
                                return Err(syn::Error::new_spanned(
                                    $lit,
                                    "Discriminant overflow for representation",
                                ));
                            };

                            ($lit != 0, $lit)
                        } else {
                            let Some($lit) = $lit.checked_add(1) else {
                                return Err(syn::Error::new_spanned(
                                    $lit,
                                    "Discriminant overflow for representation",
                                ));
                            };

                            (false, $lit)
                        }
                    }};

                    (unsigned, $lit:ident) => {{
                        if negative {
                            return Err(syn::Error::new_spanned(
                                $lit,
                                "Unsigned types can't be negative",
                            ));
                        }

                        let Some($lit) = $lit.checked_add(1) else {
                            return Err(syn::Error::new_spanned(
                                $lit,
                                "Discriminant overflow for representation",
                            ));
                        };

                        (false, $lit)
                    }};
                }

                let lit = lit.base10_parse::<$parse>()?;
                let (neg, lit) = handle_neg!($kind, lit);

                Ok((
                    neg,
                    syn::LitInt::new(&format!("{lit}{}", stringify!($ty)), self.span),
                ))
            }};
        }

        match self.num {
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
        }
    }

    /// Get the next discriminant based on the provided expression.
    pub(super) fn next(&mut self, discriminant: Option<&syn::Expr>) -> syn::Result<syn::Expr> {
        let current = if let Some(expr) = discriminant {
            self.current = Some(expr.clone());
            expr.clone()
        } else {
            let current = match take(&mut self.current) {
                Some(existing) => {
                    // We keep as_int, while it slightly bloats the macro it
                    // simplifies the resulting outgoing expression, which
                    // improves diagnostics.
                    if let Some((negative, lit)) = as_int(&existing) {
                        let (negative, lit) = self.next_index(negative, lit)?;

                        if negative {
                            syn::Expr::Unary(syn::ExprUnary {
                                attrs: Vec::new(),
                                op: syn::UnOp::Neg(<Token![-]>::default()),
                                expr: Box::new(syn::Expr::Lit(syn::ExprLit {
                                    attrs: Vec::new(),
                                    lit: syn::Lit::Int(lit),
                                })),
                            })
                        } else {
                            syn::Expr::Lit(syn::ExprLit {
                                attrs: Vec::new(),
                                lit: syn::Lit::Int(lit),
                            })
                        }
                    } else {
                        syn::Expr::Binary(syn::ExprBinary {
                            attrs: Vec::new(),
                            left: Box::new(existing),
                            op: syn::BinOp::Add(<Token![+]>::default()),
                            right: Box::new(syn::Expr::Lit(syn::ExprLit {
                                attrs: Vec::new(),
                                lit: syn::Lit::Int(syn::LitInt::new(
                                    &format!("1{}", self.num.as_ty()),
                                    self.span,
                                )),
                            })),
                        })
                    }
                }
                None => syn::Expr::Lit(syn::ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Int(syn::LitInt::new(
                        &format!("0{}", self.num.as_ty()),
                        self.span,
                    )),
                }),
            };

            self.current = Some(current.clone());
            current
        };

        Ok(syn::Expr::Group(syn::ExprGroup {
            attrs: Vec::new(),
            group_token: syn::token::Group { span: self.span },
            expr: Box::new(current),
        }))
    }
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
