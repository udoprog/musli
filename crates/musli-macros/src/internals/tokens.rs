use proc_macro2::Span;

pub(crate) struct Tokens {
    pub(crate) decode_t: syn::ExprPath,
    pub(crate) decoder_t: syn::ExprPath,
    pub(crate) default_function: syn::ExprPath,
    pub(crate) default_mode: syn::ExprPath,
    pub(crate) encode_t: syn::ExprPath,
    pub(crate) encoder_t: syn::ExprPath,
    pub(crate) error_t: syn::ExprPath,
    pub(crate) fmt: syn::ExprPath,
    pub(crate) mode_t: syn::ExprPath,
    pub(crate) pack_decoder_t: syn::ExprPath,
    pub(crate) pair_decoder_t_first: syn::ExprPath,
    pub(crate) pair_decoder_t: syn::ExprPath,
    pub(crate) pair_encoder_t: syn::ExprPath,
    pub(crate) pairs_decoder_t: syn::ExprPath,
    pub(crate) pairs_encoder_t: syn::ExprPath,
    pub(crate) sequence_encoder_t: syn::ExprPath,
    pub(crate) variant_decoder_t_tag: syn::ExprPath,
    pub(crate) variant_decoder_t: syn::ExprPath,
    pub(crate) variant_encoder_t: syn::ExprPath,
    pub(crate) visit_string_fn: syn::ExprPath,
    pub(crate) as_decoder_t: syn::ExprPath,
}

impl Tokens {
    pub(crate) fn new(span: Span, prefix: &syn::ExprPath) -> Self {
        Self {
            decode_t: path(span, prefix, ["de", "Decode"]),
            decoder_t: path(span, prefix, ["de", "Decoder"]),
            default_function: core_path(span, ["default", "Default", "default"]),
            default_mode: path(span, prefix, ["mode", "DefaultMode"]),
            encode_t: path(span, prefix, ["en", "Encode"]),
            encoder_t: path(span, prefix, ["en", "Encoder"]),
            error_t: path(span, prefix, ["error", "Error"]),
            fmt: core_path(span, ["fmt"]),
            mode_t: path(span, prefix, ["mode", "Mode"]),
            pack_decoder_t: path(span, prefix, ["de", "PackDecoder"]),
            pair_decoder_t_first: path(span, prefix, ["de", "PairDecoder", "first"]),
            pair_decoder_t: path(span, prefix, ["de", "PairDecoder"]),
            pair_encoder_t: path(span, prefix, ["en", "PairEncoder"]),
            pairs_decoder_t: path(span, prefix, ["de", "PairsDecoder"]),
            pairs_encoder_t: path(span, prefix, ["en", "PairsEncoder"]),
            sequence_encoder_t: path(span, prefix, ["en", "SequenceEncoder"]),
            variant_decoder_t_tag: path(span, prefix, ["de", "VariantDecoder", "tag"]),
            variant_decoder_t: path(span, prefix, ["de", "VariantDecoder"]),
            variant_encoder_t: path(span, prefix, ["en", "VariantEncoder"]),
            visit_string_fn: path(span, prefix, ["utils", "visit_string_fn"]),
            as_decoder_t: path(span, prefix, ["de", "AsDecoder"]),
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
