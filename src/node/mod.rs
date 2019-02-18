//! A node is a service that participate to the poldercast
//! topology.
//!
use std::{collections::HashSet, time::SystemTime};

mod address;
mod id;

pub use self::address::Address;
pub use self::id::Id;

use crate::{InterestLevel, Proximity, Subscription, Subscriptions, Topic};

/// The data associated to a Node.
///
/// This can be gossiped through the topology in order to update
/// the topology of new nodes or _better_ neighbors.
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    /// a unique identifier associated to the node
    pub(crate) id: Id,

    /// the address to contact the node
    pub(crate) address: Address,

    /// all the subscription this node is interested about
    /// (with associated priority of interest)
    pub(crate) subscriptions: Subscriptions,

    /// the `Id` of the other `Node` this `Node` is aware of
    pub(crate) subscribers: HashSet<Id>,

    /// this value denotes when this node exchange gossips
    /// with us for the last time.
    pub(crate) last_gossip: SystemTime,
}

impl Node {
    pub fn new(id: Id, address: Address) -> Self {
        Node {
            id: id,
            address,
            subscriptions: Subscriptions::new(),
            subscribers: HashSet::new(),
            last_gossip: SystemTime::now(),
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn subscriptions<'a>(&'a self) -> impl Iterator<Item = (&'a Topic, &'a InterestLevel)> {
        self.subscriptions.iter()
    }

    pub fn subscribers<'a>(&'a self) -> impl Iterator<Item = &'a Id> {
        self.subscribers.iter()
    }

    pub fn add_subscription(&mut self, subscription: Subscription) -> Option<InterestLevel> {
        self.subscriptions.add(subscription)
    }

    pub fn remove_subscription(&mut self, topic: &Topic) -> Option<InterestLevel> {
        self.subscriptions.remove(topic)
    }

    pub fn common_subscriptions<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a Topic> {
        self.subscriptions
            .common_subscriptions(&other.subscriptions)
    }

    pub fn common_subscribers<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a Id> {
        self.subscribers.intersection(&other.subscribers)
    }

    pub fn proximity(&self, other: &Self) -> Proximity {
        self.subscriptions.proximity_to(&other.subscriptions)
    }
}
