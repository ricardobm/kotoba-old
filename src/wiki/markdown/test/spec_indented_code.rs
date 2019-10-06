use super::*;

// spell-checker: disable

mod markdown_spec_indented_code {
	use super::*;

	#[test]
	fn should_parse() {
		// example 77
		test_raw(
			"    a simple\n      indented code block",
			"<pre><code>a simple\n  indented code block</code></pre>",
		);

		test_raw(
			"\ta simple\n\t  indented code block with tabs",
			"<pre><code>a simple\n  indented code block with tabs</code></pre>",
		);
	}

	#[test]
	fn has_lower_priority_than_list_continuation() {
		// example 78
		test_raw("  - foo\n\n    bar", "<ul>\n<li>\n<p>foo</p>\n<p>bar</p>\n</li>\n</ul>");

		// example 79
		test_raw(
			"1.  foo\n\n    - bar",
			"<ol>\n<li>\n<p>foo</p>\n<ul>\n<li>bar</li>\n</ul>\n</li>\n</ol>",
		);
	}

	#[test]
	fn contents_are_raw_text() {
		// example 80
		test_raw(
			"    <a/>\n    *hi*\n\n    - one",
			"<pre><code>&lt;a/&gt;\n*hi*\n\n- one</code></pre>",
		);
	}

	#[test]
	fn should_allow_blank_lines() {
		// example 81
		test_raw(
			"    chunk1\n\n    chunk2\n\n\n\n    chunk3",
			"<pre><code>chunk1\n\nchunk2\n\n\n\nchunk3</code></pre>",
		);
	}

	#[test]
	fn should_include_interior_spaces() {
		// example 82
		test_raw(
			"    chunk1\n      \n      chunk2",
			"<pre><code>chunk1\n  \n  chunk2</code></pre>",
		);
	}

	#[test]
	fn cannot_interrupt_a_paragraph() {
		// example 83
		test_raw("Foo\n    bar", "<p>Foo\n    bar</p>");
	}

	#[test]
	fn can_be_followed_by_a_paragraph() {
		// example 84
		test_raw("    foo\nbar", "<pre><code>foo</code></pre>\n<p>bar</p>");
	}

	#[test]
	fn do_not_require_blank_lines_between_blocks() {
		// example 85
		test_raw(
			"# Heading\n    foo\nHeading\n------\n    foo\n----",
			"<h1>Heading</h1>\n<pre><code>foo</code></pre>\n<h2>Heading</h2>\n<pre><code>foo</code></pre>\n<hr/>",
		);
	}

	#[test]
	fn first_line_can_be_indented_more_than_four_spaces() {
		// example 86
		test_raw("        foo\n    bar", "<pre><code>    foo\nbar</code></pre>");
	}

	#[test]
	fn does_not_include_blank_lines_around_it() {
		// example 87
		test_raw("\n    \n    foo\n    ", "<pre><code>foo</code></pre>");
	}

	#[test]
	fn includes_trailing_space() {
		// example 88
		test_raw("    foo  ", "<pre><code>foo  </code></pre>");
	}
}
