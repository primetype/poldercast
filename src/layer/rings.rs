use crate::{
    layer::{Layer, Selection, ViewBuilder},
    InterestLevel, PriorityMap, Profile, Subscription, Subscriptions, Topic,
};
use keynesis::key::ed25519;
use std::cmp::Ordering;

struct Ring {
    length: u8,
    predecessors: lru::LruCache<ed25519::PublicKey, ()>,
    successors: lru::LruCache<ed25519::PublicKey, ()>,

    current_low: Option<ed25519::PublicKey>,
    current_max: Option<ed25519::PublicKey>,
}

pub struct Rings {
    length: u8,
    links: lru::LruCache<Topic, Ring>,
}

impl Ring {
    fn new(length: u8) -> Self {
        Self {
            length,
            predecessors: lru::LruCache::new(length as usize / 2),
            successors: lru::LruCache::new(length as usize / 2),
            current_low: None,
            current_max: None,
        }
    }

    pub fn remove(&mut self, id: &ed25519::PublicKey) {
        if self.predecessors.pop(id).is_some() {
            self.current_low = self.predecessors.iter().map(|(k, _)| k).min().copied();
        }
        if self.successors.pop(id).is_some() {
            self.current_max = self.successors.iter().map(|(k, _)| k).max().copied();
        }
    }

    pub fn interest_level(&self) -> InterestLevel {
        let max = self.length;
        let size = (self.predecessors.len() as u8).wrapping_add(self.successors.len() as u8);

        let multiplier = u8::MAX.wrapping_div_euclid(max);
        let level = max.wrapping_sub(size).wrapping_mul(multiplier);

        InterestLevel::new(level)
    }

    pub fn recipients(&mut self, builder: &mut ViewBuilder) {
        let (predecessor, successor) = if let Some(from) = builder.origin() {
            (
                !self.predecessors.contains(from),
                !self.successors.contains(from),
            )
        } else {
            (true, true)
        };

        if predecessor {
            if let Some((key, ())) = self.predecessors.pop_lru() {
                builder.add(&key);
                self.predecessors.put(key, ());
            }
        }

        if successor {
            if let Some((key, ())) = self.successors.pop_lru() {
                builder.add(&key);
                self.successors.put(key, ());
            }
        }
    }

    pub fn receive_gossips(&mut self, our_id: &ed25519::PublicKey, their_id: &ed25519::PublicKey) {
        match our_id.cmp(their_id) {
            Ordering::Equal => {
                // same id, we can assume this is ourselves... even though we expect
                // ourselves to be filtered out already
            }
            Ordering::Less => {
                let new_low = if let Some(low) = self.current_low.as_ref() {
                    let r = low < their_id;

                    if r {
                        self.predecessors.pop(low);
                    }

                    r
                } else {
                    true
                };

                if new_low {
                    self.current_low = Some(*their_id);
                    self.predecessors.put(*their_id, ());
                }
            }
            Ordering::Greater => {
                let new_high = if let Some(high) = self.current_max.as_ref() {
                    let r = high > their_id;

                    if r {
                        self.successors.pop(high);
                    }

                    r
                } else {
                    true
                };

                if new_high {
                    self.current_max = Some(*their_id);
                    self.successors.put(*their_id, ());
                }
            }
        }
    }
}

impl Rings {
    pub fn new(length: u8) -> Self {
        Self {
            length,
            links: lru::LruCache::new(Subscriptions::MAX_NUM_SUBSCRIPTIONS),
        }
    }

    pub fn subscriptions(&self) -> Subscriptions {
        let mut subscriptions = Subscriptions::new();

        for (topic, ring) in self.links.iter() {
            let interest_level = ring.interest_level();

            if interest_level.no_interest() {
                // there there are no interests, just ignore it
                // and move on to the next item
                continue;
            }

            let subscription = Subscription::new(*topic, interest_level);
            if subscriptions.push(subscription.as_slice()).is_err() {
                // if we have reached the limit fo the subscriptions content
                // we stop there
                break;
            }
        }

        subscriptions
    }

    fn recipients_for_event(&mut self, topic: &Topic, builder: &mut ViewBuilder) {
        if let Some(ring) = self.links.get_mut(topic) {
            ring.recipients(builder);
        }
    }

    fn recipients_for_all(&mut self, builder: &mut ViewBuilder) {
        for (_, ring) in self.links.iter_mut() {
            ring.recipients(builder);
        }
    }

    pub fn receive_gossip(
        &mut self,
        our_id: &ed25519::PublicKey,
        their_id: &ed25519::PublicKey,
        topics: impl Iterator<Item = Topic>,
    ) {
        for topic in topics {
            if let Some(ring) = self.links.get_mut(&topic) {
                ring.receive_gossips(our_id, their_id);
            }
        }
    }
}

impl Layer for Rings {
    fn name(&self) -> &'static str {
        "poldercast::rings"
    }

    fn view(&mut self, builder: &mut ViewBuilder) {
        match builder.selection() {
            Selection::Any => {
                self.recipients_for_all(builder);
            }
            Selection::Topic { topic } => {
                self.recipients_for_event(&topic, builder);
            }
        }
    }

    fn remove(&mut self, id: &ed25519::PublicKey) {
        for (_, ring) in self.links.iter_mut() {
            ring.remove(id)
        }
    }
    fn reset(&mut self) {
        self.links.clear();
    }

    fn populate(&mut self, our_profile: &Profile, new_profile: &Profile) {
        self.receive_gossip(
            &our_profile.id(),
            &new_profile.id(),
            new_profile.subscriptions().iter().map(|sub| sub.topic()),
        )
    }

    fn subscribe(&mut self, topic: Topic) {
        if !self.links.contains(&topic) {
            self.links.put(topic, Ring::new(self.length));
        }
    }
    fn unsubscribe(&mut self, topic: &Topic) {
        self.links.pop(topic);
    }
    fn subscriptions(&self, output: &mut PriorityMap<InterestLevel, Topic>) {
        for (topic, ring) in self.links.iter() {
            let interest_level = ring.interest_level();

            if interest_level.no_interest() {
                // there there are no interests, just ignore it
                // and move on to the next item
                continue;
            }

            output.put(interest_level, *topic);
        }
    }
}
