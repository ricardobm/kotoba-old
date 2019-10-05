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
			[a-z][-a-z0-9]*              # Tag name

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
			| </ [a-z][-a-z0-9]* \s* >

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
}

pub fn parse<'a>(iter: &mut SpanIter<'a>) -> Option<Span<'a>> {
	let text = iter.remaining_text();
	if let Some(m) = RE_HTML_TAG.find(text) {
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
