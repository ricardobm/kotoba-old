use super::*;

// spell-checker: disable

mod markdown_spec_atx_headings {
	use super::*;

	#[test]
	fn should_parse() {
		// example 32
		test(
			r##"
				# foo
				## foo
				### foo
				#### foo
				##### foo
				###### foo
			"##,
			r##"
				<h1>foo</h1>
				<h2>foo</h2>
				<h3>foo</h3>
				<h4>foo</h4>
				<h5>foo</h5>
				<h6>foo</h6>
			"##,
		);

		// example 33 - more than six `#` is not a heading
		test("####### foo", "<p>####### foo</p>");

		// example 34 - should require at least one space
		test(
			r##"
				#5 bolt

				#hashtag

				#	5 bolt

				#	hashtag
			"##,
			r##"
				<p>#5 bolt</p>
				<p>#hashtag</p>
				<h1>5 bolt</h1>
				<h1>hashtag</h1>
			"##,
		);
	}

	#[test]
	fn should_not_parse_escaped() {
		// example 35
		test(
			r##"
				\## foo
			"##,
			r##"
				<p>## foo</p>
			"##,
		);
	}

	#[test]
	fn should_parse_content_as_inlines() {
		// example 36
		test(
			r##"
				# foo *bar* \*baz\*
			"##,
			r##"
				<h1>foo <em>bar</em> *baz*</h1>
			"##,
		);
	}

	#[test]
	fn should_trim_spaces() {
		// example 37
		test_raw("#\t              foo\t                 ", "<h1>foo</h1>");
	}

	#[test]
	fn should_allow_indentation() {
		// example 38
		test_raw(" ### foo", "<h3>foo</h3>");
		test_raw("  ## foo", "<h2>foo</h2>");
		test_raw("   # foo", "<h1>foo</h1>");

		// example 39
		test_raw("    # foo", "<pre><code># foo</code></pre>");

		// example 40
		test_raw("foo\n    # bar", "<p>foo\n    # bar</p>");
	}

	#[test]
	fn should_allow_closing_sequence() {
		// example 41
		test(
			r##"
				## foo ##
				   ###   bar    ###
			"##,
			r##"
				<h2>foo</h2>
				<h3>bar</h3>
			"##,
		);

		// example 42 - closing does not need to be the same length
		test(
			r##"
				# foo ##################################
				##### foo ##
			"##,
			r##"
				<h1>foo</h1>
				<h5>foo</h5>
			"##,
		);

		// example 43 - spaces are allowed after the closing
		test_raw("### foo ###     ", "<h3>foo</h3>");

		// example 44 - `#` sequence in the middle is not a closing sequence
		test("### foo ### b", "<h3>foo ### b</h3>");

		// example 45 - closing sequence must be preceded by a space
		test_raw("# foo 1#", "<h1>foo 1#</h1>");
		test_raw("# foo 2 #", "<h1>foo 2</h1>");
		test_raw("# foo 3\t#", "<h1>foo 3</h1>");

		// example 46 - backslash escaped characters do not count
		test(
			r##"
				### foo \###
				## foo #\##
				# foo \#
			"##,
			r##"
				<h3>foo ###</h3>
				<h2>foo ###</h2>
				<h1>foo #</h1>
			"##,
		);
	}

	#[test]
	fn dont_require_blank_lines_and_can_interrupt_paragraphs() {
		// example 47
		test(
			r##"
				****
				## foo
				****
			"##,
			r##"
				<hr/>
				<h2>foo</h2>
				<hr/>
			"##,
		);

		// example 48
		test(
			r##"
				Foo bar
				# baz
				Bar foo
			"##,
			r##"
				<p>Foo bar</p>
				<h1>baz</h1>
				<p>Bar foo</p>
			"##,
		);
	}

	#[test]
	fn can_be_empty() {
		// example 49
		test(
			r##"
				##
				#
				### ###
			"##,
			r##"
				<h2></h2>
				<h1></h1>
				<h3></h3>
			"##,
		);
	}
}
