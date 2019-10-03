use regex::Regex;

use super::InlineEvent;
use super::{Range, SpanIter};

/// Parses an autolink at the current position.
///
/// Skip the link and returns the inner text.
pub fn parse_autolink<'a>(iter: &mut SpanIter<'a>) -> Option<InlineEvent<'a>> {
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
		iter.skip_len(m.get(0).unwrap().as_str().len());
		Some(InlineEvent::AutoLink {
			link,
			scheme,
			prefix: "",
			delimited: true,
		})
	} else if let Some(m) = RE_EMAIL.captures(iter.chunk()) {
		let link = m.name("link").unwrap().as_str();
		iter.skip_len(m.get(0).unwrap().as_str().len());
		Some(InlineEvent::AutoLink {
			link,
			scheme: "mailto",
			prefix: "mailto:",
			delimited: true,
		})
	} else {
		None
	}
}

/// Find and parses an GFM autolink extension.
///
/// If successful, returns the range for the match and
/// the [InlineEvent::AutoLink].
pub fn parse_autolink_extension<'a>(chunk: &'a str) -> Option<(Range, InlineEvent<'a>)> {
	lazy_static! {
		// If an autolink ends in a semicolon (;), we check to see if it appears
		// to resemble an entity reference; if the preceding text is & followed
		// by one or more alphanumeric characters. If so, it is excluded from
		// the autolink.
		static ref RE_TRAILING_ENTITY: Regex = Regex::new(r#"(?xi)(&[a-z0-9]+;)+$"#).unwrap();

		// Trailing punctuation (?, !, ., ,, :, *, _, and ~) will not be
		// considered part of the autolink, though they may be included in
		// the interior of the link.
		static ref RE_TRAILING_PUNCTUATION: Regex = Regex::new(r#"[.?!,:*_~]+$"#).unwrap();

		static ref RE_AUTOLINK_GFM: Regex = Regex::new(
			r#"(?xi)
				# valid boundaries
				( ^ | \s | [*_~\(] )
				(?P<link>
					(
						# www autolink
						www\.

						# extended URL autolink
						|
						(?P<scheme> https? ) ://
					)
					(
						# Valid domain
						# ============

						# A valid domain consists of segments of alphanumeric
						# characters, # underscores (_) and hyphens (-) separated
						# by periods (.).

						# There must be at least one period, and no underscores
						# may be present in the last two segments of the domain.

						([-_a-z0-9]+\.)*

						# last two segments
						[-a-z0-9]+ (\.[-a-z0-9]+)
					)

					(?P<path>
						# after a valid domain, zero or more non-space non-`<`
						# characters may follow
						[^\s<]*
					)

					# Email autolink:
					# - One ore more characters which are alphanumeric,
					#   or `.`, `-`, `_`, or `+`.
					# - An `@` symbol.
					# - One or more characters which are alphanumeric,
					#   or `-` or `_`, separated by periods (.). There
					#   must be at least one period. The last character
					#   must not be one of `-` or `_`.
					| (?P<email>
						[-.+_a-z0-9]+ @
						( [-_a-z0-9]+ \. )+
						[-_a-z0-9]+
					)
				)
			"#
		)
		.unwrap();
	}
	for caps in RE_AUTOLINK_GFM.captures_iter(chunk) {
		let link = caps.name("link").unwrap();
		let path = caps.name("path").map(|x| x.as_str()).unwrap_or("");

		let email = caps.name("email").map(|x| x.as_str()).unwrap_or("").len() > 0;
		let start = link.start();
		let end = link.end();

		let link = link.as_str();

		let mut trim = 0;
		while trim < path.len() {
			let start_trim = trim;
			let link = &link[..link.len() - trim];
			let path = &path[..path.len() - trim];

			// When an autolink ends in ), we scan the entire autolink for the
			// total number of parentheses. If there is a greater number of
			// closing parentheses than opening ones, we don’t consider the
			// unmatched trailing parentheses part of the autolink
			if path.ends_with(')') {
				let mut ps = 0;
				let mut pe = 0;
				for c in link.chars() {
					match c {
						'(' => ps += 1,
						')' => pe += 1,
						_ => (),
					}
				}
				if ps < pe {
					trim += 1;
				}
			}

			if let Some(m) = RE_TRAILING_PUNCTUATION.find(path) {
				trim += m.as_str().len();
			} else if let Some(m) = RE_TRAILING_ENTITY.find(path) {
				trim += m.as_str().len();
			}

			if trim == start_trim {
				break;
			}
		}

		let link = &link[..link.len() - trim];

		if email {
			// the last character must not be one of `-` or `_`
			if link.ends_with(|c| c == '-' || c == '_') {
				continue;
			}
		}

		let range = Range { start, end: end - trim };

		let (scheme, prefix) = if let Some(scheme) = caps.name("scheme") {
			let scheme = scheme.as_str();
			(scheme, "")
		} else if email {
			("mailto", "mailto:")
		} else {
			("http", "http://")
		};

		let event = InlineEvent::AutoLink {
			link:      link,
			scheme:    scheme,
			prefix:    prefix,
			delimited: false,
		};
		return Some((range, event));
	}

	None
}

/*
   TODO: nested link handling

   The link syntax in markdown allows nested brackets in the link label, but
   in case those nested brackets are links themselves (e.g. the shortcut link
   reference syntax) they take precedence.

   The above means that we need to know all the link references in the
   document to parse links.
*/
