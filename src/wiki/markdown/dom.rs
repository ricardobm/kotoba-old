use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::block_parser;
use super::table_parser;
use super::{Pos, Span};

pub use self::table_parser::TableAlign;

/// Represents a plain text sequence from the Markdown source.
#[derive(Copy, Clone)]
pub struct RawStr<'a>(pub &'a str);

impl<'a> fmt::Debug for RawStr<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

#[derive(Copy, Clone)]
pub enum HeaderLevel {
	H1 = 1,
	H2 = 2,
	H3 = 3,
	H4 = 4,
	H5 = 5,
	H6 = 6,
}

/// Events generated when iterating markdown text.
#[derive(Clone, Debug)]
pub enum Event<'a> {
	/// Event that generates output.
	Output(MarkupEvent<'a>),
	/// Generated for a link reference definition.
	///
	/// This event is generated as references are found in the
	/// source, as such, to properly expand references two
	/// passes are necessary.
	Reference(LinkReference<'a>),
}

impl<'a> Event<'a> {
	pub fn is_list_open(&self) -> bool {
		match self {
			Event::Output(MarkupEvent::Open(Block::List(..))) => true,
			_ => false,
		}
	}

	pub fn is_list_close(&self) -> bool {
		match self {
			Event::Output(MarkupEvent::Close(Block::List(..))) => true,
			_ => false,
		}
	}

	pub fn is_list_item_open(&self) -> bool {
		match self {
			Event::Output(MarkupEvent::Open(Block::ListItem(..))) => true,
			_ => false,
		}
	}

	pub fn is_list_item_close(&self) -> bool {
		match self {
			Event::Output(MarkupEvent::Close(Block::ListItem(..))) => true,
			_ => false,
		}
	}

	pub fn is_open(&self) -> bool {
		match self {
			Event::Output(markup) => markup.is_open(),
			_ => false,
		}
	}

	pub fn is_close(&self) -> bool {
		match self {
			Event::Output(markup) => markup.is_close(),
			_ => false,
		}
	}
}

/// Events generated when iterating markdown text that directly generate
/// some markup.
#[derive(Clone, Debug)]
pub enum MarkupEvent<'a> {
	/// Generated for inline blocks in the output.
	Inline(Span<'a>),
	/// Generated for raw text in the output.
	Code(Span<'a>),
	/// Generated at the beginning of a block element.
	///
	/// For each `Open(block)` event there will always be a
	/// corresponding `Close(block)` event.
	Open(Block<'a>),
	/// Generated at the end of a block element.
	///
	/// Always corresponds to an [Open] event.
	Close(Block<'a>),
}

impl<'a> MarkupEvent<'a> {
	fn is_open(&self) -> bool {
		if let MarkupEvent::Open(..) = self {
			true
		} else {
			false
		}
	}

	fn is_close(&self) -> bool {
		if let MarkupEvent::Close(..) = self {
			true
		} else {
			false
		}
	}
}

pub struct MarkupWithLinks<'a, 'r>(pub &'r MarkupEvent<'a>, pub &'r LinkReferenceMap<'a>);

impl<'a, 'r> fmt::Display for MarkupWithLinks<'a, 'r> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let MarkupWithLinks(markup, refs) = self;
		super::html::output(f, markup, refs)
	}
}

/// Markdown block elements.
#[derive(Clone)]
pub enum Block<'a> {
	// ========================
	// Containers
	// ========================
	BlockQuote(Pos),
	List(ListInfo),
	ListItem(ListItemInfo),
	// ========================
	// Leaf elements
	// ========================
	Break(Pos),
	Header(HeaderLevel, Span<'a>),
	Paragraph(Span<'a>),
	HTML(Span<'a>),
	Code(Span<'a>),
	FencedCode(FencedCodeInfo<'a>),
	Table(TableInfo<'a>),
	TableHead(TableInfo<'a>),
	TableHeadCell(TableCell<'a>),
	TableBody(TableInfo<'a>),
	TableRow(TableRow<'a>),
	TableCell(TableCell<'a>),
}

impl<'a> Block<'a> {
	pub fn is_container(&self) -> bool {
		match self {
			Block::BlockQuote(..) => true,
			Block::List(..) => true,
			Block::ListItem(..) => true,
			Block::Table(..) => true,
			Block::TableHead(..) => true,
			Block::TableBody(..) => true,
			Block::TableRow(..) => true,
			_ => false,
		}
	}

	pub fn line_range(&self) -> (usize, usize) {
		match self {
			Block::BlockQuote(pos) => (pos.line, pos.line),
			Block::List(info) => (info.marker_pos.line, info.marker_pos.line),
			Block::ListItem(info) => (info.list.marker_pos.line, info.list.marker_pos.line),
			Block::Break(pos) => (pos.line, pos.line),
			Block::Header(_lvl, span) => (span.start.line, span.end.line),
			Block::Paragraph(span) => (span.start.line, span.end.line),
			Block::HTML(span) => (span.start.line, span.end.line),
			Block::Code(span) => (span.start.line, span.end.line),
			Block::FencedCode(info) => (info.code.start.line, info.code.end.line),
			Block::Table(info) => info.line_range(),
			Block::TableHead(info) => {
				let (sta, _) = info.line_range();
				let (_, end) = info.head().unwrap().line_range();
				(sta, end)
			}
			Block::TableHeadCell(cell) => cell.line_range(),
			Block::TableBody(info) => {
				let (sta, _) = if let Some(head) = info.head() {
					let (sta, end) = head.line_range();
					(sta + 1, end)
				} else {
					info.line_range()
				};
				let (_, end) = info.line_range();
				(sta, end)
			}
			Block::TableRow(row) => row.line_range(),
			Block::TableCell(cell) => cell.line_range(),
		}
	}
}

impl<'a> fmt::Debug for Block<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Block::BlockQuote(..) => write!(f, "BlockQuote"),
			Block::List(info) => write!(f, "List{:?}", info),
			Block::ListItem(info) => write!(f, "ListItem{:?}", info),
			Block::Break(..) => write!(f, "Break"),
			Block::Header(h, s) => write!(f, "Header({}, {:?})", *h as u8, s),
			Block::Paragraph(s) => write!(f, "Paragraph({:?})", s),
			Block::HTML(s) => write!(f, "HTML({:?})", s),
			Block::Code(s) => write!(f, "Code({:?})", s),
			Block::FencedCode(info) => write!(f, "FencedCode{:?}", info),
			Block::Table(info) => write!(f, "Table({:?})", info),
			Block::TableHead(info) => write!(f, "TableHead({:?})", info.head()),
			Block::TableHeadCell(cell) => write!(f, "TableHeadCell({:?})", cell),
			Block::TableBody(info) => write!(f, "TableBody({:?})", info.body()),
			Block::TableRow(row) => write!(f, "TableRow({:?})", row),
			Block::TableCell(cell) => write!(f, "TableCell({:?})", cell),
		}
	}
}

/// Data for a [Block::List].
#[derive(Clone, Default)]
pub struct ListInfo {
	/// For ordered lists this contains the start index. This is `None` for
	/// unordered lists.
	pub ordered: Option<usize>,
	/// List marker.
	///
	/// For ordered lists this is the marker after the list number (`)` or `.`).
	///
	/// For unordered lists this the item marker: either `-`, `+`, or `*`.
	pub marker: char,
	/// `true` if any item in this list is `loose`.
	///
	/// This will only be available if loose list processing is enabled.
	pub loose: Option<bool>,
	/// Marker position.
	pub marker_pos: Pos,
}

impl fmt::Debug for ListInfo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "(`")?;
		if let Some(start) = self.ordered {
			write!(f, "{}", start)?;
		}
		write!(
			f,
			"{}`{})",
			self.marker,
			if let Some(loose) = self.loose {
				if loose {
					" loose"
				} else {
					" not-loose"
				}
			} else {
				""
			},
		)
	}
}

impl ListInfo {
	pub fn from_block_info(info: block_parser::ListInfo) -> ListInfo {
		ListItemInfo::from_block_info(info).list
	}
}

/// Data for a [Block::ListItem].
#[derive(Clone)]
pub struct ListItemInfo {
	/// Information for the parent list.
	pub list: ListInfo,
	/// Zero based index for this item in the parent list.
	pub index: usize,
	/// If this item is a task item, this will contain the task state.
	pub task: Option<bool>,
	pub loose: Option<bool>,
}

impl fmt::Debug for ListItemInfo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "(#{} `", self.index)?;
		if let Some(start) = self.list.ordered {
			write!(f, "{}", start)?;
		}
		write!(
			f,
			"{}`{}{})",
			self.list.marker,
			if let Some(task) = self.task {
				if task {
					" task=1"
				} else {
					" task=0"
				}
			} else {
				""
			},
			if let Some(loose) = self.loose {
				if loose {
					" loose=1"
				} else {
					" loose=0"
				}
			} else {
				""
			},
		)
	}
}

impl ListItemInfo {
	pub fn from_block_info(info: block_parser::ListInfo) -> ListItemInfo {
		ListItemInfo {
			list:  ListInfo {
				ordered:    info.ordered,
				marker:     info.marker,
				loose:      None,
				marker_pos: info.marker_pos,
			},
			index: 0,
			task:  info.task,
			loose: Default::default(),
		}
	}
}

/// Data for a [Block::FencedCode].
#[derive(Clone)]
pub struct FencedCodeInfo<'a> {
	/// Text for the code block. This should be interpreted as raw text.
	pub code: Span<'a>,
	/// Language tag, if available.
	///
	/// This represents the first word after the opening fence.
	pub language: Option<&'a str>,
	/// Information string, if available.
	///
	/// This contains any text after the opening fence, except for the
	/// language tag.
	pub info: Option<&'a str>,
}

impl<'a> fmt::Debug for FencedCodeInfo<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "(")?;
		if let Some(language) = self.language {
			write!(f, "{} ", language)?;
		}
		if let Some(info) = self.info {
			write!(f, "{:?}", info)?;
		}
		write!(f, "{:?}", self.code)?;
		write!(f, ")")
	}
}

