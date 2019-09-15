//! Basic utilities used throughout the code

/// Simple wrapper for [std::time::Instant] that adds some logging facilities.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PerfTimer {
	t0: std::time::Instant,
}

impl PerfTimer {
	#[inline]
	pub fn now() -> PerfTimer {
		PerfTimer{ t0: std::time::Instant::now() }
	}

	#[inline]
	pub fn elapsed(&self) -> std::time::Duration {
		self.t0.elapsed()
	}
}

impl std::fmt::Display for PerfTimer {
	#[inline]
	fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result {
		write!(f, "{:.3?}", self.elapsed())
	}
}

impl slog::KV for PerfTimer {
	fn serialize(&self, _record: &slog::Record, serializer: &mut dyn slog::Serializer) -> slog::Result {
		serializer.emit_arguments("Δ", &format_args!("{}", self))
	}
}

/// Simple macro to instantiate a [PerfTimer].
macro_rules! time {
	($id:ident) => {
		let $id = $crate::base::PerfTimer::now();
	};
}
