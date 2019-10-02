use regex::Regex;

use super::common;
use super::inline::InlineEvent;
use super::{Range, Span, SpanIter};

/// Parses an autolink at the current position.
///
/// Skip the link and returns the inner text.
pub fn parse_autolink<'a>(iter: &mut SpanIter<'a>) -> Option<InlineEvent<'a>> {
	lazy_static! {
		static ref RE_AUTOLINK: Regex = Regex::new(
			r#"(?xi)
				^<
					(?P<uri>
						(?P<scheme>[a-z][-+.a-z0-9]{1,31})
						:
						(?P<address>[^<>[:space:][:cntrl:]]*)
					)
				>
			"#
		)
		.unwrap();
		static ref RE_EMAIL: Regex = Regex::new(
			r#"(?xi)
				^<(?P<uri>
					[a-zA-Z0-9.!\#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?
					(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*
				)>
			"#
		)
		.unwrap();
	}

	if let Some(m) = RE_AUTOLINK.captures(iter.chunk()) {
		let uri = m.name("uri").unwrap().as_str();
		let scheme = m.name("scheme").unwrap().as_str();
		let address = m.name("address").unwrap().as_str();
		iter.skip_len(m.get(0).unwrap().as_str().len());
		Some(InlineEvent::AutoLink {
			uri,
			scheme,
			address,
			is_email: false,
			delimited: true,
		})
	} else if let Some(m) = RE_EMAIL.captures(iter.chunk()) {
		let uri = m.name("uri").unwrap().as_str();
		iter.skip_len(m.get(0).unwrap().as_str().len());
		Some(InlineEvent::AutoLink {
			uri,
			scheme: "mailto",
			address: uri,
			is_email: true,
			delimited: true,
		})
	} else {
		None
	}
}
