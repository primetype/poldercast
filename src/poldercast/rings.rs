use crate::{
    GossipsBuilder, Id, Layer, NodeProfile, NodeRef, Nodes, Selection, Subscription, Subscriptions,
    Topic, ViewBuilder,
};
use std::collections::{BTreeMap, BTreeSet};

/// the number of neighbor for a given subscribed topic of the given node.
///
/// although the protocol only requires a view of length 2 (i.e. one
/// predecessor and one successor), we keep an additional predecessor
/// and successor in case of failures or node churn.
const RINGS_MAX_VIEW_SIZE: usize = 4;

/// see [`RINGS_MAX_VIEW_SIZE`]
const RINGS_NEIGHBOR_PREDECESSOR_SIZE: usize = RINGS_MAX_VIEW_SIZE / 2;
/// see [`RINGS_MAX_VIEW_SIZE`]
const RINGS_NEIGHBOR_SUCCESSOR_SIZE: usize = RINGS_MAX_VIEW_SIZE / 2;

/// this object is responsible for maintaining the ring links
/// of the node.
///
/// Ring module of the node, it manages the ring links.
/// It aims at discovering a node’s successor and predecessor
/// for each topic in its subscription, and at quickly adapting
/// to new successors/predecessors in dynamic networks.
///
#[derive(Clone, Debug)]
pub struct Rings {
    /// each node maintains `RINGS_MAX_VIEW_SIZE` neighbors for each topic
    /// in its subscription: `RINGS_NEIGHBOR_PREDECESSOR` with lower
    /// and `RINGS_NEIGHBOR_SUCCESSOR` with higher id.
    neighbors: BTreeMap<Topic, TopicView>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
enum Slot<A> {
    Taken(A),
    Available,
}

/// the Rings' Topic view, [`Id`] of the nodes that are subscribed
/// to a given Topic.
///
/// This structure is mainly necessary for the [`Rings`] module.
///
/// [`Rings`]: ./node/ring/struct.Rings.html
#[derive(Clone, Debug)]
struct TopicView([Slot<NodeRef>; RINGS_MAX_VIEW_SIZE]);

impl Layer for Rings {
    fn alias(&self) -> &'static str {
        "rings"
    }

    fn reset(&mut self) {
        self.neighbors = BTreeMap::new()
    }

    fn populate(&mut self, identity: &NodeProfile, all_nodes: &Nodes) {
        self.update_view(identity, all_nodes)
    }

    fn gossips(&mut self, identity: &NodeProfile, gossips: &mut GossipsBuilder, all_nodes: &Nodes) {
        self.select_nodes_to_send(identity, gossips, all_nodes.available_nodes())
    }

    fn view(&mut self, view: &mut ViewBuilder) {
        match *view.selection() {
            Selection::Any => {
                for neighborhood in self.neighbors.values() {
                    neighborhood.populate_view(view.origin().map(|from| from.public_id()), view);
                }
            }
            Selection::Topic { topic } => {
                if let Some(neighborhood) = self.neighbors.get(&topic) {
                    neighborhood.populate_view(view.origin().map(|from| from.public_id()), view);
                }
            }
        }
    }
}

impl<A> Slot<A> {
    fn option(&self) -> Option<&A> {
        match self {
            Slot::Taken(a) => Some(a),
            Slot::Available => None,
        }
    }
}

impl Default for TopicView {
    fn default() -> Self {
        TopicView([
            Slot::Available,
            Slot::Available,
            Slot::Available,
            Slot::Available,
        ])
    }
}

impl Default for Rings {
    fn default() -> Self {
        Rings {
            neighbors: BTreeMap::default(),
        }
    }
}

impl TopicView {
    fn populate_view(&self, from: Option<Id>, view_builder: &mut ViewBuilder) {
        if let Some(from) = from {
            if self.is_predecessor(from) {
                self.successors()
                    .filter_map(|slot| slot.option())
                    .for_each(|node| view_builder.add(node.clone()));
            } else if self.is_successor(from) {
                self.predecessors()
                    .filter_map(|slot| slot.option())
                    .for_each(|node| view_builder.add(node.clone()));
            } else {
                self.iter()
                    .filter_map(|slot| slot.option())
                    .for_each(|node| view_builder.add(node.clone()));
            }
        } else {
            self.iter()
                .filter_map(|slot| slot.option())
                .for_each(|node| view_builder.add(node.clone()));
        }
    }

    fn is_predecessor(&self, public_id: Id) -> bool {
        self.predecessors()
            .filter_map(|slot| slot.option())
            .any(|node| node.public_id() == public_id)
    }

    fn is_successor(&self, public_id: Id) -> bool {
        self.successors()
            .filter_map(|slot| slot.option())
            .any(|node| node.public_id() == public_id)
    }

    /// return an iterator over the successors in this [`TopicView`].
    ///
    /// In the case [`RINGS_NEIGHBOR_SUCCESSOR_SIZE`] allows more than
    /// one successor, the successors in this iterator are sorted from
    /// the closest to the node to the farthest (i.e. they are in the
    /// order of interest already.)
    ///
    /// This function returns the [`Slot`] as well, this is so we can
    /// modify the slot in the rings module when we need to add new items
    /// in the slot
    fn successors(&self) -> impl Iterator<Item = &Slot<NodeRef>> {
        self.0
            .iter()
            .skip(RINGS_NEIGHBOR_PREDECESSOR_SIZE)
            .take(RINGS_NEIGHBOR_SUCCESSOR_SIZE)
    }

