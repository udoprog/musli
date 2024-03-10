use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::de::{Decode, Decoder, PackDecoder, VariantDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder, VariantEncoder};
use crate::mode::Mode;
use crate::Context;

impl<M> Encode<M> for Ipv4Addr
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_array(cx, self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv4Addr
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_array::<C, 4>(cx).map(Ipv4Addr::from)
    }
}

impl<M> Encode<M> for Ipv6Addr
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_array(cx, self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv6Addr
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_array::<C, 16>(cx).map(Ipv6Addr::from)
    }
}

impl<M> Encode<M> for IpAddr
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        let variant = encoder.encode_variant(cx)?;

        match self {
            IpAddr::V4(v4) => variant.insert::<M, _, _, _>(cx, 0usize, v4),
            IpAddr::V6(v6) => variant.insert::<M, _, _, _>(cx, 1usize, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for IpAddr
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant(cx)?;

        let this = match variant
            .tag(cx)
            .and_then(|v| <usize as Decode<M>>::decode(cx, v))?
        {
            0 => Self::V4(
                variant
                    .variant(cx)
                    .and_then(|v| <Ipv4Addr as Decode<M>>::decode(cx, v))?,
            ),
            1 => Self::V6(
                variant
                    .variant(cx)
                    .and_then(|v| <Ipv6Addr as Decode<M>>::decode(cx, v))?,
            ),
            index => {
                return Err(cx.invalid_variant_tag("IpAddr", index));
            }
        };

        variant.end(cx)?;
        Ok(this)
    }
}

impl<M> Encode<M> for SocketAddrV4
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        let mut pack = encoder.encode_pack(cx)?;
        pack.push::<M, _, _>(cx, self.ip())?;
        pack.push::<M, _, _>(cx, self.port())?;
        pack.end(cx)
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV4
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack(cx)?;
        let ip = unpack
            .next(cx)
            .and_then(|v| <Ipv4Addr as Decode<M>>::decode(cx, v))?;
        let port = unpack
            .next(cx)
            .and_then(|v| <u16 as Decode<M>>::decode(cx, v))?;
        unpack.end(cx)?;
        Ok(SocketAddrV4::new(ip, port))
    }
}

impl<M> Encode<M> for SocketAddrV6
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        let mut pack = encoder.encode_pack(cx)?;
        pack.push::<M, _, _>(cx, self.ip())?;
        pack.push::<M, _, _>(cx, self.port())?;
        pack.push::<M, _, _>(cx, self.flowinfo())?;
        pack.push::<M, _, _>(cx, self.scope_id())?;
        pack.end(cx)
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV6
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack(cx)?;
        let ip = unpack
            .next(cx)
            .and_then(|v| <Ipv6Addr as Decode<M>>::decode(cx, v))?;
        let port = unpack
            .next(cx)
            .and_then(|v| <u16 as Decode<M>>::decode(cx, v))?;
        let flowinfo = unpack
            .next(cx)
            .and_then(|v| <u32 as Decode<M>>::decode(cx, v))?;
        let scope_id = unpack
            .next(cx)
            .and_then(|v| <u32 as Decode<M>>::decode(cx, v))?;
        unpack.end(cx)?;
        Ok(Self::new(ip, port, flowinfo, scope_id))
    }
}

impl<M> Encode<M> for SocketAddr
where
    M: Mode,
{
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Input = E::Error>,
        E: Encoder,
    {
        let variant = encoder.encode_variant(cx)?;

        match self {
            SocketAddr::V4(v4) => variant.insert::<M, _, _, _>(cx, 0usize, v4),
            SocketAddr::V6(v6) => variant.insert::<M, _, _, _>(cx, 1usize, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for SocketAddr
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant(cx)?;

        let this = match variant
            .tag(cx)
            .and_then(|v| <usize as Decode<M>>::decode(cx, v))?
        {
            0 => Self::V4(
                variant
                    .variant(cx)
                    .and_then(|v| <SocketAddrV4 as Decode<M>>::decode(cx, v))?,
            ),
            1 => Self::V6(
                variant
                    .variant(cx)
                    .and_then(|v| <SocketAddrV6 as Decode<M>>::decode(cx, v))?,
            ),
            index => {
                return Err(cx.invalid_variant_tag("SocketAddr", index));
            }
        };

        variant.end(cx)?;
        Ok(this)
    }
}
