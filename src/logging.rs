use slog::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::request::{FromRequest, Outcome, State};
use rocket::{Data, Request, Response};

use app::App;

/// UUID identifier for a request.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RequestId {
	uuid: uuid::Uuid,
}

impl RequestId {
	/// Returns a new random [RequestId].
	pub fn new() -> RequestId {
		use rand::Rng;
		use uuid::{Builder, Variant, Version};
		let rand = rand::thread_rng().gen();
		let uuid = Builder::from_bytes(rand)
			.set_variant(Variant::RFC4122)
			.set_version(Version::Random)
			.build();
		RequestId { uuid: uuid }
	}

	/// Returns a zero-ed RequestId.
	pub fn nil() -> RequestId {
		RequestId {
			uuid: uuid::Uuid::nil(),
		}
	}
}

impl std::fmt::Display for RequestId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.uuid.fmt(f)
	}
}

impl<'a, 'r> FromRequest<'a, 'r> for RequestId {
	type Error = ();

	/// Derives an instance of `Self` from the incoming request metadata.
	///
	/// If the derivation is successful, an outcome of `Success` is returned. If
	/// the derivation fails in an unrecoverable fashion, `Failure` is returned.
	/// `Forward` is returned to indicate that the request should be forwarded
	/// to other matching routes, if any.
	fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
		Outcome::Success(*request.local_cache(|| RequestId::nil()))
	}
}

/// Fairing implementation for `rocket` that sets up per-request logging.
///
/// See also:
///
/// https://api.rocket.rs/v0.4/rocket/fairing/trait.Fairing.html
/// https://api.rocket.rs/v0.4/rocket/request/trait.FromRequest.html
#[derive(Copy, Clone)]
pub struct ServerLogger {}

impl Fairing for ServerLogger {
	fn info(&self) -> Info {
		Info {
			name: "Request Logger",
			kind: Kind::Request | Kind::Response,
		}
	}

	fn on_request(&self, request: &mut Request, _data: &Data) {
		let request_id = RequestId::new();
		let app: State<&'static App> = request.guard::<State<&App>>().unwrap();
		let info = format!(
			"{} {} (from: {}) -- {}",
			request.method(),
			request.uri(),
			match request.client_ip() {
				Some(ip) => format!("{}", ip),
				None => String::from("unknown"),
			},
			request_id,
		);

		request.local_cache(|| request_id);

		// Create a logger for the request
		let (logger, store) = app.request_log(o!("request" => info));
		request.local_cache(|| store);
		request.local_cache(|| logger);

		time!(t_request);
		request.local_cache(|| t_request);
	}

	fn on_response(&self, request: &Request, response: &mut Response) {
		time!(t_none);

		// Send a request ID as a header
		let request_id = request.guard::<RequestId>().unwrap();
		response.set_raw_header("X-Request-Id", format!("{}", request_id));

		// Send the response time header
		let t_request = *request.local_cache(|| t_none);
		if t_request != t_none {
			response.set_raw_header("X-Response-Time", format!("{}", t_request));
		}
	}
}

/// Mantains a list of [RequestLogEntry] generated from a [RequestLogger].
///
/// Cloned instances share the same back-end store.
#[derive(Clone)]
pub struct RequestLogStore {
	entries: Arc<Mutex<Vec<RequestLogEntry>>>,
}

/// Entry in a [RequestLogStore].
#[derive(Debug)]
pub struct RequestLogEntry {
	pub level:  Level,
	pub msg:    String,
	pub line:   u32,
	pub column: u32,
	pub file:   &'static str,
	pub module: &'static str,

	/// Keys from the record.
	pub keys: HashMap<&'static str, String>,

	/// Keys from the logger.
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
///
/// Cloned entries share the same store.
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
