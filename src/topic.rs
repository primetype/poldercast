use std::{
    convert::{TryFrom, TryInto as _},
    fmt::{self, Formatter},
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Topic([u8; Self::SIZE]);

/// This is the interest associated to a topic
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct InterestLevel(u8);

#[derive(Clone, Copy)]
pub struct SubscriptionSlice<'a>(&'a [u8]);

#[derive(Clone, Copy)]
pub struct Subscription([u8; Self::SIZE]);

#[derive(Clone)]
pub struct Subscriptions(Vec<u8>);

#[derive(Clone, Copy)]
pub struct SubscriptionsSlice<'a>(&'a [u8]);

pub struct SubscriptionIter<'a>(SubscriptionsSlice<'a>);

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error("Invalid, length of a subscription, expected {}", Subscription::SIZE)]
    InvalidSize,

    #[error("Invalid subscription ({index})")]
    InvalidSubscriptionAt { index: usize },

    #[error(
        "Cannot have more than {} Subscriptions",
        Subscriptions::MAX_NUM_SUBSCRIPTIONS
    )]
    MaxSubscriptionReached,
}

impl Topic {
    pub const SIZE: usize = 32;

    pub const fn new(topic: [u8; Self::SIZE]) -> Self {
        Self(topic)
    }
}

impl InterestLevel {
    pub const SIZE: usize = 1;

    pub const ZERO: Self = Self::new(0);

    pub const fn new(level: u8) -> Self {
        Self(level)
    }

    pub fn priority_score(self, other: Self) -> usize {
        if self < other {
            self.0 as usize
        } else {
            self.0 as usize + other.0 as usize
        }
    }

    #[inline(always)]
    pub fn no_interest(self) -> bool {
        self == Self::ZERO
    }
}

impl Subscription {
    pub const SIZE: usize = Topic::SIZE + InterestLevel::SIZE;

    pub fn new(topic: Topic, interest_level: InterestLevel) -> Self {
        let mut sub = [0; Self::SIZE];

        sub[..Topic::SIZE].copy_from_slice(topic.as_ref());
        sub[Topic::SIZE] = interest_level.0;

        Self(sub)
    }

    pub fn as_slice(&self) -> SubscriptionSlice<'_> {
        SubscriptionSlice::from_slice_unchecked(self.0.as_ref())
    }

    pub fn topic(&self) -> Topic {
        self.as_slice().topic()
    }

    pub fn interest_level(&self) -> InterestLevel {
        self.as_slice().interest_level()
    }
}

impl<'a> SubscriptionSlice<'a> {
    pub fn to_owned(self) -> Subscription {
        Subscription(self.0.try_into().expect("Valid Subscription slice"))
    }

    pub fn try_from_slice(slice: &'a [u8]) -> Result<Self, SubscriptionError> {
        if slice.len() != Subscription::SIZE {
            return Err(SubscriptionError::InvalidSize);
        }

        Ok(Self::from_slice_unchecked(slice))
    }

    pub fn from_slice_unchecked(slice: &'a [u8]) -> Self {
        debug_assert_eq!(slice.len(), Subscription::SIZE);
        Self(slice)
    }

    pub fn topic(self) -> Topic {
        Topic(
            self.0[..Topic::SIZE]
                .as_ref()
                .try_into()
                .expect("32 bytes of Topic identifier"),
        )
    }

    pub fn interest_level(self) -> InterestLevel {
        InterestLevel(self.0[Topic::SIZE])
    }
}

impl Subscriptions {
    pub const MAX_NUM_SUBSCRIPTIONS: usize = 0b0000_0011_1111_1111; // 1023

    pub fn new() -> Self {
        Self(Vec::with_capacity(
            Self::MAX_NUM_SUBSCRIPTIONS * Subscription::SIZE,
        ))
    }

    pub fn push(&mut self, sub: SubscriptionSlice<'_>) -> Result<(), SubscriptionError> {
        if self.as_slice().number_subscriptions() >= Self::MAX_NUM_SUBSCRIPTIONS {
            return Err(SubscriptionError::MaxSubscriptionReached);
        }

        self.0.extend_from_slice(sub.as_ref());

        Ok(())
    }

