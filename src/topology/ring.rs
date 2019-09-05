//! Ring module of the node, it manages the ring links.
//! It aims at discovering a node’s successor and predecessor
//! for each topic in its subscription, and at quickly adapting
//! to new successors/predecessors in dynamic networks.
//!

use std::collections::{BTreeMap, BTreeSet};

use crate::topology::Module;
use crate::{Id, InterestLevel, Node, Topic};

/// the number of neighbor for a given subscribed topic of the given node.
///
/// although the protocol only requires a view of length 2 (i.e. one
/// predecessor and one successor), we keep an additional predecessor
/// and successor in case of failures or node churn.
pub const RINGS_MAX_VIEW_SIZE: usize = 4;

/// see [`RINGS_MAX_VIEW_SIZE`]
pub const RINGS_NEIGHBOR_PREDECESSOR_SIZE: usize = RINGS_MAX_VIEW_SIZE / 2;
/// see [`RINGS_MAX_VIEW_SIZE`]
pub const RINGS_NEIGHBOR_SUCCESSOR_SIZE: usize = RINGS_MAX_VIEW_SIZE / 2;

/// this object is responsible for maintaining the ring links
/// of the node.
#[derive(Clone, Debug)]
pub struct Rings {
    /// each node maintains `MAX_VIEW_SIZE` neighbors for each topic
    /// in its subscription: `NEIGHBOR_PREDECESSOR` with lower
    /// and `NEIGHBOR_SUCCESSOR` with higher id.
    pub(crate) neighbors: BTreeMap<Topic, TopicView>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(crate) enum Slot<A> {
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
pub struct TopicView([Slot<Id>; RINGS_MAX_VIEW_SIZE]);

impl Module for Rings {
    fn name(&self) -> &'static str {
        "rings"
    }

    fn select_gossips(
        &self,
        our_node: &Node,
        gossip_recipient: &Node,
        known_nodes: &BTreeMap<Id, Node>,
    ) -> BTreeMap<Id, Node> {
        self.select_nodes_to_send(our_node, gossip_recipient, known_nodes)
    }

    fn update(&mut self, self_node: &Node, known_nodes: &BTreeMap<Id, Node>) {
        self.update_view(self_node, known_nodes)
    }

    fn view(&self, known_nodes: &BTreeMap<Id, Node>, view: &mut BTreeMap<Id, Node>) {
        for neighborhood in self.neighbors.values() {
            for slot in neighborhood.iter() {
                if let Slot::Taken(id) = slot {
                    if let Some(node) = known_nodes.get(id) {
                        view.insert(*id, node.clone());
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    }
}

impl<A> Slot<A> {
    pub(crate) fn is_taken(&self) -> bool {
        match self {
            Slot::Taken(_) => true,
            Slot::Available => false,
        }
    }

    pub(crate) fn option(&self) -> Option<&A> {
        match self {
            Slot::Taken(a) => Some(a),
            Slot::Available => None,
        }
    }
}

impl Default for TopicView {
    fn default() -> Self {
        TopicView([Slot::Available; RINGS_MAX_VIEW_SIZE])
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
    /// the degree of the [`TopicView`] is the number of available
    /// neighbors in this view.
    pub(crate) fn degree(&self) -> usize {
        self.0.iter().filter(|v| v.is_taken()).count()
    }

    /// this function will remove the node of the given [`Id`] from the
    /// [`TopicView`].
    ///
    /// This function is linear in time. But we expect the [`RINGS_MAX_VIEW_SIZE`]
    /// to be rather small so performance should not be an issue
    pub(crate) fn remove_node(&mut self, id: Id) -> bool {
        let id = Slot::Taken(id);
        for slot in self.0.iter_mut() {
            if slot == &id {
                *slot = Slot::Available;
                return true;
            }
        }
        false
    }

    /// check if the given node of identifier [`Id`] is present in this [`TopicView`].
    ///
    /// This function is linear in time. But we expect the [`RINGS_MAX_VIEW_SIZE`]
    /// to be rather small so performance should not be an issue
    pub(crate) fn contains(&self, id: Id) -> bool {
        let id = Slot::Taken(id);
        self.0.iter().any(|v| v == &id)
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
    #[cfg(test)]
    fn successors(&self) -> impl Iterator<Item = &Slot<Id>> {
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
    pub(crate) fn successors_mut(&mut self) -> impl Iterator<Item = &mut Slot<Id>> {
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
    #[cfg(test)]
    fn predecessors(&self) -> impl Iterator<Item = &Slot<Id>> {
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
    pub(crate) fn predecessors_mut(&mut self) -> impl Iterator<Item = &mut Slot<Id>> {
        self.0
            .iter_mut()
            .rev()
            .skip(RINGS_NEIGHBOR_SUCCESSOR_SIZE)
            .take(RINGS_NEIGHBOR_PREDECESSOR_SIZE)
    }

    /// iterator over every neighbors, not ordered by preferences (see
    /// [`predecessors`] and [`successors`] for preference ordered iterators)
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Slot<Id>> {
        self.0.iter()
    }

    /// mutable iterator over every neighbors, not ordered by preferences (see
    /// [`predecessors_mut`] and [`successors_mut`] for preference ordered iterators)
    #[cfg(test)]
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Slot<Id>> {
        self.0.iter_mut()
    }
}

impl Rings {
    /// update the associated node's subscription priorities
    pub fn update_priorities(&mut self, self_node: &mut Node) {
        for (k, v) in self_node.subscriptions.iter_mut() {
            let degree = self
                .neighbors
                .entry(*k)
                .or_insert_with(TopicView::default)
                .degree();
            *v = match RINGS_MAX_VIEW_SIZE - degree {
                0 => InterestLevel::Low,
                1 => InterestLevel::Normal,
                _ => InterestLevel::High,
            };
        }
    }

    pub fn remove_node(&mut self, id: Id) -> bool {
        self.neighbors
            .iter_mut()
            .fold(false, |acc, (_, v)| acc || v.remove_node(id))
    }

    pub fn contains(&self, id: Id) -> bool {
        self.neighbors.iter().any(|(_, v)| v.contains(id))
    }

    /// return the size of the neighbor list
    pub fn degree(&self) -> usize {
        self.neighbors.iter().map(|(_, v)| v.degree()).sum()
    }

    // update the Rings view (neighbors for every topics) with the given new nodes
    fn update_view(&mut self, self_node: &Node, known_nodes: &BTreeMap<Id, Node>) {
        self.neighbors = BTreeMap::new();

        for topic in self_node.subscriptions.topics() {
            let view = select_best_nodes_for_topic(*self_node.id(), *topic, known_nodes);

            self.neighbors.insert(*topic, view);
        }
    }

    fn select_nodes_to_send(
        &self,
        self_node: &Node,
        gossip_node: &Node,
        known_nodes: &BTreeMap<Id, Node>,
    ) -> BTreeMap<Id, Node> {
        // these are the subscriptions in common between the gossip node and our nodes
        let common_topics: BTreeSet<Topic> = self_node
            .common_subscriptions(gossip_node)
            .cloned()
            .collect();

        // these are the subscribers in common between the 2 nodes
        let common_subscribers: BTreeSet<Id> = self_node
            .common_subscribers(&gossip_node)
            .cloned()
            .collect();

        // candidates are the one that are common subscribers sharing the same common
        // topics.
        let candidates: BTreeMap<Id, Node> = known_nodes
            .iter()
            .filter(|(k, v)| {
                common_subscribers.contains(k)
                    && v.subscriptions.topics().any(|k| common_topics.contains(k))
            })
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        let mut nodes = BTreeMap::new();
        for topic in common_topics {
            let view = select_best_nodes_for_topic(*gossip_node.id(), topic, &candidates);

            for candidate in view.iter().filter_map(|v| v.option()) {
                if let Some(node) = candidates.get(candidate) {
                    nodes.insert(*candidate, node.clone());
                }
            }
        }

        nodes
    }
}

fn select_best_nodes_for_topic(
    other_id: Id,
    topic: Topic,
    candidates: &BTreeMap<Id, Node>,
) -> TopicView {
    use std::ops::Bound::{self, Excluded, Included};
    let mut view = TopicView::default();

    {
        // these are the predecessor
        let mut predecessor = view.predecessors_mut();
        for (id, candidate) in candidates
            .range((Included(0.into()), Excluded(other_id)))
            .rev()
        {
            if candidate.subscriptions.contains(topic) {
                if let Some(p) = predecessor.next() {
                    *p = Slot::Taken(*id);
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
        for (id, candidate) in candidates
            .range((Excluded(other_id), Bound::Unbounded))
            .rev()
        {
            if candidate.subscriptions.contains(topic) {
                if let Some(p) = successor.next() {
                    *p = Slot::Taken(*id);
                } else {
                    // we can stop as soon as we have all the necessary element
                    break;
                }
            }
        }
    }

    view
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl<A: Arbitrary> Arbitrary for Slot<A> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match Arbitrary::arbitrary(g) {
                Some(v) => Slot::Taken(v),
                None => Slot::Available,
            }
        }
    }

    impl Arbitrary for TopicView {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut view = TopicView::default();
            for v in view.iter_mut() {
                *v = Arbitrary::arbitrary(g);
            }
            view
        }
    }

    quickcheck! {
        // here we test that reading all the predecessor and then the successor
        // is similar to reading all the elements
        fn predecessors_and_successors_is_all(view: TopicView) -> bool {
            let all = view.iter().cloned();

            // as a reminder, the predecessors are iterated from the end, so we need
            // to reverse them to see the same view as the original
            let mut predecessors : Vec<_> = view.predecessors().cloned().collect();
            predecessors.reverse();
            let chained = predecessors.into_iter().chain(view.successors().cloned());

            all.zip(chained).all(|(k, v)| k == v)
        }
    }
}
