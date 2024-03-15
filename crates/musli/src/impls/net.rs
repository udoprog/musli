use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::de::{Decode, Decoder, PackDecoder, VariantDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder, VariantEncoder};
use crate::Context;

impl<M> Encode<M> for Ipv4Addr {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M, Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_array(cx, self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv4Addr {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_array::<C, 4>(cx).map(Ipv4Addr::from)
    }
}

impl<M> Encode<M> for Ipv6Addr {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M, Input = E::Error>,
        E: Encoder,
    {
        encoder.encode_array(cx, self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv6Addr {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_array::<C, 16>(cx).map(Ipv6Addr::from)
    }
}

#[derive(Encode, Decode)]
#[musli(crate)]
enum IpAddrTag {
    Ipv4,
    Ipv6,
}

impl<M> Encode<M> for IpAddr {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M, Input = E::Error>,
        E: Encoder,
    {
        let variant = encoder.encode_variant(cx)?;

        match self {
            IpAddr::V4(v4) => variant.insert_variant(cx, IpAddrTag::Ipv4, v4),
            IpAddr::V6(v6) => variant.insert_variant(cx, IpAddrTag::Ipv6, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for IpAddr {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant(cx)?;

        let tag: IpAddrTag = variant.tag(cx).and_then(|v| cx.decode(v))?;

        let this = match tag {
            IpAddrTag::Ipv4 => Self::V4(cx.decode(variant.variant(cx)?)?),
            IpAddrTag::Ipv6 => Self::V6(cx.decode(variant.variant(cx)?)?),
        };

        variant.end(cx)?;
        Ok(this)
    }
}

impl<M> Encode<M> for SocketAddrV4 {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M, Input = E::Error>,
        E: Encoder,
    {
        let mut pack = encoder.encode_pack(cx)?;
        pack.push(cx, self.ip())?;
        pack.push(cx, self.port())?;
        pack.end(cx)
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV4 {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack(cx)?;
        let ip = unpack.next(cx).and_then(|v| cx.decode(v))?;
        let port = unpack.next(cx).and_then(|v| cx.decode(v))?;
        unpack.end(cx)?;
        Ok(SocketAddrV4::new(ip, port))
    }
}

impl<M> Encode<M> for SocketAddrV6 {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M, Input = E::Error>,
        E: Encoder,
    {
        let mut pack = encoder.encode_pack(cx)?;
        pack.push(cx, self.ip())?;
        pack.push(cx, self.port())?;
        pack.push(cx, self.flowinfo())?;
        pack.push(cx, self.scope_id())?;
        pack.end(cx)
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV6 {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack(cx)?;
        let ip = unpack.next(cx).and_then(|v| cx.decode(v))?;
        let port = unpack.next(cx).and_then(|v| cx.decode(v))?;
        let flowinfo = unpack.next(cx).and_then(|v| cx.decode(v))?;
        let scope_id = unpack.next(cx).and_then(|v| cx.decode(v))?;
        unpack.end(cx)?;
        Ok(Self::new(ip, port, flowinfo, scope_id))
    }
}

#[derive(Encode, Decode)]
#[musli(crate)]
enum SocketAddrTag {
    V4,
    V6,
}

impl<M> Encode<M> for SocketAddr {
    #[inline]
    fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<Mode = M, Input = E::Error>,
        E: Encoder,
    {
        let variant = encoder.encode_variant(cx)?;

        match self {
            SocketAddr::V4(v4) => variant.insert_variant(cx, SocketAddrTag::V4, v4),
            SocketAddr::V6(v6) => variant.insert_variant(cx, SocketAddrTag::V6, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for SocketAddr {
    #[inline]
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<Mode = M, Input = D::Error>,
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant(cx)?;

        let tag: SocketAddrTag = cx.decode(variant.tag(cx)?)?;

        let this = match tag {
            SocketAddrTag::V4 => Self::V4(cx.decode(variant.variant(cx)?)?),
            SocketAddrTag::V6 => Self::V6(cx.decode(variant.variant(cx)?)?),
        };

        variant.end(cx)?;
        Ok(this)
    }
}