    pub fn as_slice(&self) -> SubscriptionsSlice<'_> {
        SubscriptionsSlice(self.0.as_ref())
    }

    pub fn iter(&self) -> SubscriptionIter<'_> {
        self.as_slice().iter()
    }
}

impl<'a> SubscriptionsSlice<'a> {
    pub fn to_owned(self) -> Subscriptions {
        Subscriptions(self.0.to_owned())
    }

    pub fn try_from_slice(slice: &'a [u8]) -> Result<Self, SubscriptionError> {
        if slice.len() % Subscription::SIZE != 0 {
            return Err(SubscriptionError::InvalidSize);
        }

        let slice = Self::from_slice_unchecked(slice);

        if slice.number_subscriptions() > Subscriptions::MAX_NUM_SUBSCRIPTIONS {
            return Err(SubscriptionError::MaxSubscriptionReached);
        }

        for (index, entry) in slice.iter().enumerate() {
            let slice = entry.0;
            let _ = SubscriptionSlice::try_from_slice(slice)
                .map_err(|_| SubscriptionError::InvalidSubscriptionAt { index })?;
        }

        Ok(slice)
    }

    pub fn from_slice_unchecked(slice: &'a [u8]) -> Self {
        debug_assert_eq!(slice.len() % Subscription::SIZE, 0);
        Self(slice)
    }

    fn subscription_offset(self, index: usize) -> usize {
        index * Subscription::SIZE
    }

    pub fn number_subscriptions(self) -> usize {
        self.0.len() / Subscription::SIZE
    }

    pub fn iter(self) -> SubscriptionIter<'a> {
        SubscriptionIter(self)
    }

    pub fn pop_front(&mut self) -> Option<SubscriptionSlice<'a>> {
        let obj = self.get(0)?;

        self.0 = &self.0[Subscription::SIZE..];

        Some(obj)
    }

    pub fn pop_back(&mut self) -> Option<SubscriptionSlice<'a>> {
        let index = self.number_subscriptions();
        let sub = self.get(index)?;

        self.0 = &self.0[..index];

        Some(sub)
    }

    pub fn get(self, index: usize) -> Option<SubscriptionSlice<'a>> {
        let len = self.number_subscriptions();
        if len == 0 || len < index {
            None
        } else {
            let index = self.subscription_offset(index);

            Some(SubscriptionSlice::from_slice_unchecked(
                &self.0[index..index + Subscription::SIZE],
            ))
        }
    }
}

/* Default ***************************************************************** */

impl Default for Subscriptions {
    fn default() -> Self {
        Self::new()
    }
}

/* Convert ***************************************************************** */

impl<'a> TryFrom<&'a [u8]> for Topic {
    type Error = std::array::TryFromSliceError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let bytes = value.try_into()?;
        Ok(Topic::new(bytes))
    }
}

impl FromStr for Topic {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut topic = [0; Topic::SIZE];
        hex::decode_to_slice(s, &mut topic)?;
        Ok(Self(topic))
    }
}

/* AsRef ******************************************************************* */

impl<'a> AsRef<[u8]> for SubscriptionSlice<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> AsRef<[u8]> for SubscriptionsSlice<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl AsRef<[u8]> for Subscription {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for Topic {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/* Formatter *************************************************************** */

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        hex::encode(self.as_ref()).fmt(f)
    }
}

impl fmt::Debug for Topic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Topic")
            .field(&hex::encode(self.as_ref()))
            .finish()
    }
}

impl<'a> fmt::Debug for SubscriptionSlice<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Subscription")
            .field("topic", &self.topic())
            .field("interest", &self.interest_level())
            .finish()
    }
}

impl fmt::Debug for Subscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl fmt::Debug for Subscriptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<'a> fmt::Debug for SubscriptionsSlice<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/* Iterator **************************************************************** */

impl<'a> Iterator for SubscriptionIter<'a> {
    type Item = SubscriptionSlice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let r = self.0.number_subscriptions();
        (r, Some(r))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.number_subscriptions()
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.0.pop_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.0.subscription_offset(n);
        let sub = self.0.get(n)?;

