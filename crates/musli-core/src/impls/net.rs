use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;

use crate::context::Context;
use crate::de::{Decode, Decoder, SequenceDecoder, VariantDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder, VariantEncoder};
use crate::mode::{Binary, Text};

#[derive(Encode, Decode)]
#[musli(crate)]
#[musli(mode = Text, name_all = "kebab-case")]
enum IpAddrTag {
    Ipv4,
    Ipv6,
}

#[derive(Encode, Decode)]
#[musli(crate)]
#[musli(mode = Text, name_all = "kebab-case")]
enum SocketAddrTag {
    V4,
    V6,
}

impl Encode<Binary> for Ipv4Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(&self.octets())
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl Encode<Text> for Ipv4Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.collect_string(self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de> Decode<'de, Binary> for Ipv4Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<4>().map(Ipv4Addr::from)
    }
}

impl<'de> Decode<'de, Text> for Ipv4Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let cx = decoder.cx();
        decoder.decode_unsized(|string: &str| Ipv4Addr::from_str(string).map_err(cx.map()))
    }
}

impl Encode<Binary> for Ipv6Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = Binary>,
    {
        encoder.encode_array(&self.octets())
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl Encode<Text> for Ipv6Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = Text>,
    {
        encoder.collect_string(self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de> Decode<'de, Binary> for Ipv6Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<16>().map(Ipv6Addr::from)
    }
}

impl<'de> Decode<'de, Text> for Ipv6Addr {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let cx = decoder.cx();
        decoder.decode_unsized(|string: &str| Ipv6Addr::from_str(string).map_err(cx.map()))
    }
}

impl<M> Encode<M> for IpAddr
where
    IpAddrTag: Encode<M>,
    Ipv4Addr: Encode<M>,
    Ipv6Addr: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let variant = encoder.encode_variant()?;

        match self {
            IpAddr::V4(v4) => variant.insert_variant(&IpAddrTag::Ipv4, v4),
            IpAddr::V6(v6) => variant.insert_variant(&IpAddrTag::Ipv6, v6),
        }
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M> Decode<'de, M> for IpAddr
where
    IpAddrTag: Decode<'de, M>,
    Ipv4Addr: Decode<'de, M>,
    Ipv6Addr: Decode<'de, M>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
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

impl Encode<Binary> for SocketAddrV4 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = Binary>,
    {
        encoder.encode_pack_fn(|pack| {
            pack.push(self.ip())?;
            pack.push(self.port())?;
            Ok(())
        })
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl Encode<Text> for SocketAddrV4 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = Text>,
    {
        encoder.collect_string(self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de> Decode<'de, Binary> for SocketAddrV4 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = Binary>,
    {
        decoder.decode_pack(|p| Ok(SocketAddrV4::new(p.next()?, p.next()?)))
    }
}

impl<'de> Decode<'de, Text> for SocketAddrV4 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let cx = decoder.cx();
        decoder.decode_unsized(|string: &str| SocketAddrV4::from_str(string).map_err(cx.map()))
    }
}

impl Encode<Binary> for SocketAddrV6 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = Binary>,
    {
        encoder.encode_pack_fn(|pack| {
            pack.push(self.ip())?;
            pack.push(self.port())?;
            pack.push(self.flowinfo())?;
            pack.push(self.scope_id())?;
            Ok(())
        })
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl Encode<Text> for SocketAddrV6 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = Text>,
    {
        encoder.collect_string(self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de> Decode<'de, Binary> for SocketAddrV6 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = Binary>,
    {
        decoder.decode_pack(|p| Ok(Self::new(p.next()?, p.next()?, p.next()?, p.next()?)))
    }
}

impl<'de> Decode<'de, Text> for SocketAddrV6 {
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let cx = decoder.cx();
        decoder.decode_unsized(|string: &str| SocketAddrV6::from_str(string).map_err(cx.map()))
    }
}

impl<M> Encode<M> for SocketAddr
where
    SocketAddrTag: Encode<M>,
    SocketAddrV4: Encode<M>,
    SocketAddrV6: Encode<M>,
{
    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        let variant = encoder.encode_variant()?;

        match self {
            SocketAddr::V4(v4) => variant.insert_variant(&SocketAddrTag::V4, v4),
            SocketAddr::V6(v6) => variant.insert_variant(&SocketAddrTag::V6, v6),
        }
    }

    type Encode = Self;

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M> Decode<'de, M> for SocketAddr
where
    SocketAddrTag: Decode<'de, M>,
    SocketAddrV4: Decode<'de, M>,
    SocketAddrV6: Decode<'de, M>,
{
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
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

#[cfg(test)]
mod tests {
    use crate::{Decode, Encode};

    use std::net::{IpAddr, SocketAddr};

    #[derive(Encode, Decode)]
    #[musli(crate)]
    #[allow(dead_code)]
    struct Container {
        ip_addr: IpAddr,
        sock_addr: SocketAddr,
    }
}
