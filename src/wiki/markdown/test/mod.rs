use super::*;

mod basics;

mod spec_atx_headings;
mod spec_autolinks;
mod spec_basics;
mod spec_breaks;
mod spec_emphasis;
mod spec_entities;
mod spec_escapes;
mod spec_fenced_code;
mod spec_html_blocks;
mod spec_images;
mod spec_indented_code;
mod spec_link_refs;
mod spec_links;
mod spec_paragraphs;
mod spec_raw_html;
mod spec_setext_headings;
mod spec_tables;

fn test(input: &str, expected: &str) {
	let input = common::text(input);
	let expected = common::text(expected);
	test_raw(&input, &expected);
}

fn test_raw(input: &str, expected: &str) {
	let result = to_html(parse_markdown(input)).unwrap();
	assert_eq!(result, expected);
}
