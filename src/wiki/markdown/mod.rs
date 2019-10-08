use std::collections::VecDeque;
use std::fmt::Write;

mod block_parser;
use self::block_parser::{BlockEvent, BlockIterator, Container, Leaf};

mod common;

mod span;
pub use self::span::{Span, SpanIter};

mod text;
pub use self::text::{range_from, range_from_pos, Pos, PosRange, Range, TextBuffer};

mod link_ref;
use self::link_ref::parse_link_ref;

mod table_parser;

mod dom;
use self::dom::*;

mod html;
mod inline;

use util;

dbg_flag!(false);

/// Parse the input string as markdown, returning an iterator of [Element].
pub fn parse_markdown<'a>(input: &'a str) -> MarkdownIterator<'a> {
	MarkdownIterator::new(input, true)
}

/// Generate HTML code from the iterator returned by [parse_markdown].
pub fn to_html<'a>(iter: MarkdownIterator<'a>) -> util::Result<String> {
	// Collect all link references from the iterator:
	let mut references = LinkReferenceMap::new();
	let mut document = Vec::new();
	for it in iter {
		match it {
			Event::Output(markup) => {
				document.push(markup);
			}
			Event::Reference(reference) => {
				references.insert(reference);
			}
		}
	}

	// Output markup:
	let mut output = String::new();
	let mut first = true;
	let mut last_was_paragraph = false;
	for markup in document {
		if !first {
			let break_line = match &markup {
				MarkupEvent::Open(Block::Paragraph(span)) => !(span.loose == Some(false)),
				MarkupEvent::Close(Block::Paragraph(_)) => false,
				MarkupEvent::Close(Block::ListItem(info)) => !last_was_paragraph || info.list.loose == Some(true),
				MarkupEvent::Open(..) => true,
				MarkupEvent::Close(block) => block.is_container(),
				_ => false,
			};
			if break_line {
				write!(output, "\n")?;
			}
		}
		write!(output, "{}", MarkupWithLinks(&markup, &references))?;
		first = false;
		last_was_paragraph = if let MarkupEvent::Close(Block::Paragraph(_)) = markup {
			true
		} else {
			false
		};
	}
	Ok(output)
}

/// Iterates over [Element]s in a markdown text.
pub struct MarkdownIterator<'a> {
	blocks:      BlockIterator<'a>,
	loose_lists: bool,
	parents:     VecDeque<ParentContainer>,
	state:       IteratorState<'a>,

	//=============================
	// Parsing of loose lists
	//=============================

	// Next events to generate.
	queue: VecDeque<Event<'a>>,

	// Number of pending opened lists.
	pending: VecDeque<LooseItem>,
}

#[derive(Debug)]
enum LooseItem {
	List { loose: bool, index: usize, end: usize },
	ListItem { index: usize },
	Child { level: usize, line: usize, loose: bool },
}

#[derive(Debug)]
enum ParentContainer {
	BlockQuote(Pos),
	List(block_parser::ListInfo),
	ListItem(block_parser::ListInfo),
}

#[derive(Debug)]
enum IteratorState<'a> {
	Start,
	HandleStart(Container),
	HandleEnd(Container),
	HandleLeafOrBreak(Leaf<'a>),
	HandleLeaf(Leaf<'a>),
	LeafInline(Block<'a>, Span<'a>, SpanMode),
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
	),
	BeforeEnd,
	End,
}

#[derive(Debug)]
enum TableSection {
	Head,
	Body,
}

