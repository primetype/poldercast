use multiaddr::{self, Multiaddr, ToMultiaddr};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cmp::{Ord, Ordering},
    fmt,
    net::SocketAddr,
};
use thiserror::Error;

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

    #[deprecated(
        since = "0.12.0",
        note = "Use the `multi_address` function instead, this function will be removed"
    )]
    pub fn to_socketaddr(&self) -> Option<SocketAddr> {
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

    pub fn multi_address(&self) -> &Multiaddr {
        &self.0
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<Multiaddr> for Address {
    fn from(multiaddr: Multiaddr) -> Self {
        Address(multiaddr)
    }
}

impl From<Address> for Multiaddr {
    #[inline]
    fn from(address: Address) -> Self {
        address.0
    }
}

#[derive(Debug, Error)]
pub enum FromStrAddressError {
    #[error("Invalid address format: {source}")]
    Multiaddr {
        #[from]
        #[source]
        source: multiaddr::Error,
    },

    #[error("Invalid address format, rejecting non ip4 or ip6")]
    RejectingNonIp4OrIp6thread,
}

impl std::str::FromStr for Address {
    type Err = FromStrAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr = s.parse().map(Address)?;

        match addr.0.iter().next() {
            Some(multiaddr::AddrComponent::IP4(_)) => Ok(addr),
            Some(multiaddr::AddrComponent::IP6(_)) => Ok(addr),
            _ => Err(FromStrAddressError::RejectingNonIp4OrIp6thread),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.0.as_slice(), rhs.0.as_slice())
    }
}
impl Ord for Address {
    fn cmp(&self, rhs: &Self) -> Ordering {
        Ord::cmp(self.0.as_slice(), rhs.0.as_slice())
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
            serializer.serialize_bytes(&self.as_slice())
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
                s.parse::<Address>()
                    .map_err(|_err| de::Error::invalid_value(Unexpected::Str(s), &self))
            }
            fn visit_bytes<E>(self, s: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match multiaddr::Multiaddr::from_bytes(s.to_owned()) {
                    Ok(address) => Ok(Address::from(address)),
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

    #[test]
    fn reject_non_ip4_ip6_from_str() {
        const INVALID_STR: &str = "/dns4/example.com/tcp/2901";

        let parsed: Result<Address, _> = INVALID_STR.parse();

        if let Err(err) = parsed {
            assert_eq!(
                err.to_string(),
                "Invalid address format, rejecting non ip4 or ip6"
            )
        } else {
            panic!("Expecting {:?} to fail parsing FromStr", INVALID_STR)
        }
    }

    #[test]
    fn reject_non_ip4_ip6_json() {
        const INVALID_STR: &str = "\"/dns6/example.com/tcp/2901\"";

        let parsed: Result<Address, _> = serde_json::from_str(INVALID_STR);

        if let Err(err) = parsed {
            assert_eq!(
                err.to_string(),
                "invalid value: string \"/dns6/example.com/tcp/2901\", expected expected a multi format address (e.g.: `/ip4/127.0.0.1/tcp/8081\') at line 1 column 28"
            )
        } else {
            panic!(
                "Expecting {:?} to fail parsing from serde_json",
                INVALID_STR
            )
        }
    }

    #[quickcheck]
    fn address_encode_decode_json(address: Address) -> bool {
        let encoded = serde_json::to_string(&address).unwrap();
        let decoded: Address = serde_json::from_str(&encoded).unwrap();
        decoded == address
    }

    #[quickcheck]
    fn address_encode_decode_bincode(address: Address) -> bool {
        let encoded = bincode::serialize(&address).unwrap();
        let decoded: Address = bincode::deserialize(&encoded).unwrap();
        decoded == address
    }
}
