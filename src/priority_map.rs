use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::{btree_map, hash_map::RandomState, BTreeMap, HashMap},
    hash::{BuildHasher, Hash, Hasher},
};

struct Entry<K, V> {
    key: K,
    value: V,
}

#[doc(hidden)]
pub struct KeyRef<K> {
    k: *const K,
}

type PriorityGroup<K, V> = lru::LruCache<KeyRef<V>, *mut Entry<K, V>>;

pub struct PriorityMap<K, V, H = RandomState> {
    by_value: HashMap<KeyRef<V>, Box<Entry<K, V>>, H>,
    by_priority: BTreeMap<KeyRef<K>, PriorityGroup<K, V>>,
    cap: usize,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, value: V) -> Entry<K, V> {
        Self { key, value }
    }
}

impl<K, V> PriorityMap<K, V>
where
    K: Ord + Clone,
    V: Eq + Clone + Hash,
{
    pub fn new(cap: usize) -> Self {
        PriorityMap::new_with_map(cap, HashMap::with_capacity(cap))
    }
}

impl<K, V, H> PriorityMap<K, V, H>
where
    K: Ord + Clone,
    V: Eq + Clone + Hash,
    H: BuildHasher,
{
    pub fn new_with(cap: usize, hash_builder: H) -> Self {
        Self::new_with_map(cap, HashMap::with_capacity_and_hasher(cap, hash_builder))
    }

    fn new_with_map(cap: usize, by_value: HashMap<KeyRef<V>, Box<Entry<K, V>>, H>) -> Self {
        assert!(
            cap > 0,
            "Cannot do much with a cap set to 0, have at least 1 entry or use a different type"
        );
        Self {
            cap,
            by_value,
            by_priority: BTreeMap::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.by_value.capacity()
    }

    pub fn len(&self) -> usize {
        self.by_value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_value.is_empty()
    }

    pub fn contains<Q>(&self, k: &Q) -> bool
    where
        KeyRef<V>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.by_value.contains_key(k)
    }

    pub fn remove<Q>(&mut self, v: &Q) -> bool
    where
        KeyRef<V>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(mut entry) = self.by_value.remove(v) {
            let keyref: *mut K = &mut entry.key;
            let k = KeyRef { k: keyref };
            let keyref: *mut V = &mut entry.value;
            let v = KeyRef { k: keyref };

            if let btree_map::Entry::Occupied(mut occupied) = self.by_priority.entry(k) {
                occupied.get_mut().pop(&v);

                // make sure we don't keep empty priority entries
                if occupied.get().is_empty() {
                    occupied.remove();
                }
            }
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'_ K, &'_ V)> {
        self.by_priority
            .values()
            .rev()
            .flat_map(|v| v.iter())
            .map(|(_, v)| unsafe { (&(**v).key, &(**v).value) })
    }

    pub fn get<Q>(&self, k: &Q) -> Option<(&'_ K, &'_ V)>
    where
        KeyRef<V>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let e = self.by_value.get(k)?;

        Some((&e.key, &e.value))
    }

    pub fn put(&mut self, key: K, value: V) {
        // check if we have reached the cap
        if self.len() >= self.cap {
            // if we do check that we are not adding an entry that is lower bound
            // than our current lower priority
            //
            // if this is the case, return now and don't add the entry
            if let Some((lowest, _)) = self.by_priority.iter().next() {
                if unsafe { lowest.k.as_ref() }.unwrap() > &key {
                    return;
                }
            }

            while self.len() >= self.cap {
                self.pop_lowest();
            }
        }

        let entry = Entry::new(key, value);
        let mut entry = Box::new(entry);
        let entry_ptr: *mut Entry<K, V> = &mut *entry;
        let keyref: *mut K = unsafe { &mut (*entry_ptr).key };
        let k = KeyRef { k: keyref };
        let keyref: *mut V = unsafe { &mut (*entry_ptr).value };
        let v = KeyRef { k: keyref };

        if let Some(mut prev) = self.by_value.insert(v, entry) {
            let keyref: *mut V = &mut prev.value;
            let v = KeyRef { k: keyref };

            // if we have updated the priority of the value we need to also change it
            // in the previous version of the `by_priority`
            if let btree_map::Entry::Occupied(mut occupied) = self.by_priority.entry(k) {
                occupied.get_mut().pop(&v);

                // make sure we don't keep empty priority entries
                if occupied.get().is_empty() {
                    occupied.remove();
                }
            }
        }
        self.by_priority
            .entry(k)
            .or_insert_with(lru::LruCache::unbounded)
            .put(v, entry_ptr);
    }

    pub fn resize(&mut self, cap: usize) {
        // return early if capacity doesn't change
        if cap == self.cap {
            return;
        }

        while self.len() > cap {
            self.pop_lowest();
        }
        self.by_value.shrink_to_fit();

        self.cap = cap;
    }

    fn lower_bound(&self) -> Option<KeyRef<K>> {
        if let Some((k, _)) = self.by_priority.iter().next() {
            Some(KeyRef { k: k.k })
        } else {
            None
        }
    }

    pub fn pop_lowest(&mut self) -> Option<(K, V)> {
        let k = self.lower_bound()?;

        if let btree_map::Entry::Occupied(mut occupied) = self.by_priority.entry(k) {
            let lowest = occupied.get_mut().pop_lru();

            // make sure we don't keep empty priority entries
            if occupied.get().is_empty() {
                occupied.remove();
            }

            let (v, _) = lowest?;
            let entry = self.by_value.remove(&v)?;

            Some((entry.key.clone(), entry.value.clone()))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.by_priority.clear();
        self.by_value.clear();
    }
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &KeyRef<K>) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

