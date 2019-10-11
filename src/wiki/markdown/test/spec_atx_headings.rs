use super::*;

// spell-checker: disable

mod markdown_spec_atx_headings {
	use super::*;

	#[test]
	fn example_32_should_parse() {
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
	}

	#[test]
	fn example_33_more_than_six_is_not_a_heading() {
		// example 33
		test("####### foo", "<p>####### foo</p>");
	}

	#[test]
	fn example_34_should_require_at_least_one_space() {
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
	fn example_35_should_not_parse_escaped() {
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
	fn example_36_should_parse_content_as_inlines() {
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
	fn example_37_should_trim_spaces() {
		// example 37
		test_raw("#\t              foo\t                 ", "<h1>foo</h1>");
	}

	#[test]
	fn example_38_should_allow_indentation() {
		// example 38
		test_raw(" ### foo", "<h3>foo</h3>");
		test_raw("  ## foo", "<h2>foo</h2>");
		test_raw("   # foo", "<h1>foo</h1>");
	}

	#[test]
	fn example_39_should_allow_indentation() {
		// example 39
		test_raw("    # foo", "<pre><code># foo</code></pre>");
	}

	#[test]
	fn example_40_should_allow_indentation() {
		// example 40
		test_raw("foo\n    # bar", "<p>foo\n# bar</p>");
	}

	#[test]
	fn example_41_should_allow_closing_sequence() {
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
	}

	#[test]
	fn example_42_closing_does_not_need_to_be_same_length() {
		// example 42
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
	}

	#[test]
	fn example_43_spaces_are_allowed_after_closing() {
		// example 43
		test_raw("### foo ###     ", "<h3>foo</h3>");
	}

	#[test]
	fn example_44_hash_sequence_in_the_middle() {
		// example 44
		test("### foo ### b", "<h3>foo ### b</h3>");
	}

	#[test]
	fn example_45_closing_must_be_preceeded_by_a_space() {
		// example 45
		test_raw("# foo 1#", "<h1>foo 1#</h1>");
		test_raw("# foo 2 #", "<h1>foo 2</h1>");
		test_raw("# foo 3\t#", "<h1>foo 3</h1>");
	}

	#[test]
	fn example_46_backslash_escapes_do_not_count() {
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
	fn example_47_dont_require_blank_lines() {
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
	}

	#[test]
	fn example_48_can_interrupt_paragraphs() {
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
	fn example_49_can_be_empty() {
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
