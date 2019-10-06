use super::*;

mod basics;

mod spec_atx_headings;
mod spec_autolinks;
mod spec_breaks;
mod spec_emphasis;
mod spec_entities;
mod spec_escapes;
mod spec_images;
mod spec_links;
mod spec_raw_html;

fn test(input: &str, expected: &str) {
	let input = common::text(input);
	let expected = common::text(expected);
	test_raw(&input, &expected);
}

fn test_raw(input: &str, expected: &str) {
	let result = to_html(parse_markdown(input)).unwrap();
	assert_eq!(result, expected);
}
