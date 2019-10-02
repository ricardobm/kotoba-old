use super::*;

mod basics;

mod spec_atx_headings;
mod spec_autolinks;
mod spec_breaks;
mod spec_entities;
mod spec_escapes;

fn test(input: &str, expected: &str) {
	let input = common::text(input);
	let expected = common::text(expected);
	test_raw(&input, &expected);
}

fn test_raw(input: &str, expected: &str) {
	let result = to_html(parse_markdown(input)).unwrap();
	assert_eq!(result, expected);
}
