use crate::{
    Subscription, SubscriptionError, SubscriptionSlice, Subscriptions, SubscriptionsSlice,
};
use keynesis::{key::ed25519, passport::block::Time};
use std::{
    convert::TryInto as _,
    fmt::{self, Formatter},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};
use thiserror::Error;

const INFO_INDEX: usize = 0;
const INFO_END: usize = INFO_INDEX + GossipInfo::SIZE;
const ID_INDEX: usize = INFO_END;
const ID_END: usize = ID_INDEX + ed25519::PublicKey::SIZE;
const TIME_INDEX: usize = ID_END;
const TIME_END: usize = TIME_INDEX + Time::SIZE;

const IPV4_INDEX: usize = TIME_END;
const IPV4_END: usize = IPV4_INDEX + 4;

const IPV6_INDEX: usize = TIME_END;
const IPV6_END: usize = IPV6_INDEX + 16;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct GossipInfo(u16);

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Gossip(Vec<u8>);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct GossipSlice<'a>(&'a [u8]);

#[derive(Debug, Error)]
pub enum GossipError {
    #[error("Invalid gossip size, expected at least {min}")]
    InvalidSize { min: usize, max: Option<usize> },

    #[error("The signature does not match the public key and the content")]
    InvalidSignature,

    #[error("Invalid subscription ({index}): {error}")]
    InvalidSubscription {
        index: usize,
        error: SubscriptionError,
    },
}

impl GossipInfo {
    const SIZE: usize = std::mem::size_of::<u16>();

    fn try_from_slice(slice: &[u8]) -> Result<Self, GossipError> {
        if slice.len() < Self::SIZE {
            return Err(GossipError::InvalidSize {
                min: Self::SIZE,
                max: None,
            });
        }

        let sub = u16::from_be_bytes(
            slice[..Self::SIZE]
                .try_into()
                .expect("valid 2 bytes on the slice"),
        );
        Ok(Self(sub))
    }

    fn set_num_subscriptions(&mut self, num: usize) {
        let num = num & Subscriptions::MAX_NUM_SUBSCRIPTIONS;
        self.0 &= !(Subscriptions::MAX_NUM_SUBSCRIPTIONS as u16);
        self.0 &= num as u16;
    }

    fn set_ipv4(&mut self) {
        self.0 |= 0b1000_0000_0000_0000;
    }

    fn set_ipv6(&mut self) {
        self.0 &= 0b0111_1111_1111_1111;
    }

    #[inline(always)]
    fn is_ipv4(&self) -> bool {
        self.0 & 0b1000_0000_0000_0000 == 0b1000_0000_0000_0000
    }

    #[inline(always)]
    fn is_ipv6(&self) -> bool {
        !self.is_ipv4()
    }

    #[inline(always)]
    fn num_subscriptions(&self) -> usize {
        (self.0 & 0b0000_0011_1111_1111) as usize
    }

    fn signature_start(&self) -> usize {
        let subs_start = if self.is_ipv4() { 4 + 2 } else { 16 + 2 };
        let num_subs = self.num_subscriptions();

        subs_start + (num_subs * Subscription::SIZE)
    }

    fn signature_end(&self) -> usize {
        let subs_start = if self.is_ipv4() { 4 + 2 } else { 16 + 2 };
        let num_subs = self.num_subscriptions();

        subs_start + (num_subs * Subscription::SIZE) + ed25519::Signature::SIZE
    }
}

impl Gossip {
    pub const MAX_NUM_SUBSCRIPTIONS: usize = Subscriptions::MAX_NUM_SUBSCRIPTIONS;
    pub const MIN_SIZE: usize =
        IPV4_END + ed25519::Signature::SIZE + Self::MAX_NUM_SUBSCRIPTIONS * Subscription::SIZE;
    pub const MAX_SIZE: usize =
        IPV6_END + ed25519::Signature::SIZE + Self::MAX_NUM_SUBSCRIPTIONS * Subscription::SIZE;