        (self.0).0 = &(self.0).0[index..];

        Some(sub)
    }
}
impl<'a> DoubleEndedIterator for SubscriptionIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}
impl<'a> ExactSizeIterator for SubscriptionIter<'a> {
    fn len(&self) -> usize {
        self.0.number_subscriptions()
    }
}
impl<'a> FusedIterator for SubscriptionIter<'a> {}
impl<'a> IntoIterator for SubscriptionsSlice<'a> {
    type IntoIter = SubscriptionIter<'a>;
    type Item = SubscriptionSlice<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Topic {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut topic = Topic::new([0; Self::SIZE]);
            topic.0.iter_mut().for_each(|byte| {
                *byte = u8::arbitrary(g);
            });
            topic
        }
    }

    impl Arbitrary for InterestLevel {
        fn arbitrary(g: &mut Gen) -> Self {
            InterestLevel::new(u8::arbitrary(g))
        }
    }

    impl Arbitrary for Subscription {
        fn arbitrary(g: &mut Gen) -> Self {
            Self::new(Topic::arbitrary(g), InterestLevel::arbitrary(g))
        }
    }

    impl Arbitrary for Subscriptions {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut subs = Self::new();
            let count = usize::arbitrary(g) % Subscriptions::MAX_NUM_SUBSCRIPTIONS;

            for _ in 0..count {
                subs.push(Subscription::arbitrary(g).as_slice())
                    .expect("There should be enough space to add all the needed subscriptions");
            }

            subs
        }
    }

    /// make sure we are reaching an error if we are creating a subscriptions with too
    /// many entries
    #[test]
    fn subscriptions_push_max() {
        let mut subs = Subscriptions::new();
        let mut g = quickcheck::Gen::new(1024);
        let g = &mut g;

        let count = Subscriptions::MAX_NUM_SUBSCRIPTIONS;
        for _ in 0..count {
            subs.push(Subscription::arbitrary(g).as_slice())
                .expect("There should be enough space to add all the needed subscriptions");
        }

        subs.push(Subscription::arbitrary(g).as_slice())
            .expect_err("Should have failed as reaching the limits");
    }

    /// make sure we are reaching an error if we are creating a subscriptions with too
    /// many entries
    #[test]
    fn subscriptions_decode_max() {
        let mut subs = vec![0; Subscription::SIZE * (Subscriptions::MAX_NUM_SUBSCRIPTIONS + 1)];
        let mut g = quickcheck::Gen::new(1024);
        let g = &mut g;

        let count = Subscriptions::MAX_NUM_SUBSCRIPTIONS + 1;
        for _ in 0..count {
            subs.extend_from_slice(Subscription::arbitrary(g).as_ref());
        }

        SubscriptionsSlice::try_from_slice(subs.as_slice())
            .expect_err("Should have a max size reached error");
    }

    #[test]
    fn topic_from_str() {
        let topic = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

        let _topic = Topic::from_str(topic).unwrap();
    }

    #[test]
    fn topic_to_string() {
        let topic = [
            1, 35, 69, 103, 137, 171, 205, 239, 1, 35, 69, 103, 137, 171, 205, 239, 1, 35, 69, 103,
            137, 171, 205, 239, 1, 35, 69, 103, 137, 171, 205, 239,
        ];
        let topic = Topic::new(topic).to_string();

        debug_assert_eq!(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            topic,
        )
    }

    #[quickcheck]
    fn parse_valid_subscription(sub: Subscription) -> bool {
        let slice = sub.as_slice();
        let _ = SubscriptionSlice::try_from_slice(slice.as_ref()).unwrap();
        true
    }

    #[quickcheck]
    fn parse_valid_subscriptions(subs: Subscriptions) -> bool {
        let slice = subs.as_slice();
        let _ = SubscriptionsSlice::try_from_slice(slice.as_ref()).unwrap();
        true
    }

    #[quickcheck]
    fn to_string_from_str(topic: Topic) -> bool {
        let s = topic.to_string();
        let t: Topic = s.parse().unwrap();

        topic == t
    }
}
