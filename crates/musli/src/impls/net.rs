use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::de::{Decode, Decoder, SequenceDecoder, VariantDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder, VariantEncoder};

impl<M> Encode<M> for Ipv4Addr {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(&self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv4Addr {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<4>().map(Ipv4Addr::from)
    }
}

impl<M> Encode<M> for Ipv6Addr {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(&self.octets())
    }
}

impl<'de, M> Decode<'de, M> for Ipv6Addr {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<16>().map(Ipv6Addr::from)
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
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let variant = encoder.encode_variant()?;

        match self {
            IpAddr::V4(v4) => variant.insert_variant(IpAddrTag::Ipv4, v4),
            IpAddr::V6(v6) => variant.insert_variant(IpAddrTag::Ipv6, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for IpAddr {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_variant(|variant| {
            let tag = variant.decode_tag()?.decode()?;

            Ok(match tag {
                IpAddrTag::Ipv4 => Self::V4(variant.decode_value()?.decode()?),
                IpAddrTag::Ipv6 => Self::V6(variant.decode_value()?.decode()?),
            })
        })
    }
}

impl<M> Encode<M> for SocketAddrV4 {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_pack_fn(|pack| {
            pack.push(self.ip())?;
            pack.push(self.port())?;
            Ok(())
        })
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV4 {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_pack(|p| Ok(SocketAddrV4::new(p.next()?, p.next()?)))
    }
}

impl<M> Encode<M> for SocketAddrV6 {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_pack_fn(|pack| {
            pack.push(self.ip())?;
            pack.push(self.port())?;
            pack.push(self.flowinfo())?;
            pack.push(self.scope_id())?;
            Ok(())
        })
    }
}

impl<'de, M> Decode<'de, M> for SocketAddrV6 {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_pack(|p| Ok(Self::new(p.next()?, p.next()?, p.next()?, p.next()?)))
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
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let variant = encoder.encode_variant()?;

        match self {
            SocketAddr::V4(v4) => variant.insert_variant(SocketAddrTag::V4, v4),
            SocketAddr::V6(v6) => variant.insert_variant(SocketAddrTag::V6, v6),
        }
    }
}

impl<'de, M> Decode<'de, M> for SocketAddr {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_variant(|variant| {
            let tag = variant.decode_tag()?.decode()?;

            Ok(match tag {
                SocketAddrTag::V4 => Self::V4(variant.decode_value()?.decode()?),
                SocketAddrTag::V6 => Self::V6(variant.decode_value()?.decode()?),
            })
        })
    }
}