/// Data for a [Block::Table].
#[derive(Clone)]
pub struct TableInfo<'a> {
	inner: Rc<TableInner<'a>>,
}

impl<'a> TableInfo<'a> {
	pub fn new(
		span: Span<'a>,
		head: Option<table_parser::TableRow<'a>>,
		body: Vec<table_parser::TableRow<'a>>,
		cols: usize,
	) -> TableInfo<'a> {
		TableInfo {
			inner: Rc::new(TableInner {
				span: span,
				head: head,
				body: body,
				cols: cols,
			}),
		}
	}

	pub fn line_range(&self) -> (usize, usize) {
		let (sta, end) = (self.inner.span.start, self.inner.span.end);
		(sta.line, end.line)
	}
}

struct TableInner<'a> {
	span: Span<'a>,
	head: Option<table_parser::TableRow<'a>>,
	body: Vec<table_parser::TableRow<'a>>,
	cols: usize,
}

impl<'a> TableInfo<'a> {
	/// Number of columns in the table.
	pub fn cols(&self) -> usize {
		self.inner.cols
	}

	/// Table header row.
	pub fn head(&self) -> Option<TableRow<'a>> {
		self.inner.head.as_ref().map(|x| TableRow {
			table: self.clone(),
			cols:  self.inner.cols,
			iter:  x.iter(),
		})
	}

	/// Table body rows.
	pub fn body(&self) -> TableBody<'a> {
		TableBody { table: self.clone() }
	}
}

