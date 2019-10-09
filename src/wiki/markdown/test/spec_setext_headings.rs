use super::*;

// spell-checker: disable

mod markdown_spec_setext_headings {
	use super::*;

	#[test]
	fn example_50_basic_parsing() {
		// example 50
		test(
			r##"
				Foo *bar*
				=========

				Foo *bar*
				---------
			"##,
			r##"
				<h1>Foo <em>bar</em></h1>
				<h2>Foo <em>bar</em></h2>
			"##,
		);
	}

	#[test]
	fn example_51_can_span_more_than_one_line() {
		// example 51
		test(
			r##"
				Foo *bar
				baz*
				====
			"##,
			r##"
				<h1>Foo <em>bar
				baz</em></h1>
			"##,
		);
	}

	#[test]
	fn example_52_should_trim_content() {
		// example 52
		test_raw("  Foo *bar\nbaz*\t\n====", "<h1>Foo <em>bar\nbaz</em></h1>");
	}

	#[test]
	fn example_53_should_allow_any_length_of_underline() {
		// example 53
		test(
			r##"
				Foo
				-------------------------

				Foo
				=
			"##,
			r##"
				<h2>Foo</h2>
				<h1>Foo</h1>
			"##,
		);
	}

	#[test]
	fn example_54_can_be_indented() {
		// example 54
		test_raw(
			"   Foo\n---\n\n  Foo\n-----\n\n  Foo\n  ===",
			"<h2>Foo</h2>\n<h2>Foo</h2>\n<h1>Foo</h1>",
		);
	}

	#[test]
	fn example_55_four_spaces_is_too_much() {
		// example 55
		test_raw(
			"    Foo\n    ---\n\n    Foo\n---",
			"<pre><code>Foo\n---\n\nFoo</code></pre>\n<hr/>",
		);
	}

	#[test]
	fn example_56_underline_can_be_indented_and_have_trailing_spaces() {
		// example 56
		test_raw("Foo\n   ----      ", "<h2>Foo</h2>");
	}

	#[test]
	fn example_57_four_spaces_is_too_much_in_underline() {
		// example 57 - four spaces is too much
		test_raw("Foo\n    ---", "<p>Foo\n---</p>");
	}

	#[test]
	fn example_58_dont_allow_internal_spaces_in_underline() {
		// example 58
		test(
			r##"
				Foo
				= =

				Foo
				--- -
			"##,
			r##"
				<p>Foo
				= =</p>
				<p>Foo</p>
				<hr/>
			"##,
		);
	}

	#[test]
	fn example_59_trailing_spaces_do_not_cause_a_line_break() {
		// example 59
		test_raw("Foo  \n-----", "<h2>Foo</h2>");
	}

	fn example_60_backslash_does_not_cause_a_line_break() {
		// example 60
		test_raw("Foo\\\n----", "<h2>Foo\\</h2>");
	}

	#[test]
	fn example_61_has_precedence_over_inlines() {
		// example 61
		test(
			r##"
				`Foo
				----
				`

				<a title="a lot
				---
				of dashes"/>
			"##,
			r##"
				<h2>`Foo</h2>
				<p>`</p>
				<h2>&lt;a title=&quot;a lot</h2>
				<p>of dashes&quot;/&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_62_cannot_be_a_lazy_continuation() {
		// example 62
		test(
			r##"
				> Foo
				---
			"##,
			r##"
				<blockquote>
				<p>Foo</p>
				</blockquote>
				<hr/>
			"##,
		);
	}

	fn example_63_cannot_be_a_lazy_continuation() {
		// example 63
		test(
			r##"
				> foo
				bar
				===
			"##,
			r##"
				<blockquote>
				<p>foo
				bar
				===</p>
				</blockquote>
			"##,
		);
	}

	fn example_64_cannot_be_a_lazy_continuation() {
		// example 64
		test(
			r##"
				- Foo
				---
			"##,
			r##"
				<ul>
				<li>Foo</li>
				</ul>
				<hr/>
			"##,
		);
	}

	#[test]
	fn example_65_does_not_require_blank_lines() {
		// example 65
		test(
			r##"
				Foo
				Bar
				---
			"##,
			r##"
				<h2>Foo
				Bar</h2>
			"##,
		);
	}

	fn example_66_consumes_whole_paragraph() {
		// example 66
		test(
			r##"
				---
				Foo
				---
				Bar
				---
				Baz
			"##,
			r##"
				<hr/>
				<h2>Foo</h2>
				<h2>Bar</h2>
				<p>Baz</p>
			"##,
		);
	}

	#[test]
	fn example_67_cannot_be_empty() {
		// example 67
		test(
			r##"
				====
			"##,
			r##"
				<p>====</p>
			"##,
		);
	}

	#[test]
	fn example_68_should_not_be_interpretable_as_blocks() {
		// example 68
		test(
			r##"
				---
				---
			"##,
			r##"
				<hr/>
				<hr/>
			"##,
		);
	}

	fn example_69_should_not_be_interpretable_as_blocks() {
		// example 69
		test(
			r##"
				- foo
				-----
			"##,
			r##"
				<ul>
				<li>foo</li>
				</ul>
				<hr/>
			"##,
		);
	}

	fn example_70_should_not_be_interpretable_as_blocks() {
		// example 70
		test_raw("    foo\n---", "<pre><code>foo</code></pre>\n<hr/>");
	}

	fn example_71_should_not_be_interpretable_as_blocks() {
		// example 71
		test(
			r##"
				> foo
				-----
			"##,
			r##"
				<blockquote>
				<p>foo</p>
				</blockquote>
				<hr/>
			"##,
		);
	}

	fn example_72_should_not_be_interpretable_as_blocks() {
		// example 72
		test(
			r##"
				\> foo
				------
			"##,
			r##"
				<h2>&gt; foo</h2>
			"##,
		);
	}

	#[test]
	fn example_73_should_handle_multiline() {
		// example 73
		test(
			r##"
				Foo

				bar
				---
				baz
			"##,
			r##"
				<p>Foo</p>
				<h2>bar</h2>
				<p>baz</p>
			"##,
		);
	}

	fn example_74_should_handle_multiline() {
		// example 74
		test(
			r##"
				Foo
				bar

				---

				baz
			"##,
			r##"
				<p>Foo
				bar</p>
				<hr/>
				<p>baz</p>
			"##,
		);
	}

	fn example_75_should_handle_multiline() {
		// example 75
		test(
			r##"
				Foo
				bar
				* * *
				baz
			"##,
			r##"
				<p>Foo
				bar</p>
				<hr/>
				<p>baz</p>
			"##,
		);
	}

	fn example_76_should_handle_multiline() {
		// example 76
		test(
			r##"
				Foo
				bar
				\---
				baz
			"##,
			r##"
				<p>Foo
				bar
				---
				baz</p>
			"##,
		);
	}
}
