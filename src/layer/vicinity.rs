use crate::{
    layer::{Layer, ViewBuilder},
    profile::Proximity,
    InterestLevel, PriorityMap, Profile, Topic,
};
use keynesis::key::ed25519;

pub struct Vicinity {
    nodes: PriorityMap<Proximity, ed25519::PublicKey>,
}

impl Vicinity {
    pub fn new(length: usize) -> Self {
        Self {
            nodes: PriorityMap::new(length),
        }
    }
}

impl Layer for Vicinity {
    fn name(&self) -> &'static str {
        "poldercast::vicinity"
    }

    fn view(&mut self, builder: &mut ViewBuilder) {
        self.nodes.iter().for_each(|(_, v)| builder.add(v));
    }

    fn remove(&mut self, id: &ed25519::PublicKey) {
        self.nodes.remove(id);
    }
    fn reset(&mut self) {
        self.nodes.clear();
    }

    fn populate(&mut self, our_profile: &Profile, new_profile: &Profile) {
        let proximity = our_profile.proximity_to(new_profile);
        self.nodes.put(proximity, new_profile.id());
    }

    fn subscribe(&mut self, _: Topic) {}

    fn unsubscribe(&mut self, _: &Topic) {}

    fn subscriptions(&self, _output: &mut PriorityMap<InterestLevel, Topic>) {}
}
