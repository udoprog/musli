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
    pub(crate) entry_decoder_t: Import<'a>,
    pub(crate) entry_encoder_t: Import<'a>,
    pub(crate) fmt: Import<'a>,
    pub(crate) map_decoder_t: Import<'a>,
    pub(crate) map_encoder_t: Import<'a>,
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
        macro_rules! import {
            ($name:ident) => {
                Import(prefix, stringify!($name))
            };
        }

        Self {
            allocator_t: import!(Allocator),
            collect_string: import!(collect_string),
            context_t: import!(Context),
            decode_bytes_t: import!(DecodeBytes),
            decode_packed_t: import!(DecodePacked),
            decode_t: import!(Decode),
            decoder_t: import!(Decoder),
            default_function: import!(default),
            encode_bytes_t: import!(EncodeBytes),
            encode_packed_t: import!(EncodePacked),
            encode_t: import!(Encode),
            encoder_t: import!(Encoder),
            entry_decoder_t: import!(EntryDecoder),
            entry_encoder_t: import!(EntryEncoder),
            fmt: import!(fmt),
            map_decoder_t: import!(MapDecoder),
            map_encoder_t: import!(MapEncoder),
            map_hint: import!(map_hint),
            messages: import!(m),
            needs_drop: import!(needs_drop),
            offset_of: import!(offset_of),
            option: import!(Option),
            pack_decoder_t: import!(SequenceDecoder),
            result: import!(Result),
            sequence_encoder_t: import!(SequenceEncoder),
            size_of: import!(size_of),
            skip_field: import!(skip_field),
            skip: import!(skip),
            trace_decode_t: import!(DecodeTrace),
            trace_encode_t: import!(EncodeTrace),
            try_fast_decode: import!(TryFastDecode),
            try_fast_encode: import!(TryFastEncode),
            variant_decoder_t: import!(VariantDecoder),
            variant_encoder_t: import!(VariantEncoder),
            prefix,
        }
    }
}
