use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

/// the address of any given nodes
///
/// The underlying object is, for now, a [`Multiaddr`] to allow
/// compatibility with the different type of network and identification.
///
/// [`Multiaddr`]: https://docs.rs/parity-multiaddr/0.9/parity_multiaddr/struct.Multiaddr.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "Multiaddr")]
pub struct Address(Multiaddr);

impl Address {
    pub fn to_socket_addr(&self) -> Option<SocketAddr> {
        use multiaddr::Protocol::*;

        let mut iter = self.0.iter();
        match iter.next()? {
            Ip4(ipv4) => {
                if let Tcp(port) = iter.next()? {
                    Some(SocketAddr::V4(SocketAddrV4::new(
                        ipv4, port,
                    )))
                } else {
                    None
                }
            }
            Ip6(ipv6) => {
                if let Tcp(port) = iter.next()? {
                    Some(SocketAddr::V6(SocketAddrV6::new(
                        ipv6, port, 0, 0,
                    )))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn multiaddr(&self) -> &Multiaddr {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl From<IpAddr> for Address {
    fn from(addr: IpAddr) -> Self {
        Address(addr.into())
    }
}

impl From<Ipv4Addr> for Address {
    fn from(addr: Ipv4Addr) -> Self {
        Address(addr.into())
    }
}

impl From<Ipv6Addr> for Address {
    fn from(addr: Ipv6Addr) -> Self {
        Address(addr.into())
    }
}

impl From<Address> for Multiaddr {
    #[inline]
    fn from(address: Address) -> Self {
        address.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddressTryFromError {
    #[error("Invalid address format: {source}")]
    Multiaddr {
        #[from]
        #[source]
        source: multiaddr::Error,
    },

    #[error("Invalid address format, rejecting non ip4 or ip6")]
    UnsupportedProtocol,
}

impl TryFrom<Multiaddr> for Address {
    type Error = AddressTryFromError;
    fn try_from(multiaddr: Multiaddr) -> Result<Self, Self::Error> {
        use multiaddr::Protocol::*;

        match multiaddr.iter().next() {
            Some(Ip4(_)) | Some(Ip6(_)) => Ok(Address(multiaddr)),
            _ => Err(AddressTryFromError::UnsupportedProtocol),
        }
    }
}

impl std::str::FromStr for Address {
    type Err = AddressTryFromError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let multiaddr = s.parse::<Multiaddr>()?;
        Address::try_from(multiaddr)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
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
                "Invalid address format, rejecting non ip4 or ip6"
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