impl<'a> fmt::Debug for TableInfo<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<table>")?;
		if let Some(head) = self.head() {
			write!(f, "<h>{:?}</h>", head)?;
		}
		write!(f, "{:?}", self.body())?;
		write!(f, "</table>")
	}
}

/// A single cell in a [TableRow].
#[derive(Clone)]
pub struct TableCell<'a> {
	pub text:  Span<'a>,
	pub align: TableAlign,
}

impl<'a> TableCell<'a> {
	pub fn line_range(&self) -> (usize, usize) {
		(self.text.start.line, self.text.end.line)
	}
}

impl<'a> fmt::Debug for TableCell<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<c")?;
		self.align.fmt_attr(f)?;
		write!(f, ">{:?}</c>", self.text)
	}
}

/// Iterator for a row in a [TableInfo].
#[derive(Clone)]
pub struct TableRow<'a> {
	table: TableInfo<'a>,
	iter:  table_parser::RowIterator<'a>,
	cols:  usize,
}

impl<'a> TableRow<'a> {
	pub fn line_range(&self) -> (usize, usize) {
		self.iter.line_range()
	}
}

impl<'a> fmt::Debug for TableRow<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<row>")?;
		for cell in self.clone() {
			write!(f, "{:?}", cell)?;
		}
		write!(f, "</row>")
	}
}

impl<'a> Iterator for TableRow<'a> {
	type Item = TableCell<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.cols == 0 {
			None
		} else {
			self.cols -= 1;
			self.iter
				.next()
				.map(|(text, align)| TableCell { text, align })
				.or_else(|| {
					Some(TableCell {
						text:  Span::default(),
						align: TableAlign::Normal,
					})
				})
		}
	}
}

/// Rows for the body of a [TableInfo].
#[derive(Clone)]
pub struct TableBody<'a> {
	table: TableInfo<'a>,
}

impl<'a> TableBody<'a> {
	/// Number of rows in the body.
	pub fn len(&self) -> usize {
		self.table.inner.body.len()
	}

	/// Number of columns in the table.
	pub fn cols(&self) -> usize {
		self.table.inner.cols
	}

	/// Return a table row.
	pub fn row(&self, index: usize) -> TableRow<'a> {
		TableRow {
			table: self.table.clone(),
			cols:  self.table.inner.cols,
			iter:  self.table.inner.body[index].iter(),
		}
	}

	pub fn iter(&self) -> TableBodyIter<'a> {
		TableBodyIter {
			next:  0,
			table: self.table.clone(),
		}
	}
}

impl<'a> fmt::Debug for TableBody<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.iter())
	}
}