    /// return a mutable iterator over the successors in this [`TopicView`].
    ///
    /// In the case [`RINGS_NEIGHBOR_SUCCESSOR_SIZE`] allows more than
    /// one successor, the successors in this iterator are sorted from
    /// the closest to the node to the farthest (i.e. they are in the
    /// order of interest already.)
    fn successors_mut(&mut self) -> impl Iterator<Item = &mut Slot<NodeRef>> {
        self.0
            .iter_mut()
            .skip(RINGS_NEIGHBOR_PREDECESSOR_SIZE)
            .take(RINGS_NEIGHBOR_SUCCESSOR_SIZE)
    }

    /// return an iterator over the predecessors in this [`TopicView`].
    ///
    /// In the case [`RINGS_NEIGHBOR_PREDECESSOR_SIZE`] allows more than
    /// one predecessor, the predecessors in this iterator are sorted from
    /// the closest to the node to the farthest (i.e. they are in the
    /// order of interest already.)
    fn predecessors(&self) -> impl Iterator<Item = &Slot<NodeRef>> {
        self.0
            .iter()
            .rev()
            .skip(RINGS_NEIGHBOR_SUCCESSOR_SIZE)
            .take(RINGS_NEIGHBOR_PREDECESSOR_SIZE)
    }

    /// return a mutable iterator over the predecessors in this [`TopicView`].
    ///
    /// In the case [`RINGS_NEIGHBOR_PREDECESSOR_SIZE`] allows more than
    /// one predecessor, the predecessors in this iterator are sorted from
    /// the closest to the node to the farthest (i.e. they are in the
    /// order of interest already.)
    fn predecessors_mut(&mut self) -> impl Iterator<Item = &mut Slot<NodeRef>> {
        self.0
            .iter_mut()
            .rev()
            .skip(RINGS_NEIGHBOR_SUCCESSOR_SIZE)
            .take(RINGS_NEIGHBOR_PREDECESSOR_SIZE)
    }

    /// iterator over every neighbors, not ordered by preferences (see
    /// [`predecessors`] and [`successors`] for preference ordered iterators)
    fn iter(&self) -> impl Iterator<Item = &Slot<NodeRef>> {
        self.0.iter()
    }
}

impl Rings {
    // update the Rings view (neighbors for every topics) with the given new nodes
    fn update_view(&mut self, self_node: &NodeProfile, known_nodes: &Nodes) {
        self.neighbors = BTreeMap::new();

        for subscription in self_node.subscriptions().iter() {
            let view = select_best_nodes_for_topic(
                *self_node.public_id(),
                *subscription,
                known_nodes.available_nodes(),
            );

            self.neighbors.insert(subscription.topic(), view);
        }
    }

    fn select_nodes_to_send(
        &self,
        self_node: &NodeProfile,
        gossip_builder: &mut GossipsBuilder,
        known_nodes: &BTreeMap<Id, NodeRef>,
    ) {
        let gossip_node_id = gossip_builder.recipient().public_id();
        let gossip_node = gossip_builder.recipient().clone();

        // these are the subscriptions in common between the gossip node and our nodes
        let common_topics: Subscriptions = self_node
            .common_subscriptions(&gossip_node.node().profile())
            .cloned()
            .collect();

        // these are the subscribers in common between the 2 nodes
        let common_subscribers: BTreeSet<Id> = self_node
            .common_subscribers(&gossip_node.node().profile())
            .cloned()
            .collect();

        // candidates are the one that are common subscribers sharing the same common
        // topics.
        let candidates: BTreeMap<Id, NodeRef> = known_nodes
            .iter()
            .filter(|(k, v)| {
                common_subscribers.contains(k)
                    && v.node()
                        .profile()
                        .subscriptions()
                        .common_subscriptions(&common_topics)
                        .next()
                        .is_some()
            })
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        for topic in common_topics.iter() {
            let view = select_best_nodes_for_topic(gossip_node_id, *topic, &candidates);

            for candidate in view.iter().filter_map(|v| v.option()) {
                if let Some(node) = candidates.get(&candidate.public_id()) {
                    gossip_builder.add(node.clone());
                }
            }
        }
    }
}

fn select_best_nodes_for_topic(
    other_id: Id,
    subscription: Subscription,
    candidates: &BTreeMap<Id, NodeRef>,
) -> TopicView {
    use std::ops::Bound::{self, Excluded, Included};
    let mut view = TopicView::default();

    {
        // these are the predecessor
        let mut predecessor = view.predecessors_mut();
        for candidate in candidates
            .range((Included(Id::zero()), Excluded(other_id)))
            .rev()
            .map(|v| v.1)
        {
            if candidate
                .node()
                .profile()
                .subscriptions()
                .contains(subscription.topic())
            {
                if let Some(p) = predecessor.next() {
                    *p = Slot::Taken(candidate.clone());
                } else {
                    // we can stop as soon as we have all the necessary element
                    break;
                }
            }
        }
    }

    {
        // these are the successor of the topic
        let mut successor = view.successors_mut();
        for candidate in candidates
            .range((Excluded(other_id), Bound::Unbounded))
            .rev()
            .map(|v| v.1)
        {
            if candidate
                .node()
                .profile()
                .subscriptions()
                .contains(subscription.topic())
            {
                if let Some(p) = successor.next() {
                    *p = Slot::Taken(candidate.clone());
                } else {
                    // we can stop as soon as we have all the necessary element
                    break;
                }
            }
        }
    }

    view
}