impl<K: PartialOrd> PartialOrd for KeyRef<K> {
    fn partial_cmp(&self, other: &KeyRef<K>) -> Option<Ordering> {
        unsafe { (*self.k).partial_cmp(&*other.k) }
    }
}

impl<K: Ord> Ord for KeyRef<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { (*self.k).cmp(&*other.k) }
    }
}

impl<K> Borrow<K> for KeyRef<K> {
    fn borrow(&self) -> &K {
        unsafe { &*self.k }
    }
}

impl<K> Clone for KeyRef<K> {
    fn clone(&self) -> Self {
        Self { k: self.k }
    }
}
impl<K> Copy for KeyRef<K> {}

unsafe impl<K: Send, V: Send> Send for PriorityMap<K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for PriorityMap<K, V> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut map = PriorityMap::new(10);
        assert!(map.is_empty());

        map.put(1, "entry".to_owned());
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn contains() {
        let mut map = PriorityMap::<u32, String>::new(10);
        map.put(1, "entry".to_owned());

        assert!(map.contains(&"entry".to_owned()));
    }

    #[test]
    fn ignoring_lower_than_lower_bound() {
        let mut map = PriorityMap::<u32, String>::new(5);
        map.put(3, "3".to_owned());
        map.put(2, "2".to_owned());
        map.put(5, "5".to_owned());
        map.put(5, "five".to_owned());
        map.put(6, "6".to_owned());
        map.put(4, "4".to_owned());
        map.put(1, "1".to_owned());

        let mut iter = map.iter();

        assert_eq!(iter.next(), Some((&6u32, &"6".to_owned())));
        assert_eq!(iter.next(), Some((&5u32, &"five".to_owned())));
        assert_eq!(iter.next(), Some((&5u32, &"5".to_owned())));
        assert_eq!(iter.next(), Some((&4u32, &"4".to_owned())));
        assert_eq!(iter.next(), Some((&3u32, &"3".to_owned())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ordering() {
        let mut map = PriorityMap::<u32, String>::new(10);
        map.put(3, "3".to_owned());
        map.put(1, "1".to_owned());
        map.put(2, "2".to_owned());
        map.put(5, "5".to_owned());
        map.put(5, "five".to_owned());
        map.put(6, "6".to_owned());
        map.put(4, "4".to_owned());

        let mut iter = map.iter();

        assert_eq!(iter.next(), Some((&6u32, &"6".to_owned())));
        assert_eq!(iter.next(), Some((&5u32, &"five".to_owned())));
        assert_eq!(iter.next(), Some((&5u32, &"5".to_owned())));
        assert_eq!(iter.next(), Some((&4u32, &"4".to_owned())));
        assert_eq!(iter.next(), Some((&3u32, &"3".to_owned())));
        assert_eq!(iter.next(), Some((&2u32, &"2".to_owned())));
        assert_eq!(iter.next(), Some((&1u32, &"1".to_owned())));
        assert_eq!(iter.next(), None);
    }
}
