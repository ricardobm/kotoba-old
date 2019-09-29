use std::collections::VecDeque;
use std::fmt::Write;

mod block_parser;
use self::block_parser::{BlockEvent, BlockIterator, Container, Leaf};

mod common;

mod span;
pub use self::span::{Range, Span, SpanIter};

mod link_ref;
use self::link_ref::parse_link_ref;

mod table_parser;

mod dom;
use self::dom::*;

mod html;

use util;

/// Parse the input string as markdown, returning an iterator of [Element].
pub fn parse_markdown<'a>(input: &'a str) -> MarkdownIterator<'a> {
	MarkdownIterator::new(input, true)
}

/// Generate HTML code from the iterator returned by [parse_markdown].
pub fn to_html<'a>(iter: MarkdownIterator<'a>) -> util::Result<String> {
	let mut output = String::new();
	let mut first = true;
	for it in iter {
		match it {
			Event::Output(markup) => {
				if !first {
					if let MarkupEvent::Open(..) = markup {
						write!(output, "\n")?;
					}
				}
				write!(output, "{}", markup)?;
				first = false;
			}
			Event::Reference(_) => {
				// TODO: implement references
			}
		}
	}
	Ok(output)
}

/// Iterates over [Element]s in a markdown text.
pub struct MarkdownIterator<'a> {
	blocks:      BlockIterator<'a>,
	loose_lists: bool,
	parents:     VecDeque<ParentContainer>,
	state:       IteratorState<'a>,
}

enum ParentContainer {
	BlockQuote,
	List(block_parser::ListInfo),
	ListItem(block_parser::ListInfo),
}

