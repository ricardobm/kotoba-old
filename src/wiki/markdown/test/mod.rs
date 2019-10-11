use super::*;

mod basics;

mod spec_atx_headings;
mod spec_autolinks;
mod spec_basics;
mod spec_blockquotes;
mod spec_breaks;
mod spec_emphasis;
mod spec_entities;
mod spec_fenced_code;
mod spec_html_blocks;
mod spec_images;
mod spec_indented_code;
mod spec_inline_code;
mod spec_inlines;
mod spec_link_refs;
mod spec_links;
mod spec_list_items;
mod spec_lists;
mod spec_paragraphs;
mod spec_raw_html;
mod spec_setext_headings;
mod spec_tables;
mod spec_thematic_breaks;

fn test(input: &str, expected: &str) {
	let input = common::text(input);
	let expected = common::text(expected);
	test_raw(&input, &expected);
}

fn test_raw(input: &str, expected: &str) {
	let result = to_html(parse_markdown(input)).unwrap();
	assert_eq!(result, expected);
}

fn test_raw_in(input: &str, expected: &str) {
	let result = to_html(parse_markdown(input)).unwrap();
	let expected = common::text(expected);
	assert_eq!(result, expected);
}