    /// prepare a gossip without our address and public key
    pub fn new(
        address: SocketAddr,
        id: &ed25519::SecretKey,
        subscriptions: SubscriptionsSlice<'_>,
    ) -> Self {
        let mut bytes = vec![0; Self::MAX_SIZE];
        let mut info = GossipInfo(0);
        info.set_num_subscriptions(subscriptions.number_subscriptions());
        if address.is_ipv4() {
            info.set_ipv4()
        } else if address.is_ipv6() {
            info.set_ipv6()
        }

        bytes[INFO_INDEX..INFO_END].copy_from_slice(&info.0.to_be_bytes());
        bytes[ID_INDEX..ID_END].copy_from_slice(id.public_key().as_ref());
        bytes[TIME_INDEX..TIME_END].copy_from_slice(&Time::now().to_be_bytes());

        let (port_index, port_end) = match address.ip() {
            IpAddr::V4(v4) => {
                let ip = v4.octets();
                bytes[IPV4_INDEX..IPV4_END].copy_from_slice(&ip);
                (IPV4_END, IPV4_END + 2)
            }
            IpAddr::V6(v6) => {
                let ip = v6.octets();
                bytes[IPV6_INDEX..IPV6_END].copy_from_slice(&ip);
                (IPV6_END, IPV6_END + 2)
            }
        };
        bytes[port_index..port_end].copy_from_slice(&address.port().to_be_bytes());
        bytes[port_end..port_end + subscriptions.as_ref().len()]
            .copy_from_slice(subscriptions.as_ref());

        let signature_start = info.signature_start();
        let signature_end = info.signature_end();
        let signature = id.sign(&bytes[..signature_start]);
        bytes[signature_start..signature_end].copy_from_slice(signature.as_ref());

        bytes.resize(signature_end, 0);
        Self(bytes)
    }

    pub fn as_slice(&self) -> GossipSlice<'_> {
        GossipSlice(&self.0)
    }

    pub fn id(&self) -> ed25519::PublicKey {
        self.as_slice().id()
    }

    pub fn time(&self) -> Time {
        self.as_slice().time()
    }

    pub fn address(&self) -> SocketAddr {
        self.as_slice().address()
    }

    pub fn subscriptions(&self) -> SubscriptionsSlice<'_> {
        self.as_slice().subscriptions()
    }

    pub fn signature(&self) -> ed25519::Signature {
        self.as_slice().signature()
    }
}

impl<'a> GossipSlice<'a> {
    pub fn try_from_slice(slice: &'a [u8]) -> Result<Self, GossipError> {
        let info = GossipInfo::try_from_slice(slice)?;

        if info.signature_end() != slice.len() {
            return Err(GossipError::InvalidSize {
                min: info.signature_end(),
                max: Some(info.signature_end()),
            });
        }

        let gossip = Self::from_slice_unchecked(slice);

        for (index, sub) in gossip.subscriptions().iter().enumerate() {
            let slice = sub.as_ref();
            let _ = SubscriptionSlice::try_from_slice(slice)
                .map_err(|error| GossipError::InvalidSubscription { index, error })?;
        }

        let pk = gossip.id();
        let signature = gossip.signature();
        let signed_data = gossip.signed_data();

        if pk.verify(signed_data, &signature) {
            Err(GossipError::InvalidSignature)
        } else {
            Ok(Self(slice))
        }
    }

    pub fn from_slice_unchecked(slice: &'a [u8]) -> Self {
        #[cfg(debug_assertions)]
        {
            let info =
                GossipInfo::try_from_slice(slice).expect("should have the gossip info slice");
            debug_assert_eq!(info.signature_end(), slice.len());
        }

        Self(slice)
    }

    pub fn to_owned(self) -> Gossip {
        Gossip(self.0.to_owned())
    }

    fn info(&self) -> GossipInfo {
        let sub = u16::from_be_bytes(
            self.0[INFO_INDEX..INFO_END]
                .try_into()
                .expect("valid 2 bytes on the slice"),
        );
        GossipInfo(sub)
    }

    pub fn id(&self) -> ed25519::PublicKey {
        let pk: [u8; ed25519::PublicKey::SIZE] = self.0[ID_INDEX..ID_END]
            .try_into()
            .expect("valid public key");
        ed25519::PublicKey::from(pk)
    }

    pub fn time(&self) -> Time {
        let time = u32::from_be_bytes(
            self.0[TIME_INDEX..TIME_END]
                .try_into()
                .expect("Valid 4 bytes on the gossip"),
        );
        Time::from(time)
    }

    fn ipv4(&self) -> Ipv4Addr {
        let ip: [u8; 4] = self.0[IPV4_INDEX..IPV4_END]
            .try_into()
            .expect("bytes of the ipv4 address");
        Ipv4Addr::from(ip)
    }

