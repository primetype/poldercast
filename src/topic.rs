use serde::{Deserialize, Serialize};
use std::{
    cmp,
    collections::BTreeSet,
    hash::{Hash, Hasher},
    iter::FromIterator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Topic(u32);

/// This is the interest associated to a topic
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum InterestLevel {
    /// This describe a low interest level
    Low,
    /// This describe a normal interest level
    Normal,
    /// This describe an high interest level
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Subscription {
    pub topic: Topic,
    pub interest: InterestLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Subscriptions(BTreeSet<Subscription>);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ProximityScore(usize);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PriorityScore(usize);

#[derive(Copy, Clone, Debug)]
pub struct Proximity {
    priority: PriorityScore,
    proximity: ProximityScore,
}

impl Topic {
    /// create a new `Topic` value
    pub const fn new(value: u32) -> Self {
        Topic(value)
    }
}

impl Subscriptions {
    pub fn iter(&self) -> impl Iterator<Item = &Subscription> {
        self.0.iter()
    }

    pub fn insert(&mut self, subscription: Subscription) {
        self.0.insert(subscription);
    }

    pub fn contains(&self, topic: Topic) -> bool {
        self.0.contains(&Subscription {
            topic,
            interest: InterestLevel::Normal,
        })
    }

    /// retrieve the iterator over the topics common between both subscriptions
    pub fn common_subscriptions<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = &'a Subscription> {
        self.0.intersection(&other.0)
    }

    pub fn proximity_to(&self, other: &Self) -> Proximity {
        let mut priority_score = 0;
        let mut proximity_score = 0;
        for subscription in self.iter() {
            if let Some(other_subscription) = other.0.get(subscription) {
                proximity_score += 1;
                priority_score += subscription
                    .interest()
                    .priority_score(other_subscription.interest());
            }
        }
        Proximity {
            proximity: ProximityScore(proximity_score),
            priority: PriorityScore(priority_score),
        }
    }
}

impl Subscription {
    pub fn topic(self) -> Topic {
        self.topic
    }

    pub fn interest(self) -> InterestLevel {
        self.interest
    }
}

impl InterestLevel {
    #[inline]
    fn priority_score(self, other: Self) -> usize {
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

/* Comparison *************************************************************** */

impl Hash for Subscription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.topic.hash(state)
    }
}

impl PartialEq<Self> for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.topic.eq(&other.topic)
    }
}

impl Eq for Subscription {}

impl PartialOrd<Self> for Subscription {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Subscription {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.topic.cmp(&other.topic)
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

/* Convert ****************************************************************** */

impl From<u32> for Topic {
    fn from(v: u32) -> Self {
        Topic(v)
    }
}

/* Iterator ***************************************************************** */

impl FromIterator<Subscription> for Subscriptions {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Subscription>,
    {
        Subscriptions(BTreeSet::from_iter(iter))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Topic {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Topic(u32::arbitrary(g))
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
                interest: InterestLevel::arbitrary(g),
            }
        }
    }

    impl Arbitrary for Subscriptions {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Subscriptions(BTreeSet::arbitrary(g))
        }
    }

    #[quickcheck]
    fn topic_encode_decode_json(topic: Topic) -> bool {
        let encoded = serde_json::to_string(&topic).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        topic == decoded
    }

    #[quickcheck]
    fn topic_encode_decode_bincode(topic: Topic) -> bool {
        let encoded = bincode::serialize(&topic).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        topic == decoded
    }

    #[quickcheck]
    fn subscription_encode_decode_json(subscription: Subscription) -> bool {
        let encoded = serde_json::to_string(&subscription).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        subscription == decoded
    }

    #[quickcheck]
    fn subscription_encode_decode_bincode(subscription: Subscription) -> bool {
        let encoded = bincode::serialize(&subscription).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        subscription == decoded
    }

    #[quickcheck]
    fn subscriptions_encode_decode_json(subscriptions: Subscriptions) -> bool {
        let encoded = serde_json::to_string(&subscriptions).unwrap();
        let decoded = serde_json::from_str(&encoded).unwrap();
        subscriptions == decoded
    }

    #[quickcheck]
    fn subscriptions_encode_decode_bincode(subscriptions: Subscriptions) -> bool {
        let encoded = bincode::serialize(&subscriptions).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        subscriptions == decoded
    }
}