#[derive(Clone)]
pub struct TableBodyIter<'a> {
	next:  usize,
	table: TableInfo<'a>,
}

impl<'a> Iterator for TableBodyIter<'a> {
	type Item = TableRow<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.next;
		if next < self.table.inner.body.len() {
			self.next += 1;
			Some(TableRow {
				table: self.table.clone(),
				cols:  self.table.inner.cols,
				iter:  self.table.inner.body[next].iter(),
			})
		} else {
			None
		}
	}
}

impl<'a> fmt::Debug for TableBodyIter<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<b>")?;
		for row in self.clone() {
			write!(f, "{:?}", row)?;
		}
		write!(f, "</b>")
	}
}

/// Link reference.
#[derive(Clone, Debug)]
pub struct LinkReference<'a> {
	/// Link label text. This can contain inline elements.
	pub label: Span<'a>,
	/// Link title text. This cannot contain inline elements, but may
	/// contain escape sequences.
	pub title: Span<'a>,
	/// Link destination URL.
	pub url: Span<'a>,
}

#[derive(Clone)]
pub struct LinkReferenceMap<'a> {
	map: HashMap<Span<'a>, LinkReference<'a>>,
}

impl<'a> LinkReferenceMap<'a> {
	pub fn new() -> LinkReferenceMap<'a> {
		LinkReferenceMap {
			map: Default::default(),
		}
	}

	pub fn insert(&mut self, link: LinkReference<'a>) {
		// By the spec, when there are multiple matching link reference
		// definitions, the first must be used, so we don't overwrite
		let key = link.label.clone();
		self.map.entry(key).or_insert(link);
	}

	pub fn get(&self, label: &Span<'a>) -> Option<&LinkReference<'a>> {
		self.map.get(label)
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn link_reference_map_should_map_keys() {
		let key1 = span("foo");
		let key2 = span("bar");

		let val1 = LinkReference {
			label: key1.clone(),
			title: span("foo value"),
			url:   span(""),
		};
		let val2 = LinkReference {
			label: key2.clone(),
			title: span("bar value"),
			url:   span(""),
		};

		let mut map = LinkReferenceMap::new();
		map.insert(val1);
		map.insert(val2);

		assert_eq!(map.get(&key1).unwrap().title, span("foo value"));
		assert_eq!(map.get(&key2).unwrap().title, span("bar value"));
	}

	#[test]
	fn link_reference_map_should_use_first_definition() {
		let key = span("foo");

		let val1 = LinkReference {
			label: key.clone(),
			title: span("foo value"),
			url:   span(""),
		};
		let val2 = LinkReference {
			label: key.clone(),
			title: span("bar value"),
			url:   span(""),
		};

		let mut map = LinkReferenceMap::new();
		map.insert(val1);
		map.insert(val2);

		assert_eq!(map.get(&key).unwrap().title, span("foo value"));
	}

	#[test]
	fn link_reference_map_should_case_fold_and_normalize() {
		// spell-checker: disable
		let key1a = span("foo");
		let key1b = span("FOO");
		let key2a = span("    Maße  \t   αγω    ");
		let key2b = span(" maSSe\tΑΓΩ\t");
		// spell-checker: enable

		let val1a = LinkReference {
			label: key1a.clone(),
			title: span("foo value"),
			url:   span(""),
		};
		let val2a = LinkReference {
			label: key2a.clone(),
			title: span("bar value"),
			url:   span(""),
		};

		let val1b = LinkReference {
			label: key1b.clone(),
			title: span("NOT foo value"),
			url:   span(""),
		};
		let val2b = LinkReference {
			label: key2b.clone(),
			title: span("NOT bar value"),
			url:   span(""),
		};

		let mut map = LinkReferenceMap::new();
		map.insert(val1a);
		map.insert(val2a);

		assert_eq!(map.get(&key1a).unwrap().title, span("foo value"));
		assert_eq!(map.get(&key2a).unwrap().title, span("bar value"));

		assert_eq!(map.get(&key1b).unwrap().title, span("foo value"));
		assert_eq!(map.get(&key2b).unwrap().title, span("bar value"));

		map.insert(val1b);
		map.insert(val2b);

		assert_eq!(map.get(&key1a).unwrap().title, span("foo value"));
		assert_eq!(map.get(&key2a).unwrap().title, span("bar value"));

		assert_eq!(map.get(&key1b).unwrap().title, span("foo value"));
		assert_eq!(map.get(&key2b).unwrap().title, span("bar value"));
	}

	fn span<'s>(s: &'s str) -> Span<'s> {
		let sta = Pos::default();
		let end = {
			let mut p = sta;
			p.skip(s);
			p
		};
		Span {
			buffer: s,
			start:  sta,
			end:    end,
			indent: 0,
			quotes: 0,
			loose:  None,
		}
	}

}
