use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::de::{Decode, Decoder, PackDecoder, VariantDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder, VariantEncoder};
use crate::error::Error;
use crate::mode::Mode;

impl<M> Encode<M> for Ipv4Addr
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv4Addr
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<4>().map(Ipv4Addr::from)
    }
}

impl<M> Encode<M> for Ipv6Addr
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv6Addr
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<16>().map(Ipv6Addr::from)
    }
}

impl<M> Encode<M> for IpAddr
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let variant = encoder.encode_variant()?;

        match self {
            IpAddr::V4(v4) => variant.insert::<M, _, _>(0usize, v4),
            IpAddr::V6(v6) => variant.insert::<M, _, _>(0usize, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for IpAddr
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        let this = match variant.tag().and_then(<usize as Decode<M>>::decode)? {
            0 => Self::V4(
                variant
                    .variant()
                    .and_then(<Ipv4Addr as Decode<M>>::decode)?,
            ),
            1 => Self::V6(
                variant
                    .variant()
                    .and_then(<Ipv6Addr as Decode<M>>::decode)?,
            ),
            index => {
                return Err(<D::Error as Error>::invalid_variant_tag("IpAddr", index));
            }
        };

        variant.end()?;
        Ok(this)
    }
}

impl<M> Encode<M> for SocketAddrV4
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let mut pack = encoder.encode_pack()?;
        pack.push::<M, _>(self.ip())?;
        pack.push::<M, _>(self.port())?;
        pack.end()
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV4
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack()?;
        let ip = unpack.next().and_then(<Ipv4Addr as Decode<M>>::decode)?;
        let port = unpack.next().and_then(<u16 as Decode<M>>::decode)?;
        Ok(SocketAddrV4::new(ip, port))
    }
}

impl<M> Encode<M> for SocketAddrV6
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let mut pack = encoder.encode_pack()?;
        pack.push::<M, _>(self.ip())?;
        pack.push::<M, _>(self.port())?;
        pack.push::<M, _>(self.flowinfo())?;
        pack.push::<M, _>(self.scope_id())?;
        pack.end()
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV6
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack()?;
        let ip = unpack.next().and_then(<Ipv6Addr as Decode<M>>::decode)?;
        let port = unpack.next().and_then(<u16 as Decode<M>>::decode)?;
        let flowinfo = unpack.next().and_then(<u32 as Decode<M>>::decode)?;
        let scope_id = unpack.next().and_then(<u32 as Decode<M>>::decode)?;
        Ok(Self::new(ip, port, flowinfo, scope_id))
    }
}

impl<M> Encode<M> for SocketAddr
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let variant = encoder.encode_variant()?;

        match self {
            SocketAddr::V4(v4) => variant.insert::<M, _, _>(0usize, v4),
            SocketAddr::V6(v6) => variant.insert::<M, _, _>(1usize, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for SocketAddr
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        let this = match variant.tag().and_then(<usize as Decode<M>>::decode)? {
            0 => Self::V4(
                variant
                    .variant()
                    .and_then(<SocketAddrV4 as Decode<M>>::decode)?,
            ),
            1 => Self::V6(
                variant
                    .variant()
                    .and_then(<SocketAddrV6 as Decode<M>>::decode)?,
            ),
            index => {
                return Err(<D::Error as Error>::invalid_variant_tag(
                    "SocketAddr",
                    index,
                ));
            }
        };

        variant.end()?;
        Ok(this)
    }
}
