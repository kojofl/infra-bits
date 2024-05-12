use super::RTreeMap;

pub struct RSet<K: PartialOrd> {
    inner: RTreeMap<K, ()>,
}

impl<K: PartialOrd> RSet<K> {
    pub fn new() -> Self {
        Self {
            inner: RTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: K) {
        self.inner.insert(key, ());
    }

    pub fn remove(&mut self, key: K) {
        self.inner.remove(key);
    }
}