#[derive(Debug)]
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
			loose_lists: loose_lists,
			parents:     Default::default(),
			state:       IteratorState::Start,

			queue:   Default::default(),
			pending: Default::default(),
		}
	}

	fn get_next(&mut self) -> Option<Event<'a>> {
		if !self.loose_lists {
			return self.read_next();
		}

		// For loose list processing we need to wait until a list has been
		// completely defined to determined if it is loose or not

		loop {
			if self.pending.len() == 0 {
				// Process events normally until we find a list open.
				let result = if let Some(event) = self.queue.pop_front() {
					// drain the queue of pending events from a previous list
					Some(event)
				} else if let Some(event) = self.read_next() {
					if let Event::Output(MarkupEvent::Open(Block::List(ref info))) = event {
						// from this point on we start queueing events until
						// the end of the list
						self.pending.push_back(LooseItem::List {
							loose: false,
							index: self.queue.len(),
							end:   info.marker_pos.line,
						});
						self.queue.push_back(event);
						continue;
					}
					Some(event)
				} else {
					None
				};
				break result;
			} else {
				// Parse the next event in the input looking for the
				// "loose list" criteria:

				fn on_open(pending: &mut VecDeque<LooseItem>, sta: usize, end: usize) {
					// For each list item's child block we need to check if
					// there is a blank line between the start of the block
					// and the end of the previous one.

					// Get the range. The end range is not reliable because
					// we haven't reach the end of the block yet, but we'll
					// use it for now and update later at the `Close` event.
					match pending.back_mut() {
						Some(LooseItem::Child {
							ref mut level,
							ref mut line,
							ref mut loose,
						}) => {
							// We already encountered children for this
							// list item.
							//
							// The level will be zero if we just closed a
							// previous child block. Non-zero means we are
							// inside the hierarchy (the loose concept
							// applies only to direct children).
							if *level == 0 {
								if sta > *line + 1 {
									// we consume this when closing the
									// parent list item
									*loose = true;
								}
								// we probably could not even bother to
								// save the end here
								*line = end;
							}
							*level += 1;
						}
						_ => {
							// This is the first child of the list item.
							pending.push_back(LooseItem::Child {
								level: 1,
								line:  end,
								loose: false,
							});
						}
					}
				}

				fn on_close(pending: &mut VecDeque<LooseItem>, end: usize) {
					// When closing a block, we just update the child's item
					// last line number.
					match pending.back_mut() {
						Some(LooseItem::Child {
							ref mut level,
							ref mut line,
							..
						}) => {
							*level -= 1;
							*line = end;
						}
						_ => unreachable!("close event with no pending Child item"),
					}
				}

				if let Some(mut event) = self.read_next() {
					if let Event::Output(MarkupEvent::Open(Block::List(info))) = &event {
						on_open(&mut self.pending, info.marker_pos.line, info.marker_pos.line);
						self.pending.push_back(LooseItem::List {
							loose: false,
							index: self.queue.len(),
							end:   info.marker_pos.line,
						});
					} else if let Event::Output(MarkupEvent::Open(Block::ListItem(..))) = &event {
						// Found a list item
						self.pending.push_back(LooseItem::ListItem {
							index: self.queue.len(),
						});
					} else if let Event::Output(MarkupEvent::Open(block)) = &event {
						let (sta, end) = block.line_range();
						on_open(&mut self.pending, sta, end);
					} else if let Event::Output(MarkupEvent::Close(Block::List(ref mut close_info))) = &mut event {
						// close the list and update its loose information
						let end = match self.pending.pop_back() {
							// remove pending List
							Some(LooseItem::List { loose, index, end }) => match self.queue[index] {
								Event::Output(MarkupEvent::Open(ref mut block)) => match block {
									Block::List(ref mut info) => {
										// update the loose information on the opening event
										info.loose = Some(loose);
										// also update the close event info
										close_info.loose = Some(loose);
										end
									}
									_ => unreachable!("block in queue event for pending List is not a List"),
								},
								_ => unreachable!("queue event in pending List is not an open"),
							},
							_ => unreachable!("pending List and close event did not match"),
						};
						if self.pending.len() > 0 {
							on_close(&mut self.pending, end);
						}
					} else if let Event::Output(MarkupEvent::Close(Block::ListItem(ref mut close_info))) = &mut event {
						// take the loose information stored on the child items
						let loose = if let Some(LooseItem::Child { loose, .. }) = self.pending.back() {
							let loose = *loose;
							self.pending.pop_back();
							loose
						} else {
							false // is it possible for a list item to have no children?
						};
						close_info.loose = Some(loose);

						// remove the ListItem event and update the item information
						match self.pending.pop_back() {
							Some(LooseItem::ListItem { index }) => match self.queue[index] {
								Event::Output(MarkupEvent::Open(ref mut block)) => match block {
									Block::ListItem(ref mut info) => {
										info.loose = Some(loose);
										if loose {
											match self.pending.back_mut() {
												Some(LooseItem::List {
													ref mut loose,
													ref mut end,
													..
												}) => {
													*end = close_info.list.marker_pos.line;
													*loose = true; // store loose on the parent list also
												}
												_ => unreachable!("ListItem's pending parent is not a List"),
											}
										}
									}
									_ => unreachable!("block in queue event for pending ListItem is not ListItem"),
								},
								_ => unreachable!("queue event in pending ListItem is not an Open"),
							},
							_ => unreachable!("pending ListItem and close event did not match"),
						}
					} else if let Event::Output(MarkupEvent::Close(block)) = &event {
						let (_, end) = block.line_range();
						on_close(&mut self.pending, end);
					}
					self.queue.push_back(event);
				} else {
					// we should never reach the end of events with a pending
					// list open, assuming that we flush all pending close
					// events at the end of the input.
					debug_assert!(self.pending.len() == 0);
				}

				if self.pending.len() == 0 {
					// When we close the last pending item we still need to
					// update child Paragraph blocks with the loose
					// information:
					let mut loose = VecDeque::new();

					fn get_loose(stack: &VecDeque<Option<bool>>) -> Option<bool> {
						if let Some(value) = stack.back() {
							*value
						} else {
							None
						}
					}

					for it in self.queue.iter_mut() {
						if let Event::Output(ref mut markup) = it {
							match markup {
								MarkupEvent::Open(ref mut block) => {
									match block {
										Block::List(info) => {
											// Push the loose information on the
											// list.
											loose.push_back(info.loose);
										}
										Block::ListItem(ref mut info) => {
											// Update the list level information
											// for the item
											info.list.loose = get_loose(&loose);
										}
										Block::Paragraph(ref mut text) => {
											text.loose = get_loose(&loose);
										}
										_ => {
											// push a None onto to stack to
											// stop the "loose" propagating to
											// children
											loose.push_back(None);
										}
									}
								}
								MarkupEvent::Close(ref mut block) => {
									// `Close` event mirrors the `Open` above.
									match block {
										Block::List(_) => {
											loose.pop_back();
										}
										Block::ListItem(ref mut info) => {
											info.list.loose = get_loose(&loose);
										}
										Block::Paragraph(ref mut text) => {
											text.loose = get_loose(&loose);
										}
										_ => {
											loose.pop_back();
										}
									}
								}
								MarkupEvent::Code(_) | MarkupEvent::Inline(_) | MarkupEvent::Raw(_) => {
									// don't care about text
								}
							}
						}
					}
				}
			}
		}
	}

	fn read_next(&mut self) -> Option<Event<'a>> {
		let (next_state, result) = loop {
			dbg_print!(" STATE : {:?}", self.state);
			self.state = match std::mem::take(&mut self.state) {
				IteratorState::End => {
					break (IteratorState::End, None);
				}

				IteratorState::Start => {
					// Consume next block event and dispatch to one of the
					// handlers
					if let Some(next) = self.blocks.next() {
						dbg_print!(" BLOCK : {:?}", next);
						match next {
							BlockEvent::Start(container) => IteratorState::HandleStart(container),
							BlockEvent::End(container) => IteratorState::HandleEnd(container),
							BlockEvent::Leaf(leaf) => IteratorState::HandleLeafOrBreak(leaf),
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
					// close a pending open list before continuing
					if let Some(ParentContainer::List(_)) = self.parents.back() {
						let is_item = if let Container::ListItem(_) = &block {
							true
						} else {
							false
						};
						if !is_item {
							break self.close_list(IteratorState::HandleStart(block));
						}
					}

					match block {
						Container::BlockQuote(pos) => {
							self.parents.push_back(ParentContainer::BlockQuote(pos));
							IteratorState::GenerateOpen(Block::BlockQuote(pos))
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
								Some(ParentContainer::BlockQuote(..)) => (true, false),
								Some(ParentContainer::ListItem(..)) => (true, false),
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
					// close a pending open list before continuing
					if let Some(ParentContainer::List(_)) = self.parents.back() {
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
						Container::BlockQuote(..) => match self.parents.pop_back() {
							Some(ParentContainer::BlockQuote(pos)) => {
								Self::close_container_event(ParentContainer::BlockQuote(pos))
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

				// Handles semantic break blocks specially since they can close
				// block level items, such as list items:
				IteratorState::HandleLeafOrBreak(Leaf::Break(pos)) => {
					if let Some(ParentContainer::List(_)) = self.parents.back() {
						break self.close_list(IteratorState::HandleLeaf(Leaf::Break(pos)));
					}
					IteratorState::HandleLeaf(Leaf::Break(pos))
				}
				IteratorState::HandleLeafOrBreak(leaf) => IteratorState::HandleLeaf(leaf),

				// Start handling of a `BlockEvent::Leaf(leaf)` event
				IteratorState::HandleLeaf(leaf) => match Self::parse_leaf(leaf) {
					LeafOrReference::Leaf(block, text, mode) => {
						let event = Event::Output(MarkupEvent::Open(block.clone()));
						break (IteratorState::LeafInline(block, text, mode), Some(event));
					}
					LeafOrReference::Reference(link_ref) => {
						let event = Event::Reference(link_ref);
						break (IteratorState::Start, Some(event));
					}
					LeafOrReference::Table(table) => IteratorState::Table(table, TagMode::Start),
				},

				// Generates the text of a leaf block.
				IteratorState::LeafInline(block, span, mode) => {
					// TODO: parse inlines
					let event = match mode {
						SpanMode::Text => Event::Output(MarkupEvent::Inline(span)),
						SpanMode::Code => Event::Output(MarkupEvent::Code(span)),
						SpanMode::Raw => Event::Output(MarkupEvent::Raw(span)),
					};
					break (IteratorState::LeafEnd(block), Some(event));
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
							IteratorState::TableCell(table, section, body, row, cell, TagMode::Start)
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

				IteratorState::TableCell(table, section, body, row, cell, tag) => match tag {
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
						let event = Event::Output(MarkupEvent::Inline(cell.text.clone()));
						let state = IteratorState::TableCell(table, section, body, row, cell, TagMode::End);
						break (state, Some(event));
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
				},
			}
		};

		dbg_print!("OUTPUT : {:?}", result);
		self.state = next_state;
		result
	}

	fn parse_leaf(leaf: Leaf<'a>) -> LeafOrReference {
		match leaf {
			Leaf::Paragraph { text } => LeafOrReference::Leaf(Block::Paragraph(text.clone()), text, SpanMode::Text),
			Leaf::HTML { code, .. } => LeafOrReference::Leaf(Block::HTML(code.clone()), code, SpanMode::Raw),
			Leaf::LinkReference { url, label, title, .. } => LeafOrReference::Reference(LinkReference {
				label: label,
				title: title,
				url:   url,
			}),
			Leaf::IndentedCode { code } => LeafOrReference::Leaf(Block::Code(code.clone()), code, SpanMode::Code),
			Leaf::FencedCode { code, lang, info, .. } => {
				let info = FencedCodeInfo {
					code:     code.clone(),
					info:     info,
					language: lang,
				};
				LeafOrReference::Leaf(Block::FencedCode(info), code, SpanMode::Code)
			}
			Leaf::Break(pos) => LeafOrReference::Leaf(Block::Break(pos), Span::default(), SpanMode::Code),
			Leaf::Header { level, text } => {
				let level = match level {
					1 => HeaderLevel::H1,
					2 => HeaderLevel::H2,
					3 => HeaderLevel::H3,
					4 => HeaderLevel::H4,
					5 => HeaderLevel::H5,
					6 => HeaderLevel::H6,
					_ => unreachable!(),
				};
				LeafOrReference::Leaf(Block::Header(level, text.clone()), text, SpanMode::Text)
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
			ParentContainer::BlockQuote(pos) => Block::BlockQuote(pos),
			ParentContainer::ListItem(info) => Block::ListItem(ListItemInfo::from_block_info(info)),
			ParentContainer::List(info) => Block::List(ListInfo::from_block_info(info)),
		};
		Some(Event::Output(MarkupEvent::Close(block)))
	}
}

#[derive(Debug)]
enum SpanMode {
	Text,
	Code,
	Raw,
}

enum LeafOrReference<'a> {
	Leaf(Block<'a>, Span<'a>, SpanMode),
	Reference(LinkReference<'a>),
	Table(TableInfo<'a>),
}

impl<'a> Iterator for MarkdownIterator<'a> {
	type Item = Event<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.get_next()
	}
}

#[cfg(test)]
mod test;
