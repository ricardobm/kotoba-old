use regex::Regex;

use super::{Range, Span, SpanIter};

dbg_flag!(true);

const REPLACEMENT_CHAR: char = '\u{FFFD}';

/// Text parsing modes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextMode {
	/// Parse raw text, without escapes nor entities.
	Raw,
	/// Same as [Raw], but also replaces line breaks with spaces.
	InlineCode,
	/// Parse text with backslash escapes.
	WithEscapes,
	/// Parse text with backslash escapes and HTML entities.
	WithEscapesAndEntities,
	/// Parse text with backslash escapes, HTML entities and autolinks.
	WithLinks,
}

/// A textual node in the Markdown document.
#[derive(Clone, Debug)]
pub struct TextNode<'a> {
	span: Span<'a>,
	mode: TextMode,
}

impl<'a> TextNode<'a> {
	pub fn new(span: Span<'a>, mode: TextMode) -> TextNode {
		TextNode { span, mode }
	}

	pub fn iter(&self) -> TextNodeIterator<'a> {
		TextNodeIterator {
			iter: self.span.iter(),
			mode: self.mode,
			link: None,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum TextOrChar<'a> {
	Text(&'a str),
	Char(char),
}

/// Span of text in a [TextNode]
#[derive(Clone, Debug)]
pub enum TextSpan<'a> {
	/// Text content to output.
	///
	/// This is guaranteed to not contain any basic HTML entities, so it can be
	/// output to HTML without escaping.
	Text(&'a str),
	/// Single char version of [Text].
	Char(char),
	/// A hard line break (e.g. `<br/>`).
	LineBreak,
	/// HTML entity.
	///
	/// This is generated for any basic HTML entities in the Markdown text
	/// (e.g. `<`, `>`, `&`, `\`, `"`, `'`) and also for explicit entities
	/// in the source.
	Entity {
		/// The source text.
		///
		/// This can be either an entity or a character that needs escaping.
		source: &'a str,
		/// The HTML entity to generate.
		entity: &'a str,
		/// The actual Unicode text corresponding to the entity.
		output: TextOrChar<'a>,
	},
	/// GFM-like autolink.
	Link {
		/// The matched link address, as it appears on the source text.
		///
		/// This should also be used as the link label. It may or may not
		/// contain the scheme.
		///
		/// NOTE: not HTML safe.
		link: &'a str,
		/// Scheme prefix, excluding the `:`.
		///
		/// If the source does not contain the scheme, this will be a static
		/// string with the detected schema.
		scheme: &'a str,
		/// This will contain the necessary schema prefix in case the [link]
		/// does not contain it, being empty otherwise.
		///
		/// The prefix includes the `:` and possibly the `//`. It should not
		/// be used as part of the label.
		prefix: &'a str,
	},
}

pub struct TextNodeIterator<'a> {
	iter: SpanIter<'a>,
	mode: TextMode,
	link: Option<(Range, Option<TextSpan<'a>>)>,
}

impl<'a> Iterator for TextNodeIterator<'a> {
	type Item = TextSpan<'a>;

	fn next(&mut self) -> Option<TextSpan<'a>> {
		lazy_static! {
			static ref RE_VALID_ESCAPE: Regex = Regex::new(
				r#"(?x)
					^\\[\-\\\{\}\[\]\(\)\^\|\~\&\$\#/:<>"!%'*+,.;=?@_`]
				"#
			)
			.unwrap();
			static ref RE_HARD_BREAK: Regex = Regex::new(
				r#"(?x)
					^\\[\s&&[^\n\r]]*(\n|\r\n?)
				"#
			)
			.unwrap();
		}

		let (has_escapes, has_entities, has_links) = match self.mode {
			TextMode::Raw => (false, false, false),
			TextMode::InlineCode => (false, false, false),
			TextMode::WithEscapes => (true, false, false),
			TextMode::WithEscapesAndEntities => (true, true, false),
			TextMode::WithLinks => (true, true, true),
		};

		let is_inline_code = match self.mode {
			TextMode::InlineCode => true,
			_ => false,
		};

		// let iter = &mut self.iter;
		let chunk = self.iter.chunk();
		match self.iter.next_char() {
			Some('\\') if has_escapes => {
				if RE_VALID_ESCAPE.is_match(chunk) {
					self.iter.skip_bytes(1);
					next_char_escaped(&mut self.iter)
				} else if let Some(m) = RE_HARD_BREAK.find(chunk) {
					self.iter.skip_bytes(m.end());
					Some(TextSpan::LineBreak)
				} else {
					self.iter.skip_char();
					Some(TextSpan::Char('\\'))
				}
			}
			Some('\r') if is_inline_code => {
				self.iter.skip_char();
				if let Some('\n') = self.iter.next_char() {
					self.iter.skip_char();
				}
				Some(TextSpan::Text(" "))
			}
			Some('\n') if is_inline_code => {
				self.iter.skip_char();
				Some(TextSpan::Text(" "))
			}
			Some('&') if has_entities => parse_entity(&mut self.iter),
			Some(c) if needs_escaping(c) => next_char_escaped(&mut self.iter),
			Some(_) => {
				let link_value = if has_links { self.next_link() } else { None };
				let link_index = link_value.as_ref().map(|x| Some(x.0.start)).unwrap_or(None);
				if let Some(0) = link_index {
					let (range, elem) = link_value.unwrap();
					self.iter.skip_bytes(range.end);
					Some(elem)
				} else {
					let limit = chunk
						.find(|c| is_special_char(c, is_inline_code))
						.unwrap_or(chunk.len());
					let limit = if let Some(index) = link_index {
						std::cmp::min(limit, index)
					} else {
						limit
					};
					let chunk = &chunk[..limit];
					if limit == 0 {
						next_char_escaped(&mut self.iter)
					} else {
						let chunk = &chunk[..limit];
						self.iter.skip_bytes(limit);
						Some(TextSpan::Text(chunk))
					}
				}
			}
			None => None,
		}
	}
}

impl<'a> TextNodeIterator<'a> {
	/// Finds and parses the next GFM autolink in the current chunk.
	///
	/// If found, returns the link range and [TextSpan::Link].
	fn next_link(&mut self) -> Option<(Range, TextSpan<'a>)> {
		// check if we have a cached value and if its offset is still ahead of
		// the iterator position
		let offset = self.iter.pos().offset;
		if let Some((range, link)) = self.link.clone() {
			if range.start >= offset {
				if let Some(link) = link {
					let range = Range {
						start: range.start - offset,
						end:   range.end - offset,
					};
					return Some((range, link));
				} else {
					return None;
				}
			}
		}

		// find the next autolink in the current chunk.
		let chunk = self.iter.chunk();
		let previous = self.iter.previous_char();
		if let Some((range, link)) = parse_autolink_extension(chunk, previous) {
			println!("found link in {:?} at {:?} -- {:?}", chunk, range, link);
			let abs_range = Range {
				start: range.start + offset,
				end:   range.end + offset,
			};
			self.link = Some((abs_range, Some(link.clone())));
			Some((range, link))
		} else {
			let chunk_end = offset + chunk.len();
			let abs_range = Range {
				start: chunk_end,
				end:   chunk_end,
			};
			self.link = Some((abs_range, None));
			None
		}
	}
}

fn parse_entity<'a>(iter: &mut SpanIter<'a>) -> Option<TextSpan<'a>> {
	use super::entities::get_named_entity;

