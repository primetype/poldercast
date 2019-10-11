//! all the serde implementations and tests
//!

use crate::{Address};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
                match s.parse::<multiaddr::Multiaddr>() {
                    Ok(address) => Ok(Address::from(address)),
                    Err(_err) => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
                }
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
    use crate::{Address, Id, NodeData, Subscriptions};
    use bincode;
    use serde_json;

    quickcheck! {
        fn topic_encode_decode_json(subscriptions: Subscriptions) -> bool {
            let encoded = serde_json::to_string(&subscriptions).unwrap();
            let decoded : Subscriptions = serde_json::from_str(&encoded).unwrap();
            decoded == subscriptions
        }
        fn topic_encode_decode_bincode(subscriptions: Subscriptions) -> bool {
            let encoded = bincode::serialize(&subscriptions).unwrap();
            let decoded : Subscriptions = bincode::deserialize(&encoded).unwrap();
            decoded == subscriptions
        }

        fn address_encode_decode_json(address: Address) -> bool {
            let encoded = serde_json::to_string(&address).unwrap();
            let decoded : Address = serde_json::from_str(&encoded).unwrap();
            decoded == address
        }
        fn address_encode_decode_bincode(address: Address) -> bool {
            let encoded = bincode::serialize(&address).unwrap();
            let decoded : Address = bincode::deserialize(&encoded).unwrap();
            decoded == address
        }

        fn id_encode_decode_json(id: Id) -> bool {
            let encoded = serde_json::to_string(&id).unwrap();
            let decoded : Id = serde_json::from_str(&encoded).unwrap();
            decoded == id
        }
        fn id_encode_decode_bincode(id: Id) -> bool {
            let encoded = bincode::serialize(&id).unwrap();
            let decoded : Id = bincode::deserialize(&encoded).unwrap();
            decoded == id
        }

        fn node_encode_decode_json(node: NodeData) -> bool {
            let encoded = serde_json::to_string(&node).unwrap();
            let decoded : NodeData = serde_json::from_str(&encoded).unwrap();
            decoded == node
        }
        fn node_encode_decode_bincode(node: NodeData) -> bool {
            let encoded = bincode::serialize(&node).unwrap();
            let decoded : NodeData = bincode::deserialize(&encoded).unwrap();
            decoded == node
        }
    }
}
