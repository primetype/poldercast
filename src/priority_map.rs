use std::{
    borrow::Borrow,
    collections::{btree_map, hash_map::RandomState, BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
    ptr::NonNull,
    rc::Rc,
};

#[derive(Debug)]
struct Entry<K, V> {
    key: Rc<K>,
    value: Rc<V>,
}

type PriorityGroup<K, V> = lru::LruCache<Rc<V>, NonNull<Entry<K, V>>>;

pub struct PriorityMap<K, V, H = RandomState> {
    by_value: HashMap<Rc<V>, Box<Entry<K, V>>, H>,
    by_priority: BTreeMap<Rc<K>, PriorityGroup<K, V>>,
    cap: usize,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, value: V) -> Entry<K, V> {
        Self {
            key: Rc::new(key),
            value: Rc::new(value),
        }
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

    fn new_with_map(cap: usize, by_value: HashMap<Rc<V>, Box<Entry<K, V>>, H>) -> Self {
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
        Rc<V>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.by_value.contains_key(k)
    }

    pub fn remove<Q>(&mut self, v: &Q) -> bool
    where
        Rc<V>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(entry) = self.by_value.remove(v) {
            let k = entry.key.clone();
            let v = entry.value.clone();

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
            .map(|(_, v)| {
                let p = unsafe { v.as_ref() };
                (p.key.borrow(), p.value.borrow())
            })
    }

    pub fn get<Q>(&self, k: &Q) -> Option<(&'_ K, &'_ V)>
    where
        Rc<V>: Borrow<Q>,
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
                if lowest.as_ref() > &key {
                    return;
                }
            }

            while self.len() >= self.cap {
                self.pop_lowest();
            }
        }

        self.remove(&value);

        let entry = Entry::new(key, value);
        let mut entry = Box::new(entry);
        let entry_ptr: NonNull<Entry<K, V>> = unsafe { NonNull::new_unchecked(entry.as_mut()) };
        let k = entry.key.clone();
        let v = entry.value.clone();

        if self.by_value.insert(v.clone(), entry).is_some() {
            panic!("the previous entry (if any) should have been removed already");
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

    fn lower_bound(&self) -> Option<Rc<K>> {
        if let Some((k, _)) = self.by_priority.iter().next() {
            Some(k.clone())
        } else {
            None
        }
    }

    pub fn pop_lowest(&mut self) -> Option<(Rc<K>, Rc<V>)> {
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
    fn remove() {
        let mut map = PriorityMap::<u32, String>::new(10);

        let priority = 1;
        let entry1 = "entry1".to_owned();
        let entry2 = "entry2".to_owned();

        map.put(priority, entry1.clone());
        map.put(priority, entry1.clone());

        assert!(map.remove(&entry1));

        map.put(priority, entry1);
        map.put(priority, entry2.clone());

        assert!(map.remove(&entry2));
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
