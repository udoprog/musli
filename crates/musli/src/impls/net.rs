use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::de::{Decode, Decoder, PackDecoder, PairDecoder};
use crate::en::{Encode, Encoder, PackEncoder, VariantEncoder};
use crate::error::Error;

impl Encode for Ipv4Addr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.octets())
    }
}

impl<'de> Decode<'de> for Ipv4Addr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<4>().map(Ipv4Addr::from)
    }
}

impl Encode for Ipv6Addr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.octets())
    }
}

impl<'de> Decode<'de> for Ipv6Addr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<16>().map(Ipv6Addr::from)
    }
}

impl Encode for IpAddr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self {
            IpAddr::V4(v4) => {
                let mut variant = encoder.encode_variant()?;
                usize::encode(&0, variant.encode_variant_tag()?)?;
                v4.encode(variant.encode_variant_value()?)?;
                Ok(())
            }
            IpAddr::V6(v6) => {
                let mut variant = encoder.encode_variant()?;
                usize::encode(&1, variant.encode_variant_tag()?)?;
                v6.encode(variant.encode_variant_value()?)?;
                Ok(())
            }
        }
    }
}

impl<'de> Decode<'de> for IpAddr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        Ok(match variant.decode_first().and_then(usize::decode)? {
            0 => Self::V4(variant.decode_second().and_then(Ipv4Addr::decode)?),
            1 => Self::V6(variant.decode_second().and_then(Ipv6Addr::decode)?),
            index => {
                return Err(<D::Error as Error>::unsupported_variant("IpAddr", index));
            }
        })
    }
}

impl Encode for SocketAddrV4 {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        let mut pack = encoder.encode_pack()?;
        pack.push(self.ip())?;
        pack.push(self.port())?;
        pack.finish()
    }
}

impl<'de> Decode<'de> for SocketAddrV4 {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack()?;
        let ip = unpack.next().and_then(Ipv4Addr::decode)?;
        let port = unpack.next().and_then(u16::decode)?;
        Ok(SocketAddrV4::new(ip, port))
    }
}

impl Encode for SocketAddrV6 {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        let mut pack = encoder.encode_pack()?;
        pack.push(self.ip())?;
        pack.push(self.port())?;
        pack.push(self.flowinfo())?;
        pack.push(self.scope_id())?;
        Ok(())
    }
}

impl<'de> Decode<'de> for SocketAddrV6 {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack()?;
        let ip = unpack.next().and_then(Ipv6Addr::decode)?;
        let port = unpack.next().and_then(u16::decode)?;
        let flowinfo = unpack.next().and_then(u32::decode)?;
        let scope_id = unpack.next().and_then(u32::decode)?;
        Ok(Self::new(ip, port, flowinfo, scope_id))
    }
}

impl Encode for SocketAddr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self {
            SocketAddr::V4(v4) => {
                let mut variant = encoder.encode_variant()?;
                usize::encode(&0, variant.encode_variant_tag()?)?;
                v4.encode(variant.encode_variant_value()?)?;
                Ok(())
            }
            SocketAddr::V6(v6) => {
                let mut variant = encoder.encode_variant()?;
                usize::encode(&1, variant.encode_variant_tag()?)?;
                v6.encode(variant.encode_variant_value()?)?;
                Ok(())
            }
        }
    }
}

impl<'de> Decode<'de> for SocketAddr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        Ok(match variant.decode_first().and_then(usize::decode)? {
            0 => Self::V4(variant.decode_second().and_then(SocketAddrV4::decode)?),
            1 => Self::V6(variant.decode_second().and_then(SocketAddrV6::decode)?),
            index => {
                return Err(<D::Error as Error>::unsupported_variant(
                    "SocketAddr",
                    index,
                ));
            }
        })
    }
}
