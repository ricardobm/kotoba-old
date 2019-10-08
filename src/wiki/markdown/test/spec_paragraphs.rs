use super::*;

// spell-checker: disable

mod markdown_spec_paragraphs {
	use super::*;

	#[test]
	fn should_parse() {
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
	fn should_parse_with_any_line_breaks() {
		test_raw("aaa\n\nbbb", "<p>aaa</p>\n<p>bbb</p>");
		test_raw("aaa\r\rbbb", "<p>aaa</p>\n<p>bbb</p>");
		test_raw("aaa\r\n\r\nbbb", "<p>aaa</p>\n<p>bbb</p>");

		test_raw("\naaa\n\nbbb\n", "<p>aaa</p>\n<p>bbb</p>");
		test_raw("\raaa\r\rbbb\r", "<p>aaa</p>\n<p>bbb</p>");
		test_raw("\r\naaa\r\n\r\nbbb\r\n", "<p>aaa</p>\n<p>bbb</p>");

		test_raw("\naaa\nbbb\n", "<p>aaa\nbbb</p>");
		test_raw("\raaa\rbbb\r", "<p>aaa\nbbb</p>");
		test_raw("\r\naaa\r\nbbb\r\n", "<p>aaa\nbbb</p>");
	}

	#[test]
	fn should_trim_trailing_spaces() {
		test_raw("aaa \n123 \n\nbbb ", "<p>aaa\n123</p>\n<p>bbb</p>");
		test_raw("aaa \r123 \r\rbbb ", "<p>aaa\n123</p>\n<p>bbb</p>");
		test_raw("aaa \r\n123 \r\n\r\nbbb ", "<p>aaa\n123</p>\n<p>bbb</p>");

		test_raw("\naaa\n\nbbb\n", "<p>aaa</p>\n<p>bbb</p>");
		test_raw("\raaa\r\rbbb\r", "<p>aaa</p>\n<p>bbb</p>");
		test_raw("\r\naaa\r\n\r\nbbb\r\n", "<p>aaa</p>\n<p>bbb</p>");

		test_raw("\naaa\nbbb\n", "<p>aaa\nbbb</p>");
		test_raw("\raaa\rbbb\r", "<p>aaa\nbbb</p>");
		test_raw("\r\naaa\r\nbbb\r\n", "<p>aaa\nbbb</p>");
	}

	#[test]
	fn can_contain_multiple_lines_but_no_blank_lines() {
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
	fn multiple_blank_lines_have_no_effect() {
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
	fn leading_spaces_are_skipped() {
		// example 192
		test_raw("  aaa\n bbb", "<p>aaa\nbbb</p>");
	}

	#[test]
	fn lines_after_the_first_may_be_indented_by_any_amount() {
		// example 193
		test_raw(
			"aaa\n             bbb\n                                       ccc",
			"<p>aaa\nbbb\nccc</p>",
		);
	}

	#[test]
	fn first_line_may_be_indented_up_to_three_spaces() {
		// example 194
		test_raw("   aaa\nbbb", "<p>aaa\nbbb</p>");

		// example 195
		test_raw("    aaa\nbbb", "<pre><code>aaa</code></pre>\n<p>bbb</p>");
	}

	#[test]
	fn final_spaces_are_stripped() {
		// example 196
		test_raw("aaa     \nbbb     ", "<p>aaa<br/>\nbbb</p>");
	}

	#[test]
	fn ignores_blank_lines_between_blocks() {
		// example 197
		test_raw("  \n\naaa\n  \n\n# aaa\n\n  ", "<p>aaa</p>\n<h1>aaa</h1>");
	}
}
