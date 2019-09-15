use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub trait CacheKey: Send + Sync + Clone + Eq + Hash + std::fmt::Display {}
pub trait CacheVal {}

impl<T: Send + Sync + Clone + Eq + Hash + std::fmt::Display> CacheKey for T {}
impl<T> CacheVal for T {}

/// In memory cache structure with support for TTL and interior mutability.
#[allow(dead_code)]
pub struct Cache<K: CacheKey, V: CacheVal> {
	store: Arc<Mutex<CacheStore<K, V>>>,
}

impl<K: CacheKey, V: CacheVal> Clone for Cache<K, V> {
	fn clone(&self) -> Self {
		Cache {
			store: self.store.clone(),
		}
	}
}

struct CacheStore<K: CacheKey, V: CacheVal> {
	ttl: BinaryHeap<CacheKeyEntry<K>>,
	map: HashMap<K, Arc<V>>,
}

#[allow(dead_code)]
impl<K: CacheKey, V: CacheVal> Cache<K, V> {
	pub fn new() -> Cache<K, V> {
		Default::default()
	}

	/// Save an entry to the cache. Calls [purge] before inserting.
	pub fn save(&self, key: K, val: V, ttl: Duration) {
		let now = Instant::now();
		let ttl = now + ttl;

		let mut store = self.store.lock().unwrap();
		store = Self::do_purge(store);

		// Insert new entry
		store.ttl.push(CacheKeyEntry {
			expire: ttl,
			key:    key.clone(),
		});
		store.map.insert(key, Arc::new(val));
	}

	pub fn get(&self, key: &K) -> Option<Arc<V>> {
		let store = self.store.lock().unwrap();
		if let Some(val) = store.map.get(key) {
			Some(val.clone())
		} else {
			None
		}
	}

	/// Purge all expired entries from the cache.
	#[allow(dead_code)]
	pub fn purge(&self) {
		let _ = Self::do_purge(self.store.lock().unwrap());
	}

	fn do_purge(mut store: MutexGuard<'_, CacheStore<K, V>>) -> MutexGuard<'_, CacheStore<K, V>> {
		let now = Instant::now();

		// Remove all expired entries from the cache.
		while let Some(entry) = store.ttl.peek() {
			let expired = entry.expire <= now;
			if expired {
				let entry = store.ttl.pop().unwrap();
				store.map.remove(&entry.key);
			} else {
				break;
			}
		}

		store
	}
}

impl<K: CacheKey, V: CacheVal> Default for Cache<K, V> {
	fn default() -> Cache<K, V> {
		Cache {
			store: Arc::new(Mutex::new(CacheStore {
				ttl: Default::default(),
				map: Default::default(),
			})),
		}
	}
}

#[derive(PartialEq, Eq)]
struct CacheKeyEntry<K: CacheKey> {
	expire: Instant,
	key:    K,
}

impl<K: CacheKey> PartialOrd for CacheKeyEntry<K> {
	fn partial_cmp(&self, other: &CacheKeyEntry<K>) -> Option<std::cmp::Ordering> {
		Some(self.cmp(&other))
	}
}

impl<K: CacheKey> Ord for CacheKeyEntry<K> {
	fn cmp(&self, other: &CacheKeyEntry<K>) -> std::cmp::Ordering {
		other.expire.cmp(&other.expire)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::thread::{sleep, spawn};
	use std::time::Duration;

	#[test]
	fn test_cache() {
		let cache = Cache::new();

		// Insert entries
		cache.save("a", Mutex::new(vec![1]), Duration::from_millis(40));
		cache.save("b", Mutex::new(vec![2]), Duration::from_millis(40));

		// Let them expire
		sleep(Duration::from_millis(50));

		// Make sure get works and entries do not expire until cache is
		// modified
		assert_eq!(*cache.get(&"b").unwrap().lock().unwrap(), vec![2]);
		assert_eq!(*cache.get(&"a").unwrap().lock().unwrap(), vec![1]);

		// Make sure inserted entries don't expire.
		cache.save("c", Mutex::new(vec![3]), Duration::from_millis(0));

		// Make sure entries do expire.
		assert!(cache.get(&"a").is_none());
		assert!(cache.get(&"b").is_none());

		// Cache should be safe across threads
		let c = cache.clone();
		let h = spawn(move || {
			let entry = c.get(&"c").unwrap();
			let mut entry = entry.lock().unwrap();
			entry.push(2);
			entry.push(1);
		});

		h.join().unwrap();

		// Cache modifications should be visible.
		assert_eq!(*cache.get(&"c").unwrap().lock().unwrap(), vec![3, 2, 1]);

		// Purge should work.
		cache.purge();
		assert!(cache.get(&"c").is_none());
	}
}
