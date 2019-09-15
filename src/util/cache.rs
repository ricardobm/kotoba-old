use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use std::any::{Any, TypeId};
use std::cell::UnsafeCell;

pub trait CacheKey: Send + Sync + Clone + Eq + Hash + std::fmt::Display {}
pub trait CacheVal {}

impl<T: Send + Sync + Clone + Eq + Hash + std::fmt::Display> CacheKey for T {}
impl<T> CacheVal for T {}

/// Provides unique [Cache<K, V>] instances for each pair of `(K, V)` types.
pub struct CacheMap {
	inner: Arc<Mutex<CacheMapInner>>,
}

struct CacheMapInner {
	init: bool,
	data: UnsafeCell<*mut HashMap<TypeId, *mut dyn Any>>,
}

unsafe impl Send for CacheMapInner {}

impl Default for CacheMap {
	fn default() -> CacheMap {
		CacheMap {
			inner: Arc::new(Mutex::new(CacheMapInner {
				init: false,
				data: UnsafeCell::new(0 as *mut _),
			})),
		}
	}
}

impl CacheMap {
	pub fn new() -> CacheMap {
		Default::default()
	}

	/// Returns a global cache instance for a given key and value types.
	///
	/// The cache value is returned by value, but its backing store is shared
	/// with any instance returned by this method for the same `K` and `V`
	/// types.
	pub fn get<K: CacheKey + 'static, V: CacheVal + 'static>(&self) -> Cache<K, V> {
		let mut inner = self.inner.lock().unwrap();
		if !inner.init {
			let map = Box::new(Default::default());
			unsafe {
				*inner.data.get() = Box::into_raw(map);
				inner.init = true;
			}
		}

		let type_id = TypeId::of::<(K, V)>();
		let item = unsafe { (**inner.data.get()).get(&type_id) };

		let entry_ptr = if let Some(entry) = item {
			*entry
		} else {
			let entry: Box<Cache<K, V>> = Box::new(Cache::default());
			unsafe {
				let entry = entry as Box<dyn Any>;
				let entry = Box::into_raw(entry);
				(**inner.data.get()).insert(type_id, entry);
				entry
			}
		};

		unsafe {
			let cache = entry_ptr as *const Cache<K, V>;
			(*cache).clone()
		}
	}
}

impl Drop for CacheMapInner {
	fn drop(&mut self) {
		if !self.init {
			return;
		}

		unsafe {
			let map = &mut **self.data.get();
			for value in map.values_mut() {
				let mut value = Box::from_raw(*value);
				drop(&mut value);
			}

			let mut map = Box::from_raw(map);
			drop(&mut map);
		}
	}
}

/// In memory cache structure with support for TTL and interior mutability.
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

	#[test]
	fn test_cache_map() {
		let cache_map = CacheMap::new();
		let duration = Duration::from_secs(99999);

		// Test basic retrieval
		let c1 = cache_map.get::<&'static str, u16>();
		let c2 = cache_map.get::<&'static str, u32>();
		let c3 = cache_map.get::<u32, &'static str>();

		c1.save("101", 101_u16, duration);
		c1.save("102", 102_u16, duration);

		c2.save("203", 203_u32, duration);
		c2.save("204", 204_u32, duration);

		c3.save(305_u32, "305", duration);
		c3.save(306_u32, "306", duration);

		// Test retrieving an existing instance
		let c1 = cache_map.get::<&'static str, u16>();
		let c2 = cache_map.get::<&'static str, u32>();
		let c3 = cache_map.get::<u32, &'static str>();

		assert_eq!(*c1.get(&"101").unwrap(), 101_u16);
		assert_eq!(*c1.get(&"102").unwrap(), 102_u16);
		assert_eq!(*c2.get(&"203").unwrap(), 203_u32);
		assert_eq!(*c2.get(&"204").unwrap(), 204_u32);
		assert_eq!(*c3.get(&305_u32).unwrap(), "305");
		assert_eq!(*c3.get(&306_u32).unwrap(), "306");

		// Make sure instances share the backing store
		let c3_b = cache_map.get::<u32, &'static str>();
		c3.save(307_u32, "307", duration);
		assert_eq!(*c3_b.get(&307_u32).unwrap(), "307");
	}

	#[test]
	fn test_cache_map_drops() {
		struct DropCheck<T: Fn()> {
			drop_fn: Box<T>,
		}

		impl<T: Fn()> Drop for DropCheck<T> {
			fn drop(&mut self) {
				(self.drop_fn)();
			}
		}

		use std::cell::RefCell;

		let dropped = RefCell::new(false);

		{
			let cache_map = CacheMap::new();
			let cache = cache_map.get::<usize, DropCheck<_>>();
			let dropped = dropped.clone();
			let drop_fn = move || {
				dropped.replace(true);
			};

			cache.save(
				0,
				DropCheck {
					drop_fn: Box::new(drop_fn),
				},
				Duration::from_secs(9999),
			);
		}

		assert!(*dropped.borrow(), true);
	}
}
