use super::*;

mod basics;

fn test(input: &str, expected: &str) {
	let input = common::text(input);
	let expected = common::text(expected);

	let result = to_html(parse_markdown(input.as_str())).unwrap();
	assert_eq!(result, expected);
}