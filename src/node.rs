#![allow(deprecated)]

use crate::{Address, Id, Logs, Proximity, Record, Subscription, Subscriptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeInfo {
    id: Id,
    address: Option<Address>,
}

/// The profile of the node, its [`Id`], its [`Address`] and its
/// [`Subscriptions`].
///
/// This is the information that is created and propagated by the
/// Node itself.
///
/// [`Id`]: ./struct.Id.html
/// [`Address`]: ./struct.Address.html
/// [`Subscriptions`]: ./struct.Subscriptions.html
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeProfile {
    info: NodeInfo,

    subscriptions: Subscriptions,
}

pub struct NodeProfileBuilder {
    id: Id,
    address: Option<Address>,
    subscriptions: Subscriptions,
}

/// Data we store about a node, this includes the [`NodeProfile`] in the
/// latest state known of, as well as any other metadata useful for operating
/// with the node.
///
/// [`NodeProfile`]: ./struct.NodeProfile.html
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    profile: NodeProfile,

    address: Address,

    logs: Logs,

    record: Record,
}

impl NodeInfo {
    pub fn address(&self) -> Option<&Address> {
        self.address.as_ref()
    }
}

impl NodeProfileBuilder {
    pub fn new() -> Self {
        Self {
            id: Id::generate(rand::thread_rng()),
            address: None,
            subscriptions: Subscriptions::default(),
        }
    }

    pub fn address(&mut self, address: Address) -> &mut Self {
        self.address = Some(address);
        self
    }

    pub fn add_subscription(&mut self, subscription: Subscription) -> &mut Self {
        self.subscriptions.insert(subscription);
        self
    }

    pub fn build(&self) -> NodeProfile {
        NodeProfile {
            info: NodeInfo {
                id: self.id,
                address: self.address.clone(),
            },
            subscriptions: self.subscriptions.clone(),
        }
    }
}

impl NodeProfile {
    pub(crate) fn info(&self) -> &NodeInfo {
        &self.info
    }

    pub fn address(&self) -> Option<&Address> {
        self.info.address.as_ref()
    }

    pub fn subscriptions(&self) -> &Subscriptions {
        &self.subscriptions
    }

    /// list all common subscriptions between the two nodes
    pub fn common_subscriptions<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = &'a Subscription> {
        self.subscriptions
            .common_subscriptions(&other.subscriptions)
    }

    /// compute the relative proximity between these 2 nodes.
    ///
    /// This is based on the subscription. The more 2 nodes have subscription
    /// in common the _closer_ they are.
    pub fn proximity(&self, other: &Self) -> Proximity {
        self.subscriptions.proximity_to(&other.subscriptions)
    }

    pub fn check(&self) -> bool {
        // XXX: missing self validation of the data
        true
    }
}

pub enum NodeAddress {
    Discoverable(Address),
    NonDiscoverable(Address),
}

impl NodeAddress {
    pub(crate) fn is_discoverable(&self) -> bool {
        match self {
            Self::Discoverable(_address) => true,
            Self::NonDiscoverable(_address) => false,
        }
    }

    pub(crate) fn as_ref(&self) -> &Address {
        match self {
            Self::Discoverable(address) => address,
            Self::NonDiscoverable(address) => address,
        }
    }
}

impl Node {
    pub(crate) fn new(address: Address, profile: NodeProfile) -> Self {
        Self {
            profile,
            address,
            logs: Logs::default(),
            record: Record::default(),
        }
    }

    pub(crate) fn info(&self) -> &NodeInfo {
        &self.profile().info()
    }

    pub fn address(&self) -> NodeAddress {
        self.profile()
            .address()
            .cloned()
            .map(NodeAddress::Discoverable)
            .unwrap_or_else(|| NodeAddress::NonDiscoverable(self.address.clone()))
    }

    pub fn profile(&self) -> &NodeProfile {
        &self.profile
    }

    pub(crate) fn update_gossip(&mut self, gossip: NodeProfile) {
        self.update_profile(gossip);
        self.logs_mut().updated();
    }

    fn update_profile(&mut self, profile: NodeProfile) {
        self.profile = profile;
    }

    pub fn record(&self) -> &Record {
        &self.record
    }

    pub fn record_mut(&mut self) -> &mut Record {
        &mut self.record
    }

    pub fn logs(&self) -> &Logs {
        &self.logs
    }

    pub fn logs_mut(&mut self) -> &mut Logs {
        &mut self.logs
    }
}

impl Default for NodeProfileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for NodeInfo {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            NodeInfo {
                id: Id::arbitrary(g),
                address: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for NodeProfile {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            NodeProfile {
                info: NodeInfo::arbitrary(g),
                subscriptions: Subscriptions::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn node_info_encode_decode_json(node_info: NodeInfo) -> bool {
        let encoded = serde_json::to_string(&node_info).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        node_info == decoded
    }

    #[quickcheck]
    fn node_info_encode_decode_bincode(node_info: NodeInfo) -> bool {
        let encoded = bincode::serialize(&node_info).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        node_info == decoded
    }

    #[quickcheck]
    fn node_profile_encode_decode_json(node_profile: NodeProfile) -> bool {
        let encoded = serde_json::to_string(&node_profile).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        node_profile == decoded
    }

    #[quickcheck]
    fn node_profile_encode_decode_bincode(node_profile: NodeProfile) -> bool {
        let encoded = bincode::serialize(&node_profile).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        node_profile == decoded
    }
}
