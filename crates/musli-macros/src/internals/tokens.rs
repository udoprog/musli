use proc_macro2::{Span, TokenStream};
use quote::quote;

pub(crate) struct Tokens {
    pub(crate) decode_t: syn::ExprPath,
    pub(crate) decoder_t: syn::ExprPath,
    pub(crate) decoder_var: syn::Ident,
    pub(crate) default_t: TokenStream,
    pub(crate) encode_t: syn::ExprPath,
    pub(crate) encoder_t: syn::ExprPath,
    pub(crate) encoder_var: syn::Ident,
    pub(crate) error_t: syn::ExprPath,
    pub(crate) fmt: syn::ExprPath,
    pub(crate) pack_decoder_t: syn::ExprPath,
    pub(crate) pair_decoder_t: syn::ExprPath,
    pub(crate) pair_encoder_t: syn::ExprPath,
    pub(crate) pairs_decoder_t: syn::ExprPath,
    pub(crate) pairs_encoder_t: syn::ExprPath,
    pub(crate) phantom_data: syn::ExprPath,
    pub(crate) value_visitor_t: syn::ExprPath,
    pub(crate) sequence_encoder_t: syn::ExprPath,
    pub(crate) default_mode: syn::ExprPath,
}

impl Tokens {
    pub(crate) fn new(span: Span, prefix: &syn::ExprPath) -> Self {
        Self {
            decode_t: path(span, prefix, ["de", "Decode"]),
            decoder_t: path(span, prefix, ["de", "Decoder"]),
            decoder_var: syn::Ident::new("decoder", span),
            default_t: quote!(::core::default::Default::default()),
            encode_t: path(span, prefix, ["en", "Encode"]),
            encoder_t: path(span, prefix, ["en", "Encoder"]),
            encoder_var: syn::Ident::new("encoder", span),
            error_t: path(span, prefix, ["error", "Error"]),
            fmt: core_path(span, ["fmt"]),
            pack_decoder_t: path(span, prefix, ["de", "PackDecoder"]),
            pair_decoder_t: path(span, prefix, ["de", "PairDecoder"]),
            pair_encoder_t: path(span, prefix, ["en", "PairEncoder"]),
            pairs_decoder_t: path(span, prefix, ["de", "PairsDecoder"]),
            pairs_encoder_t: path(span, prefix, ["en", "PairsEncoder"]),
            phantom_data: core_path(span, ["marker", "PhantomData"]),
            value_visitor_t: path(span, prefix, ["de", "ValueVisitor"]),
            sequence_encoder_t: path(span, prefix, ["en", "SequenceEncoder"]),
            default_mode: path(span, prefix, ["mode", "DefaultMode"]),
        }
    }
}

fn path<const N: usize>(
    span: Span,
    prefix: &syn::ExprPath,
    segments: [&'static str; N],
) -> syn::ExprPath {
    let mut path = prefix.clone();

    for segment in segments {
        path.path
            .segments
            .push(syn::Ident::new(segment, span).into());
    }

    path
}

fn core_path<const N: usize>(span: Span, segments: [&'static str; N]) -> syn::ExprPath {
    let core = syn::Ident::new("core", span);
    let mut path = syn::Path::from(core);

    for segment in segments {
        path.segments.push(syn::Ident::new(segment, span).into());
    }

    syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path,
    }
}
