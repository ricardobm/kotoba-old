use slog::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mantains a list of [RequestLogEntry] generated from a [RequestLogger].
#[derive(Clone)]
pub struct RequestLogStore {
	entries: Arc<Mutex<Vec<RequestLogEntry>>>,
}

/// Log entry generated from a [RequestLogger].
#[derive(Debug)]
pub struct RequestLogEntry {
	pub level:  Level,
	pub msg:    String,
	pub line:   u32,
	pub column: u32,
	pub file:   &'static str,
	pub module: &'static str,
	pub keys:   HashMap<&'static str, String>,
	pub values: HashMap<&'static str, String>,
}

impl Serializer for RequestLogEntry {
	fn emit_arguments(&mut self, key: Key, val: &std::fmt::Arguments) -> Result {
		self.keys.insert(key, format!("{}", val));
		Ok(())
	}
}

impl RequestLogStore {
	pub fn new() -> RequestLogStore {
		RequestLogStore {
			entries: Arc::new(Mutex::new(Vec::new())),
		}
	}

	/// Returns an iterator for the items on the store.
	///
	/// Example
	/// =======
	///
	/// ```
	/// let store = RequestLogStore::new();
	/// for _it in &store.iter() {
	///     // ...
	/// }
	///
	/// let mut iter = store.iter();
	/// while Some(it) = iter.next() {
	///     // ...
	/// }
	/// ```
	#[allow(dead_code)]
	pub fn iter<'a>(&'a self) -> RequestLogStoreIter<'a, RequestLogEntry> {
		RequestLogStoreIter {
			index: 0,
			guard: self.entries.lock().unwrap(),
		}
	}

	pub fn push(&self, record: &Record, values: &OwnedKVList) {
		let mut entry = RequestLogEntry {
			level:  record.level(),
			msg:    format!("{}", record.msg()),
			line:   record.line(),
			column: record.column(),
			file:   record.file(),
			module: record.module(),
			keys:   HashMap::new(),
			values: HashMap::new(),
		};

		values.serialize(record, &mut entry).ok();
		entry.values = entry.keys;
		entry.keys = HashMap::new();

		record.kv().serialize(record, &mut entry).ok();
		self.entries.lock().unwrap().push(entry)
	}
}

/// Implements a [slog::Drain] that stores log entries in a [RequestLogStore]
/// before forwarding them to another drain.
#[derive(Clone)]
pub struct RequestLogger<D: Drain> {
	store: RequestLogStore,
	drain: D,
}

impl<D: Drain> RequestLogger<D> {
	/// Creates a new [RequestLogger] that forwards to the given drain.
	pub fn new(drain: D) -> RequestLogger<D> {
		RequestLogger {
			store: RequestLogStore::new(),
			drain: drain,
		}
	}

	/// Returns a [RequestLogStore] with entries associated to this logger.
	pub fn store(&self) -> RequestLogStore {
		self.store.clone()
	}
}

impl<D: Drain> Drain for RequestLogger<D> {
	type Ok = D::Ok;
	type Err = D::Err;

	fn log(&self, record: &Record, values: &OwnedKVList) -> std::result::Result<Self::Ok, Self::Err> {
		self.store.push(record, values);
		self.drain.log(record, values)
	}
}

pub struct RequestLogStoreIter<'a, T: 'a> {
	index: usize,
	guard: std::sync::MutexGuard<'a, Vec<T>>,
}

impl<'a, T: 'a> RequestLogStoreIter<'a, T> {
	pub fn next(&mut self) -> Option<&T> {
		if self.index < self.guard.len() {
			let p = self.index;
			self.index += 1;
			Some(&self.guard[p])
		} else {
			None
		}
	}
}

impl<'a, 'b: 'a, T: 'a> IntoIterator for &'b RequestLogStoreIter<'a, T> {
	type Item = &'a T;

	type IntoIter = std::slice::Iter<'b, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.guard.iter()
	}
}
