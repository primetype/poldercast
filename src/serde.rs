//! all the serde implementations and tests
//!

use crate::{Address, Node};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Node", 4)?;

        state.serialize_field("address", self.address())?;
        state.serialize_field("subscriptions", &self.subscriptions)?;
        state.serialize_field("subscribers", &self.subscribers)?;
        state.serialize_field("last_gossip", &self.last_gossip)?;

        state.end()
    }
}
impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        use std::fmt;

        enum Field {
            Address,
            Subscriptions,
            Subscribers,
            LastGossip,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter
                            .write_str("`address`, `subscriptions`, `subscribers` or `last_gossip`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "address" => Ok(Field::Address),
                            "subscriptions" => Ok(Field::Subscriptions),
                            "subscribers" => Ok(Field::Subscribers),
                            "last_gossip" => Ok(Field::LastGossip),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct NodeVisitor;
        impl<'de> Visitor<'de> for NodeVisitor {
            type Value = Node;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Node")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let address = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let subscriptions = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let subscribers = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let last_gossip = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                Ok(Node::reconstruct(
                    address,
                    subscriptions,
                    subscribers,
                    last_gossip,
                ))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut address = None;
                let mut subscriptions = None;
                let mut subscribers = None;
                let mut last_gossip = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Address => {
                            if address.is_some() {
                                return Err(de::Error::duplicate_field("address"));
                            }
                            address = Some(map.next_value()?);
                        }
                        Field::Subscriptions => {
                            if subscriptions.is_some() {
                                return Err(de::Error::duplicate_field("subscriptions"));
                            }
                            subscriptions = Some(map.next_value()?);
                        }
                        Field::Subscribers => {
                            if subscribers.is_some() {
                                return Err(de::Error::duplicate_field("subscribers"));
                            }
                            subscribers = Some(map.next_value()?);
                        }
                        Field::LastGossip => {
                            if last_gossip.is_some() {
                                return Err(de::Error::duplicate_field("last_gossip"));
                            }
                            last_gossip = Some(map.next_value()?);
                        }
                    }
                }
                let address = address.ok_or_else(|| de::Error::missing_field("address"))?;
                let subscriptions =
                    subscriptions.ok_or_else(|| de::Error::missing_field("subscriptions"))?;
                let subscribers =
                    subscribers.ok_or_else(|| de::Error::missing_field("subscribers"))?;
                let last_gossip =
                    last_gossip.ok_or_else(|| de::Error::missing_field("last_gossip"))?;
                Ok(Node::reconstruct(
                    address,
                    subscriptions,
                    subscribers,
                    last_gossip,
                ))
            }
        }

        const FIELDS: &'static [&'static str] =
            &["address", "subscriptions", "subscribers", "last_gossip"];
        deserializer.deserialize_struct("Node", FIELDS, NodeVisitor)
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
    use crate::{Address, Id, Node, Subscriptions};
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

        fn node_encode_decode_json(node: Node) -> bool {
            let encoded = serde_json::to_string(&node).unwrap();
            let decoded : Node = serde_json::from_str(&encoded).unwrap();
            decoded == node
        }
        fn node_encode_decode_bincode(node: Node) -> bool {
            let encoded = bincode::serialize(&node).unwrap();
            let decoded : Node = bincode::deserialize(&encoded).unwrap();
            decoded == node
        }
    }
}
