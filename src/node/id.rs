use crate::node::Address;
use cryptoxide::{blake2b::Blake2b, digest::Digest};
#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};

/// this is a poldercast unique identifier.
///
/// It can be used for different purposes:
///
/// * the unique identifier the a node identifies itself within the topology;
/// * a mean to compare 2 nodes within the topology
///
/// TODO: highlights which module is utilizing the Id for proximity
/// (I think this is rings).
///
/// There is 2 ways to create a new Id:
///
/// 1. you can [compute] from an address:
///    ```
///    # use poldercast::{Id, Address};
///    # use multiaddr::Error;
///    # fn test() -> Result<(), Error> {
///    let address: Address = "/ip4/127.0.0.1/tcp/8080".parse()?;
///    let id = Id::compute(&address);
///    # assert_eq!(id, [31, 73, 11, 34, 104, 252, 242, 59].into());
///    # Ok(()) }
///    # test().unwrap();
///    ```
/// 2. you can manually construct one;
///    ```
///    # use poldercast::Id;
///    let id = Id::from(9283991);
///    ```
///
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct Id([u8; ID_LEN]);

pub const ID_LEN: usize = 8;

impl Id {
    /// generate a new identifier utilizing the given Random Number Generator
    /// ([RNG]).
    ///
    /// [RNG]: http://rust-random.github.io/rand/rand_core/trait.RngCore.html
    pub fn compute(address: &Address) -> Self {
        let mut hasher = Blake2b::new(ID_LEN);
        hasher.input(address.as_slice());
        let mut bytes = [0; ID_LEN];
        hasher.result(&mut bytes);
        Id(bytes)
    }
}

impl From<[u8; ID_LEN]> for Id {
    fn from(v: [u8; ID_LEN]) -> Self {
        Id(v)
    }
}

impl From<u64> for Id {
    fn from(v: u64) -> Self {
        Id(u64::to_le_bytes(v))
    }
}

impl From<Id> for u64 {
    fn from(v: Id) -> Self {
        u64::from_le_bytes(v.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::node::Address;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Id {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes = [0; ID_LEN];
            bytes.iter_mut().for_each(|byte| *byte = u8::arbitrary(g));
            Id(bytes)
        }
    }

    quickcheck! {
        fn same_address_same_hash(address: Address) -> bool {
            let id1 = Id::compute(&address);
            let id2 = Id::compute(&address);

            id1 == id2
        }

        fn different_addresses_different_hashes(address1: Address, address2: Address) -> bool {
            let id1 = Id::compute(&address1);
            let id2 = Id::compute(&address2);

            if address1 == address2 {
                id1 == id2
            } else {
                id1 != id2
            }
        }
    }
}
