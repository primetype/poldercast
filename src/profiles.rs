use crate::Profile;
use keynesis::key::ed25519;
use lru::LruCache;
use std::sync::Arc;

pub struct Profiles {
    pub dirty: LruCache<ed25519::PublicKey, Arc<Profile>>,
    pub pool: LruCache<ed25519::PublicKey, Arc<Profile>>,
    pub trusted: LruCache<ed25519::PublicKey, Arc<Profile>>,
}

impl Profiles {
    pub fn new(dirty: usize, pool: usize, trusted: usize) -> Self {
        Self {
            dirty: LruCache::new(dirty),
            pool: LruCache::new(pool),
            trusted: LruCache::new(trusted),
        }
    }

    pub fn promote(&mut self, entry: &ed25519::PublicKey) {
        if let Some(profile) = self.pool.pop(entry) {
            // if there is an overflow coming up, instead of losing
            // the entries we would rotate from the trusted LRU
            // we demote the least used to the lower pool
            while self.trusted.len() >= self.trusted.cap() {
                if let Some((id, profile)) = self.trusted.pop_lru() {
                    self.pool.put(id, profile);
                } else {
                    unreachable!("cap should be greater than 0")
                }
            }

            self.trusted.put(*entry, profile);
        }

        if let Some(profile) = self.dirty.pop(entry) {
            self.pool.put(*entry, profile);
        }
    }

    pub fn demote(&mut self, entry: &ed25519::PublicKey) {
        if let Some(profile) = self.pool.pop(entry) {
            self.dirty.put(*entry, profile);
        } else if let Some(profile) = self.trusted.pop(entry) {
            self.pool.put(*entry, profile);
        }
    }

    pub fn put(&mut self, id: ed25519::PublicKey, profile: Arc<Profile>) -> bool {
        if let Some(entry) = self.dirty.peek(&id).cloned() {
            if entry.last_update() < profile.last_update() {
                self.dirty.put(id, profile);
            }
            false
        } else if let Some(entry) = self.trusted.peek(&id).cloned() {
            if entry.last_update() < profile.last_update() {
                self.trusted.put(id, profile);
                true
            } else {
                false
            }
        } else if let Some(entry) = self.pool.peek(&id).cloned() {
            if entry.last_update() < profile.last_update() {
                self.pool.put(id, profile);
                true
            } else {
                false
            }
        } else {
            self.pool.put(id, profile);
            true
        }
    }

    pub fn get(&mut self, id: &ed25519::PublicKey) -> Option<&Arc<Profile>> {
        if let Some(profile) = self.trusted.get(id) {
            Some(profile)
        } else if let Some(profile) = self.pool.get(id) {
            Some(profile)
        } else if let Some(profile) = self.dirty.get(id) {
            Some(profile)
        } else {
            None
        }
    }
}

impl Default for Profiles {
    fn default() -> Self {
        Self::new(512, 256, 128)
    }
}
