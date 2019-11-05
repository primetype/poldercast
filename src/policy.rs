use crate::Node;
use std::{collections::VecDeque, time::SystemTime};

/// default quarantine duration is 30min
const DEFAULT_QUARANTINE_DURATION: u64 = 1800;

pub struct DefaultPolicy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strike {
    when: SystemTime,

    reason: StrikeReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrikeReason {
    CannotConnect,
    InvalidPublicId,
    InvalidData,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Record {
    strikes: VecDeque<Strike>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyReport {
    None,
    Quarantine,
    LiftQuarantine,
    Forget,
}

pub trait Policy {
    fn check(&mut self, node: &mut Node) -> PolicyReport;
}

impl Default for DefaultPolicy {
    fn default() -> Self {
        DefaultPolicy
    }
}

impl<P: Policy + ?Sized> Policy for Box<P> {
    fn check(&mut self, node: &mut Node) -> PolicyReport {
        self.as_mut().check(node)
    }
}

impl Policy for DefaultPolicy {
    fn check(&mut self, node: &mut Node) -> PolicyReport {
        // if the node is already quarantined
        if let Some(since) = node.logs().quarantined() {
            let duration = since.elapsed().unwrap();

            if duration < std::time::Duration::from_secs(DEFAULT_QUARANTINE_DURATION) {
                // the node still need to do some quarantine time
                PolicyReport::None
            } else if node.logs().last_update().elapsed().unwrap()
                < std::time::Duration::from_secs(DEFAULT_QUARANTINE_DURATION)
            {
                // the node has been quarantined long enough, check if it has been updated
                // while being quarantined (i.e. the node is still up and advertising itself
                // or others are still gossiping about it.)

                // the fact that this `Policy` does clean the records is a policy choice.
                // one could prefer to keep the record longers for future `check`.
                node.record_mut().clean_slate();
                PolicyReport::LiftQuarantine
            } else {
                // it appears the node was quarantine and is no longer active or gossiped
                // about, so we can forget it
                PolicyReport::Forget
            }
        } else if node.record().is_clear() {
            // if the record is clear, do nothing, leave the Node in the available nodes
            PolicyReport::None
        } else {
            // if the record is not `clear` then we quarantine the block for some time
            PolicyReport::Quarantine
        }
    }
}

impl Record {
    /// returns `true` if the record show that the `Node` is clear
    /// of any strike
    pub fn is_clear(&self) -> bool {
        self.strikes.is_empty()
    }

    pub fn clean_slate(&mut self) {
        self.strikes = VecDeque::default()
    }

    pub fn strike(&mut self, reason: StrikeReason) {
        self.strikes.push_back(Strike {
            when: SystemTime::now(),
            reason,
        })
    }
}