	lazy_static! {
		static ref RE_ENTITY: Regex = Regex::new(r#"^&\w+;"#).unwrap();
		static ref RE_ENTITY_DEC: Regex = Regex::new(r#"^&\#(?P<v>[0-9]{1,7});"#).unwrap();
		static ref RE_ENTITY_HEX: Regex = Regex::new(r#"^&\#[xX](?P<v>[0-9A-Fa-f]{1,6});"#).unwrap();
	}

	let chunk = iter.chunk();
	if let Some(m) = RE_ENTITY.find(chunk) {
		let len = m.end();
		let entity = m.as_str();
		if let Some(output) = get_named_entity(entity) {
			let txt = TextSpan::Entity {
				source: entity,
				entity: entity,
				output: TextOrChar::Text(output),
			};
			iter.skip_bytes(len);
			return Some(txt);
		}
	} else if let Some(caps) = RE_ENTITY_DEC.captures(chunk) {
		let src = caps.get(0).unwrap();
		let len = src.end();
		let src = src.as_str();
		let dec = caps.name("v").unwrap().as_str().parse::<u32>().unwrap();
		let chr = std::char::from_u32(dec).unwrap_or(REPLACEMENT_CHAR);
		let txt = entity_or_char(src, chr);
		iter.skip_bytes(len);
		return Some(txt);
	} else if let Some(caps) = RE_ENTITY_HEX.captures(chunk) {
		let src = caps.get(0).unwrap();
		let len = src.end();
		let src = src.as_str();
		let hex = u32::from_str_radix(caps.name("v").unwrap().as_str(), 16).unwrap();
		let chr = std::char::from_u32(hex).unwrap_or(REPLACEMENT_CHAR);
		let txt = entity_or_char(src, chr);
		iter.skip_bytes(len);
		return Some(txt);
	}

	next_char_escaped(iter)
}

fn next_char_escaped<'a>(iter: &mut SpanIter<'a>) -> Option<TextSpan<'a>> {
	let chunk = iter.chunk();
	let mut chars = chunk.char_indices();
	let (_, next) = chars.next().unwrap();
	let (size, _) = chars.next().unwrap_or((chunk.len(), ' '));

	let source = &chunk[..size];
	iter.skip_bytes(size);
	if let Some(entity) = super::html_entity(next) {
		Some(TextSpan::Entity {
			source: source,
			entity: entity,
			output: TextOrChar::Text(source),
		})
	} else {
		Some(TextSpan::Text(source))
	}
}

fn entity_or_char<'a>(source: &'a str, c: char) -> TextSpan<'a> {
	if let Some(entity) = super::html_entity(c) {
		TextSpan::Entity {
			source: source,
			entity: entity,
			output: TextOrChar::Char(c),
		}
	} else {
		TextSpan::Char(c)
	}
}

