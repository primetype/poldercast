use cryptoxide::ed25519;
use rand_core::{CryptoRng, RngCore};
#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};
use std::fmt;

/// this is a poldercast unique identifier.
///
/// It can be used for different purposes:
///
/// * the unique identifier the a node identifies itself within the topology;
/// * a mean to compare 2 nodes within the topology
///
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct Id([u8; ID_LEN]); // TODO: PublicKey

#[derive(Clone)]
pub struct PrivateId([u8; ed25519::SEED_LENGTH]);

pub const ID_LEN: usize = ed25519::PUBLIC_KEY_LENGTH;

impl PrivateId {
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut seed = [0u8; ed25519::SEED_LENGTH];
        rng.fill_bytes(&mut seed);
        Self::from(seed)
    }

    pub fn id(&self) -> Id {
        let (_, pk) = ed25519::keypair(&self.0);
        Id::from(pk)
    }
}

impl Id {
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut id = Self::zero();
        rng.fill_bytes(&mut id.0);
        id
    }

    pub(crate) fn zero() -> Self {
        Id([0; ID_LEN])
    }
}

impl From<[u8; ed25519::SEED_LENGTH]> for PrivateId {
    fn from(seed: [u8; ed25519::SEED_LENGTH]) -> Self {
        PrivateId(seed)
    }
}

impl From<[u8; ID_LEN]> for Id {
    fn from(v: [u8; ID_LEN]) -> Self {
        Id(v)
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        hex::encode(self).fmt(f)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        hex::encode(self).fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for PrivateId {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes = [0; ed25519::SEED_LENGTH];
            PrivateId::from(bytes)
        }
    }

    impl Arbitrary for Id {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes = [0; ID_LEN];
            Id::from(bytes)
        }
    }
}
