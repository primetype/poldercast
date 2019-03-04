use std::fmt;

use multiaddr::{self, Multiaddr, ToMultiaddr};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// the address of any given nodes
///
/// The underlying object is, for now, a [`Multiaddr`] to allow
/// compatibility with the different type of network and identification.
///
/// [`Multiaddr`]: https://docs.rs/multiaddr/latest/multiaddr/struct.Multiaddr.html
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Address(Multiaddr);

impl Address {
    /// create a new address from any given [`ToMultiaddr`] implementors
    ///
    /// # Examples
    ///
    /// TODO: simple example with SockAddr from standard lib
    /// TODO: simple example with a multiaddr object and a string
    ///
    /// TODO: [`ToMultiaddr`]
    pub fn new<Addr>(addr: Addr) -> Result<Self, multiaddr::Error>
    where
        Addr: ToMultiaddr,
    {
        addr.to_multiaddr().map(Address)
    }

    pub fn to_socketaddr(&self) -> Option<std::net::SocketAddr> {
        let components = self.0.iter().collect::<Vec<_>>();

        match components.get(0)? {
            multiaddr::AddrComponent::IP4(ipv4) => {
                if let multiaddr::AddrComponent::TCP(port) = components.get(1)? {
                    Some(std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
                        *ipv4, *port,
                    )))
                } else {
                    None
                }
            }
            multiaddr::AddrComponent::IP6(ipv6) => {
                if let multiaddr::AddrComponent::TCP(port) = components.get(1)? {
                    Some(std::net::SocketAddr::V6(std::net::SocketAddrV6::new(
                        *ipv6, *port, 0, 0,
                    )))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl From<Multiaddr> for Address {
    fn from(multiaddr: Multiaddr) -> Self {
        Address(multiaddr)
    }
}

impl std::str::FromStr for Address {
    type Err = <Multiaddr as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Address)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("{}", self))
        } else {
            serializer.serialize_bytes(&self.0.to_bytes())
        }
    }
}
impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Unexpected, Visitor};
        use std::fmt::Formatter;

        struct AddressVisitor;
        impl<'de> Visitor<'de> for AddressVisitor {
            type Value = Address;

            fn expecting(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
                write!(
                    f,
                    "expected a multi format address (e.g.: `/ip4/127.0.0.1/tcp/8081')"
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match s.parse() {
                    Ok(address) => Ok(Address(address)),
                    Err(_err) => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
                }
            }
            fn visit_bytes<E>(self, s: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match multiaddr::Multiaddr::from_bytes(s.to_owned()) {
                    Ok(address) => Ok(Address(address)),
                    Err(_err) => Err(de::Error::invalid_value(Unexpected::Bytes(s), &self)),
                }
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(AddressVisitor)
        } else {
            deserializer.deserialize_bytes(AddressVisitor)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Address {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let ip: (u8, u8, u8, u8) = Arbitrary::arbitrary(g);
            let port: u16 = Arbitrary::arbitrary(g);
            let address = format!("/ip4/{}.{}.{}.{}/tcp/{}", ip.0, ip.1, ip.2, ip.3, port);
            address.parse().unwrap()
        }
    }

    quickcheck! {
        fn encode_decode_json(address: Address) -> bool {
            let encoded = serde_json::to_string(&address).unwrap();
            let decoded : Address = serde_json::from_str(&encoded).unwrap();
            decoded == address
        }
        fn encode_decode_bincode(address: Address) -> bool {
            let encoded = bincode::serialize(&address).unwrap();
            let decoded : Address = bincode::deserialize(&encoded).unwrap();
            decoded == address
        }
    }
}
