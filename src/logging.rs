use slog::*;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::sync::{Arc, Mutex};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::RawStr;
use rocket::request::{FromFormValue, FromParam, FromRequest, Outcome, State};
use rocket::{Data, Request, Response};

use app::App;
use util;

/// Wrapper for a [slog::Logger] that can be used as a `rocket` request
/// guard.
pub struct RequestLog {
	log: Logger,
}

impl RequestLog {
	pub fn wrap(log: Logger) -> RequestLog {
		RequestLog { log }
	}
}

impl std::ops::Deref for RequestLog {
	type Target = Logger;

	fn deref(&self) -> &Self::Target {
		&self.log
	}
}

impl<'a, 'r> FromRequest<'a, 'r> for RequestLog {
	type Error = ();

	fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
		let log = request.local_cache(|| -> Logger { panic!("request logger has not been registered") });
		Outcome::Success(RequestLog::wrap(log.clone()))
	}
}

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

	/// Parse a string as a [RequestId].
	pub fn parse<S: AsRef<str>>(s: S) -> util::Result<RequestId> {
		let uuid = uuid::Uuid::parse_str(s.as_ref())?;
		Ok(RequestId { uuid: uuid })
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
		write!(f, "{}", self.uuid.to_simple())
	}
}

impl<'a, 'r> FromRequest<'a, 'r> for RequestId {
	type Error = ();

	fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
		Outcome::Success(*request.local_cache(|| RequestId::nil()))
	}
}

impl<'v> FromFormValue<'v> for RequestId {
	type Error = &'v RawStr;

	fn from_form_value(form_value: &'v RawStr) -> std::result::Result<Self, Self::Error> {
		if let Ok(value) = RequestId::parse(form_value) {
			Ok(value)
		} else {
			Err(form_value)
		}
	}

	#[inline(always)]
	fn default() -> Option<Self> {
		None
	}
}

impl<'a> FromParam<'a> for RequestId {
	type Error = &'a RawStr;

	fn from_param(param: &'a RawStr) -> std::result::Result<Self, Self::Error> {
		if let Ok(value) = RequestId::parse(param) {
			Ok(value)
		} else {
			Err(param)
		}
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

		let target = format!(
			"{} {} ({})",
			request.method(),
			percent_encoding::percent_decode_str(&request.uri().to_string()).decode_utf8_lossy(),
			request_id,
		);

		let client = match request.client_ip() {
			Some(ip) => format!("{}", ip),
			None => String::from("unknown"),
		};

		request.local_cache(|| request_id);

		// Create a logger for the request
		let (logger, store) = app.request_log(o!("client" => client, "target" => target));
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

		// Store log entries by the request id:

		let app: State<&'static App> = request.guard::<State<&App>>().unwrap();

		let entries = request.local_cache(|| RequestLogStore::new());
		let entries = entries.iter().into_iter().cloned().collect::<Vec<_>>();

		let cache = app.cache();
		cache.save(request_id, entries, std::time::Duration::from_secs(10 * 60));
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

/// Mantains a list of [LogEntry] generated from a [RequestLogger].
///
/// Cloned instances share the same back-end store.
#[derive(Clone)]
pub struct RequestLogStore {
	entries: Arc<Mutex<Vec<LogEntry>>>,
}

/// Saved entry from a logger.
#[derive(Clone, Debug, Serialize)]
pub struct LogEntry {
	#[serde(serialize_with = "level_to_string")]
	pub level: Level,

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

impl LogEntry {
	pub fn from_record(record: &Record, values: &OwnedKVList) -> LogEntry {
		let mut entry = LogEntry {
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
		entry
	}
}

fn level_to_string<S: serde::Serializer>(level: &Level, s: S) -> std::result::Result<S::Ok, S::Error> {
	s.serialize_str(level.as_str())
}

impl Serializer for LogEntry {
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
	pub fn iter<'a>(&'a self) -> RequestLogStoreIter<'a, LogEntry> {
		RequestLogStoreIter {
			index: 0,
			guard: self.entries.lock().unwrap(),
		}
	}

	pub fn push(&self, record: &Record, values: &OwnedKVList) {
		let entry = LogEntry::from_record(record, values);
		self.entries.lock().unwrap().push(entry)
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

/// Implement a ring logger drain that keeps the last N entries.
#[derive(Clone)]
pub struct RingLogger {
	keep_n:  usize,
	entries: Arc<Mutex<LinkedList<LogEntry>>>,
}

impl RingLogger {
	pub fn new(keep_n: usize) -> RingLogger {
		RingLogger {
			keep_n:  keep_n,
			entries: Default::default(),
		}
	}

	/// Returns a copy of all entries current in the logger.
	pub fn entries(&self) -> Vec<LogEntry> {
		let mut out = Vec::new();
		let entries = self.entries.lock().unwrap();
		for it in entries.iter() {
			out.push(it.clone());
		}
		out
	}

	fn push(&self, record: &Record, values: &OwnedKVList) {
		let entry = LogEntry::from_record(record, values);
		let mut entries = self.entries.lock().unwrap();
		entries.push_back(entry);
		if self.keep_n > 0 {
			while entries.len() > self.keep_n {
				entries.pop_front();
			}
		}
	}
}

impl Drain for RingLogger {
	type Ok = ();
	type Err = ();

	fn log(&self, record: &Record, values: &OwnedKVList) -> std::result::Result<Self::Ok, Self::Err> {
		self.push(record, values);
		Ok(())
	}
}
