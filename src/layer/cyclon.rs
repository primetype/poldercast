use crate::{
    layer::{Layer, ViewBuilder},
    InterestLevel, PriorityMap, Profile, Topic,
};
use keynesis::key::ed25519;

pub struct Cyclon {
    nodes: lru::LruCache<ed25519::PublicKey, ()>,
}

impl Cyclon {
    pub fn new(length: usize) -> Self {
        Self {
            nodes: lru::LruCache::new(length),
        }
    }
}

impl Layer for Cyclon {
    fn name(&self) -> &'static str {
        "poldercast::cyclon"
    }

    fn view(&mut self, builder: &mut ViewBuilder) {
        self.nodes.iter().for_each(|(k, _)| builder.add(k));
    }

    fn remove(&mut self, id: &ed25519::PublicKey) {
        self.nodes.pop(id);
    }
    fn reset(&mut self) {
        self.nodes.clear();
    }

    fn populate(&mut self, _our_profile: &Profile, new_profile: &Profile) {
        self.nodes.put(new_profile.id(), ());
    }

    fn subscribe(&mut self, _topic: Topic) {}

    fn unsubscribe(&mut self, _topic: &Topic) {}

    fn subscriptions(&self, _output: &mut PriorityMap<InterestLevel, Topic>) {}
}