    fn ipv6(&self) -> Ipv6Addr {
        let ip: [u8; 16] = self.0[IPV6_INDEX..IPV6_END]
            .try_into()
            .expect("bytes of the ipv6 address");
        Ipv6Addr::from(ip)
    }

    pub fn address(&self) -> SocketAddr {
        let info = self.info();

        let (ip, port_index, port_end) = if info.is_ipv4() {
            let ipv4 = self.ipv4();
            (IpAddr::V4(ipv4), IPV4_END, IPV4_END + 2)
        } else if info.is_ipv6() {
            let ipv6 = self.ipv6();
            (IpAddr::V6(ipv6), IPV6_END, IPV6_END + 2)
        } else {
            unreachable!("It should be either an IPv6 or IPv4")
        };
        let port = u16::from_be_bytes(
            self.0[port_index..port_end]
                .try_into()
                .expect("valid 2 bytes on the slice"),
        );
        SocketAddr::new(ip, port)
    }

    pub fn subscriptions(&self) -> SubscriptionsSlice<'a> {
        let info = self.info();

        let start_index = if info.is_ipv4() {
            IPV4_END + 2
        } else if info.is_ipv6() {
            IPV6_END + 2
        } else {
            unreachable!("It should be either an IPv6 or IPv4")
        };
        let slice =
            &self.0[start_index..start_index + (info.num_subscriptions() * Subscription::SIZE)];
        SubscriptionsSlice::from_slice_unchecked(slice)
    }

    fn signed_data(&self) -> &[u8] {
        let info = self.info();
        &self.0[..info.signature_start()]
    }

    pub fn signature(&self) -> ed25519::Signature {
        let info = self.info();
        let signature: [u8; ed25519::Signature::SIZE] = self.0
            [info.signature_start()..info.signature_end()]
            .try_into()
            .expect("64 bytes of the signature");
        signature.into()
    }
}

/* AsRef ******************************************************************* */

impl<'a> AsRef<[u8]> for GossipSlice<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl AsRef<[u8]> for Gossip {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/* Formatter *************************************************************** */

impl<'a> fmt::Debug for GossipSlice<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gossip")
            .field("id", &self.id())
            .field("time", &self.time())
            .field("address", &self.address())
            .field("subscriptions", &self.subscriptions())
            .field("signature", &self.signature())
            .finish()
    }
}

impl fmt::Debug for Gossip {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keynesis::Seed;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Gossip {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut seed = [0; Seed::SIZE];
            seed.iter_mut().for_each(|byte| {
                *byte = u8::arbitrary(g);
            });
            let mut rng = Seed::from(seed).into_rand_chacha();

            let address = SocketAddr::arbitrary(g);
            let id = ed25519::SecretKey::new(&mut rng);
            let subscriptions = Subscriptions::arbitrary(g);

            Self::new(address, &id, subscriptions.as_slice())
        }
    }

    #[test]
    fn simple_ipv4() {
        let mut rng = Seed::from([0; Seed::SIZE]).into_rand_chacha();
        let id = ed25519::SecretKey::new(&mut rng);

        let address: SocketAddr = "127.0.0.1:9876".parse().unwrap();
        let subscriptions = Subscriptions::new();

        let gossip = Gossip::new(address, &id, subscriptions.as_slice());

        let slice = gossip.as_slice();
        let decoded = GossipSlice::try_from_slice(slice.as_ref())
            .unwrap()
            .to_owned();

        assert_eq!(gossip.0, decoded.0);
        assert_eq!(decoded.address(), address);
    }

    #[test]
    fn simple_ipv6() {
        let mut rng = Seed::from([0; Seed::SIZE]).into_rand_chacha();
        let id = ed25519::SecretKey::new(&mut rng);

        let address: SocketAddr = "[::1]:9876".parse().unwrap();
        let subscriptions = Subscriptions::new();

        let gossip = Gossip::new(address, &id, subscriptions.as_slice());

        let slice = gossip.as_slice();
        let decoded = GossipSlice::try_from_slice(slice.as_ref())
            .unwrap()
            .to_owned();

        assert_eq!(gossip.0, decoded.0);
        assert_eq!(decoded.address(), address);
    }

    #[quickcheck]
    fn parse_valid_gossip(gossip: Gossip) -> bool {
        let slice = gossip.as_slice();
        let decoded = GossipSlice::try_from_slice(slice.as_ref())
            .unwrap()
            .to_owned();

        assert_eq!(gossip.0, decoded.0);
        true
    }
}
