use std::collections::HashMap;
use std::collections::VecDeque;

pub struct FIFOCache<K: Sized, V> {
    inner: HashMap<K, V>,
    order: VecDeque<K>,
    pub maxsize: usize,
}

impl<K, V> FIFOCache<K, V> {
    #[must_use]
    pub fn new(maxsize: usize, capacity: usize) -> Self {
        if capacity > 0 {
            let cap = if maxsize > 0 && capacity <= maxsize {
                capacity
            } else {
                maxsize
            };

            return FIFOCache {
                inner: HashMap::with_capacity(cap),
                order: VecDeque::with_capacity(cap),
                maxsize,
            };
        }

        FIFOCache {
            inner: HashMap::new(),
            order: VecDeque::new(),
            maxsize,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn order_capacity(&self) -> usize {
        self.order.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K: std::hash::Hash + Eq + Clone, V> FIFOCache<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> pyo3::PyResult<()> {
        if self.maxsize > 0 && self.inner.len() >= self.maxsize && self.inner.get(&key).is_none() {
            self.popitem();
        }

        let length = self.inner.len();
        let time_to_shrink = ((length + 1) == self.maxsize) && length == self.inner.capacity();

        match self.inner.insert(key.clone(), value) {
            Some(_) => (),
            None => self.order.push_back(key),
        }

        if time_to_shrink {
            self.inner.shrink_to_fit();
        }

        Ok(())
    }

    pub fn update<T: IntoIterator<Item = pyo3::PyResult<(K, V)>>>(
        &mut self,
        iterable: T,
    ) -> pyo3::PyResult<()> {
        for result in iterable {
            let (key, val) = result?;
            self.insert(key, val)?;
        }

        Ok(())
    }
}

impl<K: std::hash::Hash + Eq, V> FIFOCache<K, V> {
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
        self.order.shrink_to_fit();
    }

    pub fn popitem(&mut self) -> Option<V> {
        match self.order.pop_front() {
            Some(x) => self.inner.remove(&x),
            None => None,
        }
    }

    pub fn first(&self) -> Option<&K> {
        self.order.front()
    }

    pub fn last(&self) -> Option<&K> {
        self.order.back()
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.inner.remove(key) {
            Some(val) => {
                let index = self.order.iter().position(|x| x == key).unwrap();
                self.order.remove(index);
                Some(val)
            }
            None => None,
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn clear(&mut self, reuse: bool) {
        self.inner.clear();
        self.order.clear();

        if !reuse {
            self.inner.shrink_to_fit();
            self.order.shrink_to_fit();
        }
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, K, V> {
        self.inner.keys()
    }

    pub fn sorted_keys(&self) -> std::collections::vec_deque::Iter<'_, K> {
        self.order.iter()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, K, V> {
        self.inner.values()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.inner.iter()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> FIFOCache<K, V> {
    pub fn setdefault(&mut self, key: K, default: V) -> pyo3::PyResult<V> {
        let exists = self.inner.get(&key);
        if exists.is_some() {
            return Ok(exists.cloned().unwrap());
        }

        if self.maxsize > 0 && self.inner.len() >= self.maxsize {
            self.popitem();
        }

        let length = self.inner.len();
        let time_to_shrink = ((length + 1) == self.maxsize) && length == self.inner.capacity();

        self.inner.insert(key.clone(), default.clone());
        self.order.push_back(key);

        if time_to_shrink {
            self.inner.shrink_to_fit();
        }

        Ok(default)
    }
}

impl<K: PartialEq, V> PartialEq for FIFOCache<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.maxsize == other.maxsize && self.order == other.order
    }
}
impl<K: PartialEq, V> Eq for FIFOCache<K, V> {}