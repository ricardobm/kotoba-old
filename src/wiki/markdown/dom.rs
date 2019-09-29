use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

use super::block_parser;
use super::table_parser;
use super::Span;

pub use self::table_parser::TableAlign;

// ==============================
// Note on markdown text handling
// ==============================
//
// We use three types of string that deal with raw markdown text:
//
// - `&'a str`
//
//   is raw slice borrowed directly from the source. Never spans more
//   than one line and cannot contain escape sequences or any inline
//   element.
//
//   Note that this can still contain `U+0000` characters that should
//   be replaced by `U+FFFD` when generating the output.
//
// - `RawStr<'a>`
//
//   is the same as an `&'a str` but this may contain escape sequences
//   that need to be translated when generating the output.
//
// - `Span<'a>`
//
//   is a multiline block of inline text.
//
//   This can contain `U+0000` and escape sequences, the same as the
//   other types, but may also contain inline elements that need to be
//   parsed separately (this will depend on where syntactically the
//   span of text is located).
//
//   The raw string in a `Span` can also contain blockquote markers and
//   indentation that need to be stripped. As such, the `Span` provides
//   an iterator model for consuming the text while skipping ignored
//   characters.

/// Raw markdown text from the source, possibly containing escape
/// sequences but no inlines nor line breaks.
#[derive(Copy, Clone)]
pub struct RawStr<'a>(pub &'a str);

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
#[derive(Clone)]
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

	fn is_open(&self) -> bool {
		match self {
			Event::Output(markup) => markup.is_open(),
			_ => false,
		}
	}

	fn is_close(&self) -> bool {
		match self {
			Event::Output(markup) => markup.is_close(),
			_ => false,
		}
	}
}

/// Events generated when iterating markdown text that directly generate
/// some markup.
#[derive(Clone)]
pub enum MarkupEvent<'a> {
	/// Generated any text in the output.
	///
	/// Note that this is NOT encoded as HTML.
	Text(&'a str),
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

impl<'a> fmt::Display for MarkupEvent<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		super::html::fmt_html(self, f)
	}
}

/// Markdown block elements.
#[derive(Clone)]
pub enum Block<'a> {
	// ========================
	// Containers
	// ========================
	BlockQuote,
	List(ListInfo),
	ListItem(ListItemInfo),
	// ========================
	// Leaf elements
	// ========================
	Break,
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

impl<'a> fmt::Debug for Block<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Block::BlockQuote => write!(f, "BlockQuote"),
			Block::List(info) => write!(f, "List{:?}", info),
			Block::ListItem(info) => write!(f, "ListItem{:?}", info),
			Block::Break => write!(f, "Break"),
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

	loose: Cell<Option<bool>>,
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
					" task=true"
				} else {
					" task=false"
				}
			} else {
				""
			},
			if let Some(loose) = self.loose.get() {
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

impl ListItemInfo {
	pub fn from_block_info(info: block_parser::ListInfo) -> ListItemInfo {
		ListItemInfo {
			list:  ListInfo {
				ordered: info.ordered,
				marker:  info.marker,
				loose:   None,
			},
			index: 0,
			task:  info.task,
			loose: Default::default(),
		}
	}
}

impl ListItemInfo {
	/// Return `true` if the list item contains blank lines.
	pub fn loose(&self) -> bool {
		panic!();
	}

	pub fn is_list_loose(&self) -> bool {
		match self.list.loose {
			Some(true) => true,
			_ => false,
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
#[derive(Clone)]
pub struct LinkReference<'a> {
	/// Link label text. This can contain inline elements.
	pub label: Span<'a>,
	/// Link title text. This cannot contain inline elements, but may
	/// contain escape sequences.
	pub title: Span<'a>,
	/// Link destination URL.
	pub url: RawStr<'a>,
}

use std::collections::VecDeque;

type EventList<'a> = VecDeque<Event<'a>>;

pub fn compute_looseness<'a>(ls: &mut EventList<'a>) {
	assert!(ls.len() > 0 && ls[0].is_list_open());
}
