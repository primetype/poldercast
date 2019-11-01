use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

/// this is a poldercast unique identifier.
///
/// It can be used for different purposes:
///
/// * the unique identifier the a node identifies itself within the topology;
/// * a mean to compare 2 nodes within the topology
///
/// The requirements from poldercast paper is that the [`Id`] is to be sortable
/// and to occupy a circular value space.
///
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id([u8; ID_LEN]);

const ID_LEN: usize = 24;

impl Id {
    pub fn generate<R: RngCore>(mut rng: R) -> Self {
        let mut id = Self::zero();
        rng.fill_bytes(&mut id.0);
        id
    }

    pub(crate) fn zero() -> Self {
        Id([0; ID_LEN])
    }
}

/* Conversions ************************************************************** */

impl From<[u8; ID_LEN]> for Id {
    fn from(v: [u8; ID_LEN]) -> Self {
        Id(v)
    }
}

/* AsRef ******************************************************************** */

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/* Display ***************************************************************** */

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Id").field(&hex::encode(self)).finish()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        hex::encode(self).fmt(f)
    }
}

/* FromStr ****************************************************************** */

impl FromStr for Id {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v = Self::zero();
        hex::decode_to_slice(s, &mut v.0)?;
        Ok(v)
    }
}

/* Serde ******************************************************************** */

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            serializer.serialize_bytes(self.as_ref())
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(internal::IdVisitor)
        } else {
            deserializer.deserialize_bytes(internal::IdVisitor)
        }
    }
}

mod internal {
    use super::{Id, ID_LEN};
    use serde::de::{Error, Visitor};
    use std::fmt;

    pub struct IdVisitor;
    impl<'de> Visitor<'de> for IdVisitor {
        type Value = Id;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str(&format!("expecting an ID of {} bytes", ID_LEN))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            v.parse().map_err(E::custom)
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v.len() != ID_LEN {
                Err(E::custom(format!(
                    "Invalid length size, expect {} but had {}",
                    ID_LEN,
                    v.len()
                )))
            } else {
                let mut sig = Id::zero();
                sig.0.copy_from_slice(v);
                Ok(sig)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Id {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes = [0; ID_LEN];
            g.fill_bytes(&mut bytes);
            Id::from(bytes)
        }
    }

    #[quickcheck]
    fn id_display_from_str(public_id: Id) -> bool {
        let encoded = public_id.to_string();
        let decoded = encoded.parse::<Id>().unwrap();
        public_id == decoded
    }

    #[quickcheck]
    fn id_encode_decode_json(public_id: Id) -> bool {
        let encoded = serde_json::to_string(&public_id).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        public_id == decoded
    }

    #[quickcheck]
    fn id_encode_decode_bincode(public_id: Id) -> bool {
        let encoded = bincode::serialize(&public_id).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        public_id == decoded
    }
}
