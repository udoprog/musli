use proc_macro2::{Ident, Span, TokenTree};
use quote::ToTokens;
use syn::Token;

#[derive(Clone, Copy)]
pub(crate) struct Import<'a>(&'a syn::Path, &'static str);

impl ToTokens for Import<'_> {
    #[inline]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Import(path, name) = *self;

        path.to_tokens(tokens);
        <Token![::]>::default().to_tokens(tokens);
        tokens.extend([TokenTree::Ident(Ident::new("__priv", Span::call_site()))]);
        <Token![::]>::default().to_tokens(tokens);
        tokens.extend([TokenTree::Ident(Ident::new(name, Span::call_site()))]);
    }
}

#[derive(Clone)]
pub(crate) struct Tokens<'a> {
    pub(crate) allocator_t: Import<'a>,
    pub(crate) as_decoder_t: Import<'a>,
    pub(crate) collect_string: Import<'a>,
    pub(crate) context_t: Import<'a>,
    pub(crate) decode_bytes_t: Import<'a>,
    pub(crate) decode_packed_t: Import<'a>,
    pub(crate) decode_t: Import<'a>,
    pub(crate) decoder_t: Import<'a>,
    pub(crate) default_function: Import<'a>,
    pub(crate) encode_bytes_t: Import<'a>,
    pub(crate) encode_packed_t: Import<'a>,
    pub(crate) encode_t: Import<'a>,
    pub(crate) encoder_t: Import<'a>,
    pub(crate) fmt: Import<'a>,
    pub(crate) map_decoder_t: Import<'a>,
    pub(crate) map_encoder_t: Import<'a>,
    pub(crate) map_entry_encoder_t: Import<'a>,
    pub(crate) map_hint: Import<'a>,
    pub(crate) messages: Import<'a>,
    pub(crate) needs_drop: Import<'a>,
    pub(crate) offset_of: Import<'a>,
    pub(crate) option: Import<'a>,
    pub(crate) pack_decoder_t: Import<'a>,
    pub(crate) result: Import<'a>,
    pub(crate) sequence_encoder_t: Import<'a>,
    pub(crate) size_of: Import<'a>,
    pub(crate) skip_field: Import<'a>,
    pub(crate) skip: Import<'a>,
    pub(crate) struct_field_decoder_t: Import<'a>,
    pub(crate) trace_decode_t: Import<'a>,
    pub(crate) trace_encode_t: Import<'a>,
    pub(crate) try_fast_decode: Import<'a>,
    pub(crate) try_fast_encode: Import<'a>,
    pub(crate) variant_decoder_t: Import<'a>,
    pub(crate) variant_encoder_t: Import<'a>,
    pub(crate) prefix: &'a syn::Path,
}

impl<'a> Tokens<'a> {
    pub(crate) fn new(prefix: &'a syn::Path) -> Self {
        Self {
            allocator_t: Import(prefix, "Allocator"),
            as_decoder_t: Import(prefix, "AsDecoder"),
            collect_string: Import(prefix, "collect_string"),
            context_t: Import(prefix, "Context"),
            decode_bytes_t: Import(prefix, "DecodeBytes"),
            decode_packed_t: Import(prefix, "DecodePacked"),
            decode_t: Import(prefix, "Decode"),
            decoder_t: Import(prefix, "Decoder"),
            default_function: Import(prefix, "default"),
            encode_bytes_t: Import(prefix, "EncodeBytes"),
            encode_packed_t: Import(prefix, "EncodePacked"),
            encode_t: Import(prefix, "Encode"),
            encoder_t: Import(prefix, "Encoder"),
            fmt: Import(prefix, "fmt"),
            map_decoder_t: Import(prefix, "MapDecoder"),
            map_encoder_t: Import(prefix, "MapEncoder"),
            map_entry_encoder_t: Import(prefix, "EntryEncoder"),
            map_hint: Import(prefix, "MapHint"),
            messages: Import(prefix, "m"),
            needs_drop: Import(prefix, "needs_drop"),
            offset_of: Import(prefix, "offset_of"),
            option: Import(prefix, "Option"),
            pack_decoder_t: Import(prefix, "SequenceDecoder"),
            result: Import(prefix, "Result"),
            sequence_encoder_t: Import(prefix, "SequenceEncoder"),
            size_of: Import(prefix, "size_of"),
            skip_field: Import(prefix, "skip_field"),
            skip: Import(prefix, "skip"),
            struct_field_decoder_t: Import(prefix, "EntryDecoder"),
            trace_decode_t: Import(prefix, "DecodeTrace"),
            trace_encode_t: Import(prefix, "EncodeTrace"),
            try_fast_decode: Import(prefix, "TryFastDecode"),
            try_fast_encode: Import(prefix, "TryFastEncode"),
            variant_decoder_t: Import(prefix, "VariantDecoder"),
            variant_encoder_t: Import(prefix, "VariantEncoder"),
            prefix,
        }
    }
}
