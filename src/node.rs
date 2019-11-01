use crate::{Address, Id, Logs, Proximity, Record, Subscription, Subscriptions};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
    #[serde(flatten)]
    info: NodeInfo,

    subscriptions: Subscriptions,

    subscribers: BTreeSet<Id>,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    profile: NodeProfile,

    logs: Logs,

    record: Record,
}

impl NodeInfo {
    pub fn id(&self) -> &Id {
        &self.id
    }
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

    pub fn id(&mut self, id: Id) -> &mut Self {
        self.id = id;
        self
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
            subscribers: BTreeSet::default(),
        }
    }
}

impl NodeProfile {
    pub(crate) fn info(&self) -> &NodeInfo {
        &self.info
    }

    pub fn id(&self) -> &Id {
        &self.info.id
    }

    pub fn address(&self) -> Option<&Address> {
        self.info.address.as_ref()
    }

    pub fn subscriptions(&self) -> &Subscriptions {
        &self.subscriptions
    }

    pub fn subscribers(&self) -> &BTreeSet<Id> {
        &self.subscribers
    }

    /// list all common subscriptions between the two nodes
    pub fn common_subscriptions<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = &'a Subscription> {
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

    pub fn check(&self) -> bool {
        // XXX: missing self validation of the data
        true
    }
}

impl Node {
    pub(crate) fn new(profile: NodeProfile) -> Self {
        Self {
            profile,
            logs: Logs::default(),
            record: Record::default(),
        }
    }

    pub(crate) fn info(&self) -> &NodeInfo {
        &self.profile().info()
    }

    pub fn address(&self) -> Option<&Address> {
        self.profile().address()
    }

    pub fn id(&self) -> &Id {
        self.profile().id()
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
