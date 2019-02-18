use rand_core::RngCore;

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
/// 1. you can [generate] a new one;
///    ```
///    # use rand_chacha::ChaChaRng;
///    # use rand_core::{SeedableRng, RngCore};
///    # use poldercast::Id;
///    # let mut rng = ChaChaRng::seed_from_u64(0x4152484920);
///    let id = Id::generate(&mut rng);
///    # assert_eq!(id, 293088806904227309252857358049315044442.into());
///    ```
/// 2. you can manually construct one;
///    ```
///    # use poldercast::Id;
///    let id = Id::from(293088806904227309252857358049315044442);
///    ```
///
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Id(u128);

impl Id {
    /// generate a new identifier utilizing the given Random Number Generator
    /// ([RNG]).
    ///
    /// [RNG]: http://rust-random.github.io/rand/rand_core/trait.RngCore.html
    pub fn generate<Rng>(rng: &mut Rng) -> Self
    where
        Rng: RngCore,
    {
        let mut bytes = [0; 16];
        rng.fill_bytes(&mut bytes);
        let id = u128::from_le_bytes(bytes);
        Id(id)
    }
}

impl From<u128> for Id {
    fn from(v: u128) -> Self {
        Id(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Id {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Id(u128::arbitrary(g))
        }
    }
}
