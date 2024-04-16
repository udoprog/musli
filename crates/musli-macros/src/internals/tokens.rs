use proc_macro2::Span;

pub(crate) struct Tokens {
    pub(crate) as_decoder_t: syn::Path,
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
    pub(crate) option: syn::Path,
    pub(crate) pack_decoder_t: syn::Path,
    pub(crate) pack_encoder_t: syn::Path,
    pub(crate) priv_write: syn::Path,
    pub(crate) result_err: syn::Path,
    pub(crate) result_ok: syn::Path,
    pub(crate) result: syn::Path,
    pub(crate) skip_field: syn::Path,
    pub(crate) skip: syn::Path,
    pub(crate) map_decoder_t: syn::Path,
    pub(crate) map_encoder_t: syn::Path,
    pub(crate) struct_field_decoder_t: syn::Path,
    pub(crate) map_entry_encoder_t: syn::Path,
    pub(crate) struct_hint: syn::Path,
    pub(crate) trace_decode_t: syn::Path,
    pub(crate) trace_encode_t: syn::Path,
    pub(crate) unsized_struct_hint: syn::Path,
    pub(crate) variant_decoder_t: syn::Path,
    pub(crate) variant_encoder_t: syn::Path,
}

impl Tokens {
    pub(crate) fn new(span: Span, prefix: &syn::Path) -> Self {
        Self {
            as_decoder_t: path(span, prefix, ["de", "AsDecoder"]),
            context_t: path(span, prefix, ["Context"]),
            decode_bytes_t: path(span, prefix, ["de", "DecodeBytes"]),
            decode_t: path(span, prefix, ["de", "Decode"]),
            decoder_t: path(span, prefix, ["de", "Decoder"]),
            default_function: path(span, prefix, ["__priv", "default"]),
            default_mode: path(span, prefix, ["mode", "DefaultMode"]),
            encode_bytes_t: path(span, prefix, ["en", "EncodeBytes"]),
            encode_t: path(span, prefix, ["en", "Encode"]),
            encoder_t: path(span, prefix, ["en", "Encoder"]),
            fmt: path(span, prefix, ["__priv", "fmt"]),
            option_none: path(span, prefix, ["__priv", "None"]),
            option_some: path(span, prefix, ["__priv", "Some"]),
            option: path(span, prefix, ["__priv", "Option"]),
            pack_decoder_t: path(span, prefix, ["de", "PackDecoder"]),
            pack_encoder_t: path(span, prefix, ["en", "PackEncoder"]),
            priv_write: path(span, prefix, ["__priv", "write"]),
            result_err: path(span, prefix, ["__priv", "Err"]),
            result_ok: path(span, prefix, ["__priv", "Ok"]),
            result: path(span, prefix, ["__priv", "Result"]),
            skip_field: path(span, prefix, ["__priv", "skip_field"]),
            skip: path(span, prefix, ["__priv", "skip"]),
            map_decoder_t: path(span, prefix, ["de", "MapDecoder"]),
            map_encoder_t: path(span, prefix, ["en", "MapEncoder"]),
            struct_field_decoder_t: path(span, prefix, ["de", "MapEntryDecoder"]),
            map_entry_encoder_t: path(span, prefix, ["en", "MapEntryEncoder"]),
            struct_hint: path(span, prefix, ["hint", "StructHint"]),
            trace_decode_t: path(span, prefix, ["de", "TraceDecode"]),
            trace_encode_t: path(span, prefix, ["en", "TraceEncode"]),
            unsized_struct_hint: path(span, prefix, ["hint", "UnsizedStructHint"]),
            variant_decoder_t: path(span, prefix, ["de", "VariantDecoder"]),
            variant_encoder_t: path(span, prefix, ["en", "VariantEncoder"]),
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
