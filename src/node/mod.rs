//! A node is a service that participate to the poldercast
//! topology.
//!

use crate::{InterestLevel, Proximity, Subscription, Subscriptions, Topic};
use rand_core::{CryptoRng, RngCore};
#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, time::SystemTime};

mod address;
mod id;

pub use self::address::Address;
pub use self::id::{Id, PrivateId};

/// this is the data that the local node contains
pub struct Node {
    private_id: PrivateId,

    node_data: NodeData,
}

/// The data associated to a Node.
///
/// This can be gossiped through the topology in order to update
/// the topology of new nodes or _better_ neighbors.
///
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct NodeData {
    /// a unique identifier associated to the node
    pub(crate) id: Id,

    /// the address to contact the node
    pub(crate) address: Option<Address>,

    /// all the subscription this node is interested about
    /// (with associated priority of interest)
    pub(crate) subscriptions: Subscriptions,

    /// the `Id` of the other `Node` this `Node` is aware of
    pub(crate) subscribers: BTreeSet<Id>,

    /// this value denotes when this node exchange gossips
    /// with us for the last time.
    pub(crate) last_gossip: SystemTime,
}

impl Node {
    /// create a new unreachable Node with the given [`Id`].
    ///
    /// [`Id`]: ./struct.Id.html
    ///
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let private_id = PrivateId::generate(rng);
        Self::new(private_id)
    }

    pub fn new(private_id: PrivateId) -> Self {
        let id = private_id.id();

        Node {
            private_id,
            node_data: NodeData::new(id),
        }
    }

    pub fn generate_with<R: RngCore + CryptoRng>(rng: &mut R, address: Address) -> Self {
        let private_id = PrivateId::generate(rng);
        Self::new_with(private_id, address)
    }

    pub fn new_with(private_id: PrivateId, address: Address) -> Self {
        let id = private_id.id();

        Node {
            private_id,
            node_data: NodeData::new_with(id, address),
        }
    }

    pub fn data(&self) -> &NodeData {
        &self.node_data
    }

    pub fn data_mut(&mut self) -> &mut NodeData {
        &mut self.node_data
    }
}

impl NodeData {
    /// create a new unreachable Node with the given [`Id`].
    ///
    /// [`Address`]: ./struct.Address.html
    ///
    fn new(id: Id) -> Self {
        NodeData {
            id,
            address: None,
            subscriptions: Subscriptions::default(),
            subscribers: BTreeSet::new(),
            last_gossip: SystemTime::now(),
        }
    }
    /// create a new Node with the given [`Address`].
    ///
    /// [`Address`]: ./struct.Address.html
    ///
    fn new_with(id: Id, address: Address) -> Self {
        NodeData {
            id: id,
            address: Some(address),
            subscriptions: Subscriptions::default(),
            subscribers: BTreeSet::new(),
            last_gossip: SystemTime::now(),
        }
    }

    /// access the unique identifier of the `Node`.
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// get the Node's address (mean to contact it)
    pub fn address(&self) -> &Option<Address> {
        &self.address
    }

    /// these are the [`Topic`] and the [`InterestLevel`] associated.
    ///
    pub fn subscriptions(&self) -> impl Iterator<Item = (&Topic, &InterestLevel)> {
        self.subscriptions.iter()
    }

    /// the nodes that are related to this Node
    pub fn subscribers(&self) -> impl Iterator<Item = &Id> {
        self.subscribers.iter()
    }

    /// add a subscription
    pub fn add_subscription(&mut self, subscription: Subscription) -> Option<InterestLevel> {
        self.subscriptions.add(subscription)
    }

    /// remove a subscriptions
    pub fn remove_subscription(&mut self, topic: Topic) -> Option<InterestLevel> {
        self.subscriptions.remove(topic)
    }

    /// list all common subscriptions between the two nodes
    pub fn common_subscriptions<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a Topic> {
        self.subscriptions
            .common_subscriptions(&other.subscriptions)
    }

    /// list all common subscribers between the two nodes
    pub fn common_subscribers<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a Id> {
        self.subscribers.intersection(&other.subscribers)
    }

    /// compute the relative proximity between these 2 nodes.
    ///
    /// This is based on the subscription. The more 2 nodes have subscription
    /// in common the _closer_ they are.
    pub fn proximity(&self, other: &Self) -> Proximity {
        self.subscriptions.proximity_to(&other.subscriptions)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for NodeData {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            use std::ops::Sub;
            let address: Option<Address> = Arbitrary::arbitrary(g);
            let id = Id::arbitrary(g);

            NodeData {
                id,
                address,
                subscriptions: Subscriptions::arbitrary(g),
                subscribers: Arbitrary::arbitrary(g),
                last_gossip: SystemTime::now()
                    .sub(std::time::Duration::new(u32::arbitrary(g) as u64, 0)),
            }
        }
    }
}
