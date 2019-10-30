use crate::{
    Address, Gossip, Id, Logs, Proximity, Record, StrikeReason, Subscription, Subscriptions,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeSet,
    rc::Rc,
};

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
    public_id: Id,

    address: Option<Address>,

    subscriptions: Subscriptions,

    subscribers: BTreeSet<Id>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRef {
    node: Rc<RefCell<Node>>,
}

impl NodeProfile {
    pub fn public_id(&self) -> &Id {
        &self.public_id
    }

    pub fn address(&self) -> Option<&Address> {
        self.address.as_ref()
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
    fn new(profile: NodeProfile) -> Self {
        Self {
            profile,
            logs: Logs::default(),
            record: Record::default(),
        }
    }

    pub fn profile(&self) -> &NodeProfile {
        &self.profile
    }

    fn update_gossip(&mut self, gossip: Gossip) {
        self.update_profile(gossip.node);
        self.logs_mut().updated();
    }

    fn update_profile(&mut self, profile: NodeProfile) {
        self.profile = profile;
    }

    pub fn record(&self) -> &Record {
        &self.record
    }

    pub(crate) fn record_mut(&mut self) -> &mut Record {
        &mut self.record
    }

    pub fn logs(&self) -> &Logs {
        &self.logs
    }

    pub(crate) fn logs_mut(&mut self) -> &mut Logs {
        &mut self.logs
    }
}

impl NodeRef {
    pub(crate) fn new(profile: NodeProfile) -> Self {
        Self {
            node: Rc::new(RefCell::new(Node::new(profile))),
        }
    }

    pub fn node(&self) -> Ref<Node> {
        self.node.borrow()
    }

    pub fn node_mut(&self) -> RefMut<Node> {
        self.node.borrow_mut()
    }

    pub fn public_id(&self) -> Id {
        *self.node().profile().public_id()
    }

    pub fn strike(&self, strike: StrikeReason) {
        self.node_mut().record.strike(strike);
    }

    /// update the [`NodeProfile`] with the newly provided one.
    ///
    /// [`NodeProfile`]: ./struct.NodeProfile.html
    pub(crate) fn update_gossip(&mut self, gossip: Gossip) {
        self.node_mut().update_gossip(gossip)
    }
}
