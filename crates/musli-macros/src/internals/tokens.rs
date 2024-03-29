use proc_macro2::Span;
use syn::Token;

pub(crate) struct Tokens {
    pub(crate) as_decoder_t: syn::Path,
    pub(crate) buf_t: syn::Path,
    pub(crate) context_t: syn::Path,
    pub(crate) decode_bytes_t: syn::Path,
    pub(crate) decode_t: syn::Path,
    pub(crate) decoder_t: syn::Path,
    pub(crate) default_function: syn::Path,
    pub(crate) default_mode: syn::Path,
    pub(crate) encode_bytes_t: syn::Path,
    pub(crate) encode_t: syn::Path,
    pub(crate) encoder_t: syn::Path,
    pub(crate) fmt: syn::Path,
    pub(crate) option_none: syn::Path,
    pub(crate) option_some: syn::Path,
    pub(crate) pack_decoder_t: syn::Path,
    pub(crate) pack_encoder_t: syn::Path,
    pub(crate) result: syn::Path,
    pub(crate) skip: syn::Path,
    pub(crate) struct_decoder_t: syn::Path,
    pub(crate) struct_encoder_t: syn::Path,
    pub(crate) struct_field_decoder_t: syn::Path,
    pub(crate) struct_field_encoder_t: syn::Path,
    pub(crate) trace_decode_t: syn::Path,
    pub(crate) trace_encode_t: syn::Path,
    pub(crate) variant_decoder_t: syn::Path,
    pub(crate) variant_encoder_t: syn::Path,
    pub(crate) visit_owned_fn: syn::Path,
}

impl Tokens {
    pub(crate) fn new(span: Span, prefix: &syn::Path) -> Self {
        Self {
            as_decoder_t: path(span, prefix, ["de", "AsDecoder"]),
            buf_t: path(span, prefix, ["Buf"]),
            context_t: path(span, prefix, ["Context"]),
            decode_t: path(span, prefix, ["de", "Decode"]),
            decode_bytes_t: path(span, prefix, ["de", "DecodeBytes"]),
            decoder_t: path(span, prefix, ["de", "Decoder"]),
            default_function: core(span, ["default", "Default", "default"]),
            default_mode: path(span, prefix, ["mode", "DefaultMode"]),
            encode_t: path(span, prefix, ["en", "Encode"]),
            encode_bytes_t: path(span, prefix, ["en", "EncodeBytes"]),
            encoder_t: path(span, prefix, ["en", "Encoder"]),
            fmt: core(span, ["fmt"]),
            option_none: core(span, ["option", "Option", "None"]),
            option_some: core(span, ["option", "Option", "Some"]),
            pack_decoder_t: path(span, prefix, ["de", "PackDecoder"]),
            struct_decoder_t: path(span, prefix, ["de", "StructDecoder"]),
            struct_field_decoder_t: path(span, prefix, ["de", "StructFieldDecoder"]),
            struct_encoder_t: path(span, prefix, ["en", "StructEncoder"]),
            struct_field_encoder_t: path(span, prefix, ["en", "StructFieldEncoder"]),
            result: core(span, ["result", "Result"]),
            skip: path(span, prefix, ["de", "Skip"]),
            pack_encoder_t: path(span, prefix, ["en", "PackEncoder"]),
            trace_decode_t: path(span, prefix, ["de", "TraceDecode"]),
            trace_encode_t: path(span, prefix, ["en", "TraceEncode"]),
            variant_decoder_t: path(span, prefix, ["de", "VariantDecoder"]),
            variant_encoder_t: path(span, prefix, ["en", "VariantEncoder"]),
            visit_owned_fn: path(span, prefix, ["utils", "visit_owned_fn"]),
        }
    }
}

fn path<const N: usize>(span: Span, prefix: &syn::Path, segments: [&'static str; N]) -> syn::Path {
    let mut path = prefix.clone();

    for segment in segments {
        path.segments.push(syn::Ident::new(segment, span).into());
    }

    path
}

fn core<const N: usize>(span: Span, segments: [&'static str; N]) -> syn::Path {
    let core = syn::Ident::new("core", span);
    let mut path = syn::Path::from(core);
    path.leading_colon = Some(<Token![::]>::default());

    for segment in segments {
        path.segments.push(syn::Ident::new(segment, span).into());
    }

    path
}
