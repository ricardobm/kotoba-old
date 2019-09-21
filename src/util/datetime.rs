use chrono::{Utc, Local};

#[derive(Clone, Serialize, Deserialize)]
pub struct DateTime(chrono::DateTime<Utc>);

#[derive(Clone)]
pub struct LocalDateTime(chrono::DateTime<Local>);

impl DateTime {
	pub fn now() -> DateTime {
		DateTime(Utc::now())
	}

	pub fn to_local(&self) -> LocalDateTime {
		LocalDateTime(self.0.with_timezone(&Local))
	}

	pub fn format(&self, fmt: &str) -> String {
		format!("{}", self.0.format(fmt))
	}
}

impl LocalDateTime {
	pub fn now() -> LocalDateTime {
		LocalDateTime(Local::now())
	}

	pub fn to_utc(&self) -> DateTime {
		DateTime(self.0.with_timezone(&Utc))
	}
}

impl std::fmt::Display for DateTime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::fmt::Display for LocalDateTime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::fmt::Debug for DateTime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

impl std::fmt::Debug for LocalDateTime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
