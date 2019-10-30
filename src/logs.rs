use crate::Topic;
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logs {
    creation_time: SystemTime,
    last_update: SystemTime,
    last_gossip: SystemTime,

    quarantined: Option<SystemTime>,

    last_use_of: HashMap<Topic, SystemTime>,
}

impl Logs {
    pub fn creation_time(&self) -> &SystemTime {
        &self.creation_time
    }

    pub fn last_update(&self) -> &SystemTime {
        &self.last_update
    }

    pub fn last_gossip(&self) -> &SystemTime {
        &self.last_gossip
    }

    pub fn last_use_of(&self, topic: Topic) -> Option<&SystemTime> {
        self.last_use_of.get(&topic)
    }

    pub fn quarantined(&self) -> Option<&SystemTime> {
        self.quarantined.as_ref()
    }

    pub(crate) fn gossiping(&mut self) {
        self.last_gossip = SystemTime::now()
    }

    pub(crate) fn updated(&mut self) {
        self.last_update = SystemTime::now()
    }

    pub(crate) fn quarantine(&mut self) {
        self.quarantined = Some(SystemTime::now())
    }

    pub(crate) fn lift_quarantine(&mut self) {
        self.quarantined = None
    }

    pub(crate) fn use_of(&mut self, topic: Topic) {
        self.last_use_of.insert(topic, SystemTime::now());
    }
}

impl Default for Logs {
    fn default() -> Self {
        let now = SystemTime::now();
        Self {
            creation_time: now,
            last_update: now,
            last_gossip: now,

            quarantined: None,
            last_use_of: HashMap::default(),
        }
    }
}