enum IteratorState<'a> {
	Start,
	HandleStart(Container),
	HandleEnd(Container),
	HandleLeaf(Leaf<'a>),
	LeafText(Block<'a>, SpanIter<'a>, SpanMode),
	LeafEnd(Block<'a>),
	GenerateOpen(Block<'a>),
	CloseListAndOpenNew(block_parser::ListInfo),
	OpenNewList(block_parser::ListInfo),
	OpenListItem(block_parser::ListInfo),
	Table(TableInfo<'a>, TagMode),
	TableHead(TableInfo<'a>, TableRow<'a>, TagMode),
	TableBody(TableInfo<'a>, Option<TableBodyIter<'a>>, TagMode),
	TableRow(
		TableInfo<'a>,
		TableSection,
		Option<TableBodyIter<'a>>,
		TableRow<'a>,
		TagMode,
	),
	TableCell(
		TableInfo<'a>,
		TableSection,
		Option<TableBodyIter<'a>>,
		TableRow<'a>,
		TableCell<'a>,
		TagMode,
		Option<SpanIter<'a>>,
	),
	BeforeEnd,
	End,
}

enum TableSection {
	Head,
	Body,
}

enum TagMode {
	Start,
	Content,
	End,
}

impl<'a> Default for IteratorState<'a> {
	fn default() -> Self {
		IteratorState::Start
	}
}

impl<'a> MarkdownIterator<'a> {
	fn new(input: &'a str, loose_lists: bool) -> MarkdownIterator<'a> {
		MarkdownIterator {
			blocks:      BlockIterator::new(input),
			loose_lists: loose_lists, // TODO: support this
			parents:     Default::default(),
			state:       IteratorState::Start,
		}
	}

	fn get_next(&mut self) -> Option<Event<'a>> {
		let (next_state, result) = loop {
			self.state = match std::mem::take(&mut self.state) {
				IteratorState::End => {
					break (IteratorState::End, None);
				}

				IteratorState::Start => {
					// Consume next block event and dispatch to one of the
					// handlers
					if let Some(next) = self.blocks.next() {
						match next {
							BlockEvent::Start(container) => IteratorState::HandleStart(container),
							BlockEvent::End(container) => IteratorState::HandleEnd(container),
							BlockEvent::Leaf(leaf) => IteratorState::HandleLeaf(leaf),
						}
					} else {
						IteratorState::BeforeEnd
					}
				}

				IteratorState::BeforeEnd => match self.parents.pop_back() {
					Some(container) => {
						break (IteratorState::BeforeEnd, Self::close_container_event(container));
					}
					None => IteratorState::End,
				},

				// Start handling of a `BlockEvent::Start(container)` event
				IteratorState::HandleStart(block) => {
					match block {
						Container::BlockQuote => {
							self.parents.push_back(ParentContainer::BlockQuote);
							IteratorState::GenerateOpen(Block::BlockQuote)
						}

						Container::ListItem(item_info) => {
							// To generate a list item we need check if the
							// current open container is a root list element:
							//
							// - If it isn't then we'll need to open a new list.
							// - If it is, we need to check if we can append to
							//   it, or if we'll have to close and generate a
							//   new list.
							//
							let (is_new_list, has_list) = match self.parents.iter().last() {
								None => (true, false),
								Some(ParentContainer::BlockQuote) => (true, false),
								Some(ParentContainer::ListItem(_)) => (true, false),
								Some(ParentContainer::List(info)) => (!info.is_next_same_list(&item_info), true),
							};

							if is_new_list {
								if has_list {
									IteratorState::CloseListAndOpenNew(item_info)
								} else {
									IteratorState::OpenNewList(item_info)
								}
							} else {
								IteratorState::OpenListItem(item_info)
							}
						}
					}
				}

				// Generate a non-list-item block "open" event...
				IteratorState::GenerateOpen(block) => {
					// ...first we need to check if the last container open
					// is a root list element and close it.
					if let Some(ParentContainer::List(_)) = self.parents.iter().last() {
						// assert that we are not being used with a ListItem
						if let Block::ListItem(_) = block {
							unreachable!();
						}
						break self.close_list(IteratorState::GenerateOpen(block));
					}

					// ...if there is no open list we are good to go
					let event = Event::Output(MarkupEvent::Open(block));
					break (IteratorState::Start, Some(event));
				}

				// Handles closing a currently opened list and opening a new one
				IteratorState::CloseListAndOpenNew(item_info) => {
					break self.close_list(IteratorState::OpenNewList(item_info));
				}

				// Handles opening a new list and generating its first item
				IteratorState::OpenNewList(item_info) => {
					self.parents.push_back(ParentContainer::List(item_info.clone()));
					let list = Block::List(ListInfo::from_block_info(item_info.clone()));
					let event = Event::Output(MarkupEvent::Open(list));
					break (IteratorState::OpenListItem(item_info), Some(event));
				}

				// Handles generating a list item
				IteratorState::OpenListItem(item_info) => {
					self.parents.push_back(ParentContainer::ListItem(item_info.clone()));
					let item = Block::ListItem(ListItemInfo::from_block_info(item_info));
					let event = Event::Output(MarkupEvent::Open(item));
					break (IteratorState::Start, Some(event));
				}

				// Start handling of a `BlockEvent::End(container)` event
				IteratorState::HandleEnd(block) => {
					// It is possible that we have a root list element opened,
					// since those have no explicit closing. We must close
					// those before continuing.
					if let Some(ParentContainer::List(_)) = self.parents.iter().last() {
						break self.close_list(IteratorState::HandleEnd(block));
					}

					let event = match block {
						Container::BlockQuote => match self.parents.pop_back() {
							Some(ParentContainer::BlockQuote) => {
								Self::close_container_event(ParentContainer::BlockQuote)
							}
							_ => unreachable!(),
						},
						Container::ListItem(_) => match self.parents.pop_back() {
							Some(ParentContainer::ListItem(info)) => {
								Self::close_container_event(ParentContainer::ListItem(info))
							}
							_ => unreachable!(),
						},
					};

					break (IteratorState::Start, event);
				}

				// Start handling of a `BlockEvent::Leaf(leaf)` event
				IteratorState::HandleLeaf(leaf) => match Self::parse_leaf(leaf) {
					LeafOrReference::Leaf(block, text, mode) => {
						let event = Event::Output(MarkupEvent::Open(block.clone()));
						break (IteratorState::LeafText(block, text, mode), Some(event));
					}
					LeafOrReference::Reference(link_ref) => {
						let event = Event::Reference(link_ref);
						break (IteratorState::Start, Some(event));
					}
					LeafOrReference::Table(table) => IteratorState::Table(table, TagMode::Start),
				},

				// Generates the text of a leaf block.
				IteratorState::LeafText(block, mut text, mode) => {
					if let Some(s) = text.next() {
						// TODO: parse inlines
						let event = Event::Output(MarkupEvent::Text(s));
						break (IteratorState::LeafText(block, text, mode), Some(event));
					} else {
						IteratorState::LeafEnd(block)
					}
				}

				// Handles closing a leaf block.
				IteratorState::LeafEnd(block) => {
					let event = Event::Output(MarkupEvent::Close(block));
					break (IteratorState::Start, Some(event));
				}

				//=============================
				// Table states
				//=============================
				IteratorState::Table(table, tag) => match tag {
					TagMode::Start => {
						let block = Block::Table(table.clone());
						let event = Event::Output(MarkupEvent::Open(block));
						break (IteratorState::Table(table, TagMode::Content), Some(event));
					}
					TagMode::Content => {
						if let Some(head) = table.head() {
							IteratorState::TableHead(table, head, TagMode::Start)
						} else {
							IteratorState::TableBody(table, None, TagMode::Start)
						}
					}
					TagMode::End => {
						let block = Block::Table(table.clone());
						let event = Event::Output(MarkupEvent::Close(block));
						break (IteratorState::Start, Some(event));
					}
				},

				IteratorState::TableHead(table, head, tag) => match tag {
					TagMode::Start => {
						let block = Block::TableHead(table.clone());
						let event = Event::Output(MarkupEvent::Open(block));
						break (IteratorState::TableBody(table, None, TagMode::Start), Some(event));
					}
					TagMode::Content => IteratorState::TableRow(table, TableSection::Head, None, head, TagMode::Start),
					TagMode::End => {
						let block = Block::TableHead(table.clone());
						let event = Event::Output(MarkupEvent::Close(block));
						break (IteratorState::TableBody(table, None, TagMode::Start), Some(event));
					}
				},

				IteratorState::TableBody(table, body, tag) => match tag {
					TagMode::Start => {
						let block = Block::TableBody(table.clone());
						let event = Event::Output(MarkupEvent::Open(block));
						break (IteratorState::TableBody(table, None, TagMode::Start), Some(event));
					}
					TagMode::Content => {
						if let Some(mut body) = body {
							if let Some(row) = body.next() {
								IteratorState::TableRow(table, TableSection::Body, Some(body), row, TagMode::Start)
							} else {
								IteratorState::TableBody(table, None, TagMode::End)
							}
						} else {
							let iter = table.body().iter();
							IteratorState::TableBody(table, Some(iter), tag)
						}
					}
					TagMode::End => {
						let block = Block::TableBody(table.clone());
						let event = Event::Output(MarkupEvent::Close(block));
						break (IteratorState::Table(table, TagMode::End), Some(event));
					}
				},

				IteratorState::TableRow(table, section, body, mut row, tag) => match tag {
					TagMode::Start => {
						let block = Block::TableRow(row.clone());
						let event = Event::Output(MarkupEvent::Open(block));
						break (
							IteratorState::TableRow(table, section, body, row, TagMode::Content),
							Some(event),
						);
					}
					TagMode::Content => {
						if let Some(cell) = row.next() {
							IteratorState::TableCell(table, section, body, row, cell, TagMode::Start, None)
						} else {
							IteratorState::TableRow(table, section, body, row, TagMode::End)
						}
					}
					TagMode::End => {
						let next = match section {
							TableSection::Head => IteratorState::TableHead(table, row.clone(), TagMode::End),
							TableSection::Body => IteratorState::TableBody(table, body, TagMode::Content),
						};
						let block = Block::TableRow(row);
						let event = Event::Output(MarkupEvent::Close(block));
						break (next, Some(event));
					}
				},

				IteratorState::TableCell(table, section, body, row, cell, tag, text) => {
					match tag {
						TagMode::Start => {
							let block = match section {
								TableSection::Head => Block::TableHeadCell(cell.clone()),
								TableSection::Body => Block::TableCell(cell.clone()),
							};
							let event = Event::Output(MarkupEvent::Open(block));
							break (
								IteratorState::TableRow(table, section, body, row, TagMode::Content),
								Some(event),
							);
						}
						TagMode::Content => {
							if let Some(mut text) = text {
								if let Some(s) = text.next() {
									// TODO: parse inlines
									let event = Event::Output(MarkupEvent::Text(s));
									let state =
										IteratorState::TableCell(table, section, body, row, cell, tag, Some(text));
									break (state, Some(event));
								} else {
									IteratorState::TableCell(table, section, body, row, cell, TagMode::End, None)
								}
							} else {
								let iter = Some(cell.text.iter());
								IteratorState::TableCell(table, section, body, row, cell, tag, iter)
							}
						}
						TagMode::End => {
							let block = match section {
								TableSection::Head => Block::TableHeadCell(cell.clone()),
								TableSection::Body => Block::TableCell(cell.clone()),
							};
							let event = Event::Output(MarkupEvent::Close(block));
							break (
								IteratorState::TableRow(table, section, body, row, TagMode::Content),
								Some(event),
							);
						}
					}
				}
			}
		};

		self.state = next_state;
		result
	}

	fn parse_leaf(leaf: Leaf<'a>) -> LeafOrReference {
		match leaf {
			Leaf::Paragraph { text } => {
				let iter = text.iter();
				LeafOrReference::Leaf(Block::Paragraph(text), iter, SpanMode::Text)
			}
			Leaf::HTML { code, .. } => {
				let iter = code.iter();
				LeafOrReference::Leaf(Block::HTML(code), iter, SpanMode::Code)
			}
			Leaf::LinkReference { url, label, title } => LeafOrReference::Reference(LinkReference {
				label: label,
				title: title,
				url:   RawStr(url),
			}),
			Leaf::IndentedCode { code } => {
				let iter = code.iter();
				LeafOrReference::Leaf(Block::Code(code), iter, SpanMode::Code)
			}
			Leaf::FencedCode { code, lang, info, .. } => {
				let iter = code.iter();
				let info = FencedCodeInfo {
					code:     code,
					info:     info,
					language: lang,
				};
				LeafOrReference::Leaf(Block::FencedCode(info), iter, SpanMode::Code)
			}
			Leaf::Break => {
				let iter = Span::empty().iter();
				LeafOrReference::Leaf(Block::Break, iter, SpanMode::Code)
			}
			Leaf::Header { level, text } => {
				let iter = text.iter();
				let level = match level {
					1 => HeaderLevel::H1,
					2 => HeaderLevel::H2,
					3 => HeaderLevel::H3,
					4 => HeaderLevel::H4,
					5 => HeaderLevel::H5,
					6 => HeaderLevel::H6,
					_ => unreachable!(),
				};
				LeafOrReference::Leaf(Block::Header(level, text), iter, SpanMode::Text)
			}
			Leaf::Table { span, head, body, cols } => {
				let info = TableInfo::new(span, head, body, cols.unwrap());
				LeafOrReference::Table(info)
			}
		}
	}

	fn close_list(&mut self, next_state: IteratorState<'a>) -> (IteratorState<'a>, Option<Event<'a>>) {
		match self.parents.pop_back() {
			Some(ParentContainer::List(list_info)) => {
				let list = Block::List(ListInfo::from_block_info(list_info));
				let event = Event::Output(MarkupEvent::Close(list));
				(next_state, Some(event))
			}
			_ => unreachable!(),
		}
	}

	fn close_container_event(container: ParentContainer) -> Option<Event<'a>> {
		let block = match container {
			ParentContainer::BlockQuote => Block::BlockQuote,
			ParentContainer::ListItem(info) => Block::ListItem(ListItemInfo::from_block_info(info)),
			ParentContainer::List(info) => Block::List(ListInfo::from_block_info(info)),
		};
		Some(Event::Output(MarkupEvent::Close(block)))
	}
}

enum SpanMode {
	Text,
	Code,
}

enum LeafOrReference<'a> {
	Leaf(Block<'a>, SpanIter<'a>, SpanMode),
	Reference(LinkReference<'a>),
	Table(TableInfo<'a>),
}

impl<'a> Iterator for MarkdownIterator<'a> {
	type Item = Event<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.get_next()
	}
}

//
// TESTS
//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_markdown_simple() {
		// Simple paragraphs
		test(
			r#"
			Paragraph 1

			Paragraph 2

			3.1
			3.2
			"#,
			r#"
				<p>Paragraph 1</p>
				<p>Paragraph 2</p>
				<p>3.1
				3.2</p>
			"#,
		);
	}

	#[test]
	fn test_markdown_breaks() {
		// Thematic breaks
		test(
			r#"
			***

			---
			___

			P1

			   ---
			   ***
			   ___
			P2

			+++
			--
			**
			__

			 ***
			  * * *
			   *  *  *
			"#,
			r#"
				<hr/>
				<hr/>
				<hr/>
				<p>P1</p>
				<hr/>
				<hr/>
				<hr/>
				<p>P2</p>
				<p>+++
				--
				**
				__</p>
				<hr/>
				<hr/>
				<hr/>
			"#,
		);
	}

	#[test]
	fn test_markdown_atx_headings() {
		test(
			r#"
			# H 1
			## H 2
			### H 3
			#### H 4
			##### H 5
			###### H 6

			P1
			 # H1 # ##############
			  ## H2##
			   ### H3 # # #
			P2
			####### H7
			"#,
			r#"
				<h1>H 1</h1>
				<h2>H 2</h2>
				<h3>H 3</h3>
				<h4>H 4</h4>
				<h5>H 5</h5>
				<h6>H 6</h6>
				<p>P1</p>
				<h1>H1 #</h1>
				<h2>H2##</h2>
				<h3>H3 # #</h3>
				<p>P2
				####### H7</p>
			"#,
		)
	}

	#[test]
	fn test_markdown_setext_headings() {
		test(
			r#"
			Title 1
			=======

			Title 2
			-------

			Multi-line
			   Title 2
			   ---

			L1
			L2
			==
			===
			L3
			--
			---

			"#,
			r#"
				<h1>Title 1</h1>
				<h2>Title 2</h2>
				<h2>Multi-line
				   Title 2</h2>
				<h1>L1
				L2
				==</h1>
				<h2>L3
				--</h2>
			"#,
		);
	}

	fn test(input: &str, expected: &str) {
		let input = common::text(input);
		let expected = common::text(expected);

		let result = to_html(parse_markdown(input.as_str())).unwrap();
		assert_eq!(result, expected);
	}
}
