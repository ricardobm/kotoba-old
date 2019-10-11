use super::*;

// spell-checker: disable

mod markdown_spec_paragraphs {
	use super::*;

	#[test]
	fn example_189_should_parse() {
		// example 189
		test(
			r##"
				aaa

				bbb
			"##,
			r##"
				<p>aaa</p>
				<p>bbb</p>
			"##,
		);
	}

	#[test]
	fn example_190_can_contain_multiple_lines_but_no_blank_lines() {
		// example 190
		test(
			r##"
				aaa
				bbb


				ccc
				ddd
			"##,
			r##"
				<p>aaa
				bbb</p>
				<p>ccc
				ddd</p>
			"##,
		);

		test(
			r##"
				aaa
				bbb
				123

				ccc
				ddd
				456
			"##,
			r##"
				<p>aaa
				bbb
				123</p>
				<p>ccc
				ddd
				456</p>
			"##,
		);
	}

	#[test]
	fn example_191_multiple_blank_lines_have_no_effect() {
		// example 191
		test(
			r##"
				aaa


				bbb
			"##,
			r##"
				<p>aaa</p>
				<p>bbb</p>
			"##,
		);
	}

	#[test]
	fn example_192_leading_spaces_are_skipped() {
		// example 192
		test_raw("  aaa\n bbb", "<p>aaa\nbbb</p>");
	}

	#[test]
	fn example_193_lines_after_the_first_may_be_indented_by_any_amount() {
		// example 193
		test_raw(
			"aaa\n             bbb\n                                       ccc",
			"<p>aaa\nbbb\nccc</p>",
		);
	}

	#[test]
	fn example_194_first_line_may_be_indented_up_to_three_spaces() {
		// example 194
		test_raw("   aaa\nbbb", "<p>aaa\nbbb</p>");
	}

	#[test]
	fn example_195_first_line_may_be_indented_up_to_three_spaces() {
		// example 195
		test_raw("    aaa\nbbb", "<pre><code>aaa</code></pre>\n<p>bbb</p>");
	}

	#[test]
	fn example_196_final_spaces_are_stripped() {
		// example 196
		test_raw("aaa     \nbbb     ", "<p>aaa<br/>\nbbb</p>");
	}

	#[test]
	fn example_197_ignores_blank_lines_between_blocks() {
		// example 197
		test_raw("  \n\naaa\n  \n\n# aaa\n\n  ", "<p>aaa</p>\n<h1>aaa</h1>");
	}
}