fn needs_escaping(chr: char) -> bool {
	match chr {
		'\0' | '&' | '<' | '>' | '\'' | '"' => true,
		_ => false,
	}
}

/// Check if a character needs special handling as an inline.
#[inline]
fn is_special_char(chr: char, include_eol: bool) -> bool {
	match chr {
		// escapes
		'\\' => true,
		// should be replaced by `U+FFFD`
		'\0' => true,
		// HTML entities
		'&' | '<' | '>' | '\'' | '"' => true,
		// code spans
		'`' => true,
		// emphasis and strikethrough
		'*' | '_' | '~' => true,
		// links
		'[' | '!' => true,
		// line breaks
		'\r' | '\n' => include_eol,
		_ => false,
	}
}

fn parse_autolink_extension<'a>(chunk: &'a str, previous: Option<char>) -> Option<(Range, TextSpan<'a>)> {
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
		if start == 0 {
			match previous {
				None | Some('*') | Some('_') | Some('~') | Some('(') => { },
				Some(c) if c.is_whitespace() => { },
				_ => {
					// autolinks are only acceptable after spaces or one of the
					// characters above
					continue;
				}
			}
		}
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

		let link = TextSpan::Link {
			link:   link,
			scheme: scheme,
			prefix: prefix,
		};
		return Some((range, link));
	}

	None
}
