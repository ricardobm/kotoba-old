use regex::Regex;

use super::Span;
use super::SpanIter;

lazy_static! {
	static ref RE_HTML_TAG: Regex = Regex::new(
		r#"(?ix)^(
			# =========================
			# Open tag
			# =========================
			<
			(?P<ns>[a-z][-a-z0-9]*)    # Tag name

			# Attributes
			(
				\s+[_:a-z][-a-z0-9._:]*  # Attribute name

				# Attribute value
				(
					\s*=\s*
					(
						[^\s"'=<>`]+     # Unquoted value
						|
						'[^']*'          # Single quoted value
						|
						"[^"]*"          # Double quoted value
					)
				)?
			)*

			\s* /?>

			# =========================
			# Closing tag
			# =========================
			| </ (?P<ne>[a-z][-a-z0-9]*) \s* >

			# =========================
			# HTML comment
			# =========================
			| <!-- [\s\S]*? -->

			# =========================
			# Processing instruction
			# =========================
			| <\? ( [^\?] | \? [^>] )* \?>

			# =========================
			# Declaration
			# =========================
			| <! [A-Z]+ \s* [^>]+ >

			# =========================
			# CDATA
			# =========================
			| <!\[CDATA\[
				# string of characters not including ]]>
				(
					[^]]
					| \] [^]]
					| \] \]+ [^]>]
				)* \]\]>
		)"#
	)
	.unwrap();

	/// Tags that are not allowable as per the GFM spec.
	///
	/// We filter those out in two ways:
	///
	/// - First we avoid parsing this tag name in inline raw HTML, so it ends
	///   up generated as plain text or markup.
	/// - Second we forcefully escape the `<` on the raw HTML output, which
	///   might generate some weird output but prevents the tags from being
	///   generated.
	static ref RE_DISALLOWED_TAG_NAME: Regex = Regex::new(
		r#"(?ix)
			^(
				title
				| textarea
				| xmp
				| iframe
				| noembed
				| noframes
				| plaintext
				| script
				| style
			)(\b|$)
		"#
	)
	.unwrap();
}

pub fn parse<'a>(iter: &mut SpanIter<'a>) -> Option<Span<'a>> {
	let text = iter.remaining_text();
	if let Some(caps) = RE_HTML_TAG.captures(text) {
		if let Some(name) = caps.name("ns").or(caps.name("ne")) {
			if is_disallowed_tag(name.as_str()) {
				return None;
			}
		}
		let m = caps.get(0).unwrap();
		let text = &text[..m.end()];
		if text.starts_with("<!--") {
			let pre = "<!--".len();
			let pos = "-->".len();
			let s = &text[pre..text.len() - pos];
			if s.starts_with(">") || s.starts_with("->") || s.ends_with("-") || s.contains("--") {
				return None;
			}
		}
		let span = iter.span().sub_from_text(text);
		iter.skip_to(span.end);
		Some(span)
	} else {
		None
	}
}

pub fn iter_allowed_html<'a>(input: &'a str) -> impl Iterator<Item = &'a str> {
	input
		.split('<')
		.map(|s| {
			let iter = if is_disallowed_tag(s.trim_start()) {
				std::iter::once("&lt;")
			} else {
				std::iter::once("<")
			};
			iter.chain(std::iter::once(s))
		})
		.flatten()
		.skip(1) // the first '<' is bogus because of the split
}

fn is_disallowed_tag(name: &str) -> bool {
	RE_DISALLOWED_TAG_NAME.is_match(name)
}
