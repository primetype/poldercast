use crate::{
    topic::{InterestLevel, Subscriptions, Topic},
    Gossip, PriorityMap, Subscription,
};
use keynesis::{key::ed25519, passport::block::Time};
use std::net::SocketAddr;

pub struct Profile {
    subscriptions: PriorityMap<InterestLevel, Topic>,
    gossip: Gossip,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Proximity {
    priority: usize,
    proximity: usize,
}

impl Profile {
    pub fn new(address: SocketAddr, id: &ed25519::SecretKey) -> Self {
        let gossip = Gossip::new(address, id, Subscriptions::new().as_slice());

        Self {
            gossip,
            subscriptions: PriorityMap::new(Subscriptions::MAX_NUM_SUBSCRIPTIONS),
        }
    }

    pub fn from_gossip(gossip: Gossip) -> Self {
        let mut subscriptions = PriorityMap::new(Subscriptions::MAX_NUM_SUBSCRIPTIONS);

        for subscription in gossip.subscriptions() {
            let interest_level = subscription.interest_level();
            let topic = subscription.topic();
            subscriptions.put(interest_level, topic);
        }

        Self {
            gossip,
            subscriptions,
        }
    }

    pub(crate) fn clear_subscriptions(&mut self) {
        self.subscriptions.clear();
    }

    pub(crate) fn subscriptions_mut(&mut self) -> &mut PriorityMap<InterestLevel, Topic> {
        &mut self.subscriptions
    }

    pub(crate) fn unsubscribe(&mut self, topic: &Topic) {
        self.subscriptions.remove(topic);
    }

    pub fn gossip(&self) -> &Gossip {
        &self.gossip
    }

    pub(crate) fn commit_gossip(&mut self, id: &ed25519::SecretKey) -> &Gossip {
        let subscriptions = self.subscriptions();

        self.gossip = Gossip::new(self.address(), id, subscriptions.as_slice());

        &self.gossip
    }

    pub fn id(&self) -> ed25519::PublicKey {
        self.gossip.id()
    }

    pub fn last_update(&self) -> Time {
        self.gossip.time()
    }

    pub fn address(&self) -> SocketAddr {
        self.gossip.address()
    }

    pub fn subscriptions(&self) -> Subscriptions {
        let mut subscriptions = Subscriptions::new();
        for (interest_level, topic) in self.subscriptions.iter() {
            let sub = Subscription::new(*topic, *interest_level);
            subscriptions
                .push(sub.as_slice())
                .expect("We are already limiting the number of internal subscriptions");
        }
        subscriptions
    }

    pub fn proximity_to(&self, to: &Self) -> Proximity {
        let mut priority_score = 0;
        let mut proximity_score = 0;
        for (interest_level, topic) in self.subscriptions.iter() {
            if let Some((to, _)) = to.subscriptions.get(topic) {
                proximity_score += 1;
                priority_score += interest_level.priority_score(*to);
            }
        }
        Proximity {
            proximity: proximity_score,
            priority: priority_score,
        }
    }
}

impl PartialOrd<Self> for Proximity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Proximity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::{Equal, Greater, Less};
        if self.priority > other.priority {
            Greater
        } else if self.priority < other.priority {
            Less
        } else if self.proximity > other.proximity {
            Greater
        } else if self.proximity < other.proximity {
            Less
        } else {
            Equal
        }
    }
}

impl From<Gossip> for Profile {
    fn from(gossip: Gossip) -> Self {
        Self::from_gossip(gossip)
    }
}
