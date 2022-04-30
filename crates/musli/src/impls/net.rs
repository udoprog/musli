use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::de::{Decode, Decoder, PackDecoder, PairDecoder};
use crate::en::{Encode, Encoder, PairEncoder, SequenceEncoder};
use crate::error::Error;

impl<Mode> Encode<Mode> for Ipv4Addr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.octets())
    }
}

impl<'de, Mode> Decode<'de, Mode> for Ipv4Addr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<4>().map(Ipv4Addr::from)
    }
}

impl<Mode> Encode<Mode> for Ipv6Addr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.octets())
    }
}

impl<'de, Mode> Decode<'de, Mode> for Ipv6Addr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<16>().map(Ipv6Addr::from)
    }
}

impl<Mode> Encode<Mode> for IpAddr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let variant = encoder.encode_variant()?;

        match self {
            IpAddr::V4(v4) => variant.insert::<Mode, _, _>(0usize, v4),
            IpAddr::V6(v6) => variant.insert::<Mode, _, _>(0usize, v6),
        }
    }
}

impl<'de, Mode> Decode<'de, Mode> for IpAddr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        Ok(
            match variant.first().and_then(<usize as Decode<Mode>>::decode)? {
                0 => Self::V4(
                    variant
                        .second()
                        .and_then(<Ipv4Addr as Decode<Mode>>::decode)?,
                ),
                1 => Self::V6(
                    variant
                        .second()
                        .and_then(<Ipv6Addr as Decode<Mode>>::decode)?,
                ),
                index => {
                    return Err(<D::Error as Error>::invalid_variant_tag("IpAddr", index));
                }
            },
        )
    }
}

impl<Mode> Encode<Mode> for SocketAddrV4 {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let mut pack = encoder.encode_pack()?;
        pack.push::<Mode, _>(self.ip())?;
        pack.push::<Mode, _>(self.port())?;
        pack.end()
    }
}

impl<'de, Mode> Decode<'de, Mode> for SocketAddrV4 {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack()?;
        let ip = unpack.next().and_then(<Ipv4Addr as Decode<Mode>>::decode)?;
        let port = unpack.next().and_then(<u16 as Decode<Mode>>::decode)?;
        Ok(SocketAddrV4::new(ip, port))
    }
}

impl<Mode> Encode<Mode> for SocketAddrV6 {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let mut pack = encoder.encode_pack()?;
        pack.push::<Mode, _>(self.ip())?;
        pack.push::<Mode, _>(self.port())?;
        pack.push::<Mode, _>(self.flowinfo())?;
        pack.push::<Mode, _>(self.scope_id())?;
        pack.end()
    }
}

impl<'de, Mode> Decode<'de, Mode> for SocketAddrV6 {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack()?;
        let ip = unpack.next().and_then(<Ipv6Addr as Decode<Mode>>::decode)?;
        let port = unpack.next().and_then(<u16 as Decode<Mode>>::decode)?;
        let flowinfo = unpack.next().and_then(<u32 as Decode<Mode>>::decode)?;
        let scope_id = unpack.next().and_then(<u32 as Decode<Mode>>::decode)?;
        Ok(Self::new(ip, port, flowinfo, scope_id))
    }
}

impl<Mode> Encode<Mode> for SocketAddr {
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let variant = encoder.encode_variant()?;

        match self {
            SocketAddr::V4(v4) => variant.insert::<Mode, _, _>(0usize, v4),
            SocketAddr::V6(v6) => variant.insert::<Mode, _, _>(1usize, v6),
        }
    }
}

impl<'de, Mode> Decode<'de, Mode> for SocketAddr {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut variant = decoder.decode_variant()?;

        Ok(
            match variant.first().and_then(<usize as Decode<Mode>>::decode)? {
                0 => Self::V4(
                    variant
                        .second()
                        .and_then(<SocketAddrV4 as Decode<Mode>>::decode)?,
                ),
                1 => Self::V6(
                    variant
                        .second()
                        .and_then(<SocketAddrV6 as Decode<Mode>>::decode)?,
                ),
                index => {
                    return Err(<D::Error as Error>::invalid_variant_tag(
                        "SocketAddr",
                        index,
                    ));
                }
            },
        )
    }
}
