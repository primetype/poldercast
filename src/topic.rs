use std::collections::HashMap;

/// A topic is a unique identifier to a subject of pub/sup one node
/// is interested about.
///
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Topic([u8; 16]);

/// This is the interest associated to a topic
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InterestLevel {
    /// This describe a low interest level
    Low,
    /// This describe a normal interest level
    Normal,
    /// This describe an high interest level
    High,
}

/// This described a subscription to a topic.
#[derive(Clone, Debug)]
pub struct Subscription {
    pub topic: Topic,
    pub interest_level: InterestLevel,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Subscriptions(HashMap<Topic, InterestLevel>);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ProximityScore(usize);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PriorityScore(usize);

#[derive(Copy, Clone, Hash, Debug)]
pub struct Proximity {
    priority: PriorityScore,
    proximity: ProximityScore,
}

impl InterestLevel {
    #[inline]
    fn priority_score(&self, other: &Self) -> usize {
        use InterestLevel::*;
        match (self, other) {
            (Low, Low) => 1,
            (Low, Normal) => 2,
            (Normal, Low) => 2,
            (Low, High) => 3,
            (High, Low) => 3,
            (Normal, Normal) => 5,
            (Normal, High) => 6,
            (High, Normal) => 6,
            (High, High) => 10,
        }
    }
}

impl Subscriptions {
    pub fn new() -> Self {
        Subscriptions(HashMap::new())
    }

    /// add a new subscription, return the replaced/updated subscription if
    /// topic already present.
    pub fn add(&mut self, subscription: Subscription) -> Option<InterestLevel> {
        self.0
            .insert(subscription.topic, subscription.interest_level)
    }

    pub fn contains(&self, topic: &Topic) -> bool {
        self.0.contains_key(topic)
    }

    pub fn remove(&mut self, subscription: &Topic) -> Option<InterestLevel> {
        self.0.remove(subscription)
    }

    pub fn topics<'a>(&'a self) -> impl Iterator<Item = &'a Topic> {
        self.0.keys()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a Topic, &'a InterestLevel)> {
        self.0.iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (&'a Topic, &'a mut InterestLevel)> {
        self.0.iter_mut()
    }

    /// retrieve the iterator over the topics common between both subscriptions
    pub fn common_subscriptions<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a Topic> {
        self.0
            .keys()
            .filter(move |topic| other.0.contains_key(topic))
    }

    pub fn proximity_to(&self, other: &Self) -> Proximity {
        let mut priority_score = 0;
        let mut proximity_score = 0;
        for (subscription, interest_level) in self.iter() {
            if let Some(other_interest_level) = other.0.get(subscription) {
                proximity_score += 1;
                priority_score += interest_level.priority_score(&other_interest_level);
            }
        }
        Proximity {
            proximity: ProximityScore(proximity_score),
            priority: PriorityScore(priority_score),
        }
    }
}

impl Subscription {
    pub fn new(topic: Topic, interest_level: InterestLevel) -> Self {
        Subscription {
            topic,
            interest_level,
        }
    }
}

/* *****************************************************************
 * we provide custom comparison implementation for the Subscription
 *
 * This is to accommodate the `Subscriptions` type (and the inner
 * operations).
 */

impl PartialEq<Topic> for Subscription {
    fn eq(&self, topic: &Topic) -> bool {
        &self.topic == topic
    }
}
impl PartialEq<Self> for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.topic == other.topic
    }
}
impl Eq for Subscription {}
impl PartialOrd<Topic> for Subscription {
    fn partial_cmp(&self, topic: &Topic) -> Option<std::cmp::Ordering> {
        self.topic.partial_cmp(topic)
    }
}
impl PartialOrd<Self> for Subscription {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.topic.partial_cmp(&other.topic)
    }
}
impl Ord for Subscription {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.topic.cmp(&other.topic)
    }
}
impl std::hash::Hash for Subscription {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        std::hash::Hash::hash(&self.topic, state)
    }
}

impl PartialEq<Self> for Proximity {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}
impl Eq for Proximity {}
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

/* ****************************** From ********************************* */

impl From<u128> for Topic {
    fn from(v: u128) -> Self {
        Topic(v.to_le_bytes())
    }
}
impl From<[u8; 16]> for Topic {
    fn from(v: [u8; 16]) -> Self {
        Topic(v)
    }
}

/* ****************************** AsRef ********************************* */

impl AsRef<[u8]> for Topic {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Topic {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Topic::from(u128::arbitrary(g))
        }
    }

    impl Arbitrary for InterestLevel {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match u8::arbitrary(g) % 3 {
                0 => InterestLevel::Low,
                1 => InterestLevel::Normal,
                _ => InterestLevel::High,
            }
        }
    }

    impl Arbitrary for Subscription {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Subscription {
                topic: Topic::arbitrary(g),
                interest_level: InterestLevel::arbitrary(g),
            }
        }
    }

    impl Arbitrary for Subscriptions {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let subscriptions: Vec<Subscription> = Arbitrary::arbitrary(g);

            let mut subs = Subscriptions::new();
            for subscription in subscriptions {
                subs.add(subscription);
            }
            subs
        }
    }
}
