use regex::Regex;

use super::SpanIter;

/// A `< >` delimited autolink.
#[derive(Clone, Debug)]
pub struct AutoLink<'a> {
	/// The matched link address, as it appears on the source text.
	///
	/// This should also be used as the link label. It may or may not
	/// contain the scheme.
	///
	/// NOTE: not HTML safe.
	pub link: &'a str,
	/// Scheme prefix, excluding the `:`.
	///
	/// If the source does not contain the scheme, this will be a static
	/// string with the detected schema.
	pub scheme: &'a str,
	/// This will contain the necessary schema prefix in case the [link]
	/// does not contain it, being empty otherwise.
	///
	/// The prefix includes the `:` and possibly the `//`. It should not
	/// be used as part of the label.
	pub prefix: &'a str,
}

/// Parses an autolink at the current position.
///
/// Skip the link and returns the inner text.
pub fn parse<'a>(iter: &mut SpanIter<'a>) -> Option<AutoLink<'a>> {
	lazy_static! {
		static ref RE_AUTOLINK: Regex = Regex::new(
			r#"(?xi)
				^<
					(?P<link>
						(?P<scheme>
							[a-z][-+.a-z0-9]{1,31}
						) :
						[^<>[:space:][:cntrl:]]*
					)
				>
			"#
		)
		.unwrap();
		static ref RE_EMAIL: Regex = Regex::new(
			r#"(?xi)
				^<(?P<link>
					[a-zA-Z0-9.!\#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?
					(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*
				)>
			"#
		)
		.unwrap();
	}

	if let Some(m) = RE_AUTOLINK.captures(iter.chunk()) {
		let link = m.name("link").unwrap().as_str();
		let scheme = m.name("scheme").unwrap().as_str();
		iter.skip_bytes(m.get(0).unwrap().as_str().len());
		Some(AutoLink {
			link,
			scheme,
			prefix: "",
		})
	} else if let Some(m) = RE_EMAIL.captures(iter.chunk()) {
		let link = m.name("link").unwrap().as_str();
		iter.skip_bytes(m.get(0).unwrap().as_str().len());
		Some(AutoLink {
			link,
			scheme: "mailto",
			prefix: "mailto:",
		})
	} else {
		None
	}
}
