use crate::{
    layer::{self, Layer, LayerBuilder, Selection, ViewBuilder},
    Gossip, Profile, Profiles, Topic,
};
use keynesis::key::ed25519;
use std::{net::SocketAddr, sync::Arc};

pub struct Topology {
    view_layers: Vec<Box<dyn Layer>>,
    gossip_layers: Vec<Box<dyn Layer>>,
    profile: Profile,
    profiles: Profiles,
}

struct DefaultBuilder;

impl LayerBuilder for DefaultBuilder {
    fn build_for_view(&self) -> Vec<Box<dyn Layer>> {
        let mut layers: Vec<Box<dyn Layer>> = Vec::with_capacity(3);

        layers.push(Box::new(layer::Rings::new(4)));
        layers.push(Box::new(layer::Vicinity::new(20)));
        layers.push(Box::new(layer::Cyclon::new(20)));

        layers
    }

    fn build_for_gossip(&self) -> Vec<Box<dyn Layer>> {
        let mut layers: Vec<Box<dyn Layer>> = Vec::with_capacity(3);

        layers.push(Box::new(layer::Rings::new(10)));
        layers.push(Box::new(layer::Vicinity::new(10)));
        layers.push(Box::new(layer::Cyclon::new(10)));

        layers
    }
}

impl Topology {
    /// create a Topology for the given profile
    pub fn new(address: SocketAddr, id: &ed25519::SecretKey) -> Self {
        Self::new_with(address, id, DefaultBuilder)
    }

    pub fn new_with<LB>(address: SocketAddr, id: &ed25519::SecretKey, builder: LB) -> Self
    where
        LB: LayerBuilder,
    {
        let profile = Profile::new(address, id);
        Self {
            view_layers: builder.build_for_view(),
            gossip_layers: builder.build_for_gossip(),

            profile,
            profiles: Profiles::new(512, 256, 128),
        }
    }

    pub fn update_profile_subscriptions(&mut self, id: &ed25519::SecretKey) {
        self.profile.clear_subscriptions();
        for layer in self.view_layers.iter_mut() {
            layer.subscriptions(self.profile.subscriptions_mut());
        }

        self.profile.commit_gossip(id);
    }

    /// subscribe to the given topic
    ///
    /// this function also update our profile
    pub fn subscribe_topic(&mut self, topic: Topic) {
        for layer in self.view_layers.iter_mut() {
            layer.subscribe(topic);
        }
    }

    /// unsubscribe to the given topic
    ///
    /// this function also update our profile
    pub fn unsubscribe_topic(&mut self, topic: &Topic) {
        for layer in self.view_layers.iter_mut() {
            layer.unsubscribe(topic);
        }

        self.profile.unsubscribe(topic);
    }

    /// call this function if you could not establish an handshake from this
    /// peer. This will prevent to use it in the next profile update.
    ///
    /// The node will be removed from our layers, but it will not be
    /// entirely from our profile pool. We may share it to other nodes
    /// we may find it relevant
    pub fn remove_peer(&mut self, id: &ed25519::PublicKey) {
        for layer in self.view_layers.iter_mut() {
            layer.remove(id);
        }

        self.profiles.demote(id);
    }

    /// call this function to validate you were able to connect with the given
    /// peer. This will help the system make sure this entry is kept and reuse
    ///
    /// Call this function every time you successfully establish an handshake
    pub fn promote_peer(&mut self, id: &ed25519::PublicKey) {
        self.profiles.promote(id)
    }

    /// add a Peer to the Topology
    ///
    /// the peer will be considered automatically for all our layers.
    /// If the peer seem appropriate it will be added to the view and we will
    /// attempt to connect to it later.
    ///
    /// However, if the peer was already demoted some times (i.e. the peer was already
    /// known and we already know we cannot connect to it for now, it will be required
    /// to be "forgotten" or to be "promoted" in order to move away from the naughty
    /// list).
    pub fn add_peer(&mut self, peer: Profile) -> bool {
        let id = peer.id();

        let peer = Arc::new(peer);

        if !self.profiles.put(id, Arc::clone(&peer)) {
            return false;
        }

        for layer in self.view_layers.iter_mut() {
            layer.populate(&self.profile, &peer);
        }

        true
    }

    pub fn gossips_for(&mut self, recipient: &ed25519::PublicKey) -> Vec<Gossip> {
        let mut gossips = Vec::with_capacity(1024);

        let recipient = if let Some(recipient) = self.profiles.get(recipient) {
            Arc::clone(recipient)
        } else {
            return gossips;
        };

        let id = recipient.id();

        for layer in self.gossip_layers.iter_mut() {
            layer.reset();
        }

        for subscription in recipient.subscriptions().iter() {
            for layer in self.gossip_layers.iter_mut() {
                layer.subscribe(subscription.topic());
            }
        }

        for profile in self.view(None, Selection::Any) {
            for layer in self.gossip_layers.iter_mut() {
                layer.populate(recipient.as_ref(), &profile);
            }
        }

        let mut builder = ViewBuilder::new(Selection::Any);
        for layer in self.gossip_layers.iter_mut() {
            layer.view(&mut builder);
        }
        let mut keys = builder.build();

        keys.remove(&id); // remove the recipient's ID

        for key in keys {
            if let Some(profile) = self.profiles.get(&key) {
                gossips.push(profile.gossip().clone());
            } else {
                // we populated the gossip's view with the profiles' nodes
                // so we should have all the entries that have been selected
                // in the view.
                unreachable!()
            }
        }

        gossips.push(self.profile.gossip().clone());

        gossips
    }

    pub fn view(
        &mut self,
        from: Option<&ed25519::PublicKey>,
        selection: Selection,
    ) -> Vec<Arc<Profile>> {
        let mut builder = ViewBuilder::new(selection);
        if let Some(origin) = from {
            builder.with_origin(*origin);
        }

        for layer in self.view_layers.iter_mut() {
            layer.view(&mut builder);
        }

        let keys = builder.build();

        let mut profiles = Vec::with_capacity(keys.len());

        for key in keys {
            if let Some(profile) = self.profiles.get(&key) {
                profiles.push(Arc::clone(profile));
            }
        }

        profiles
    }

    pub fn get(&mut self, id: &ed25519::PublicKey) -> Option<&Arc<Profile>> {
        self.profiles.get(id)
    }

    pub fn peers(&self) -> &Profiles {
        &self.profiles
    }
}
