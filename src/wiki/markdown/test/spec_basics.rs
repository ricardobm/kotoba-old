use super::*;

// spell-checker: disable

mod markdown_spec_basics {
	use super::*;

	#[test]
	fn example_1_tabs() {
		// example 1
		test_raw("\tfoo\tbaz\t\tbim", "<pre><code>foo\tbaz\t\tbim</code></pre>");
	}

	#[test]
	fn example_2_tabs() {
		// example 2
		test_raw("  \tfoo\tbaz\t\tbim", "<pre><code>foo\tbaz\t\tbim</code></pre>");
	}

	#[test]
	fn example_3_tabs() {
		// example 3
		test_raw("    a\ta\n    ὐ\ta", "<pre><code>a\ta\nὐ\ta</code></pre>");
	}

	#[test]
	fn example_4_tabs() {
		// example 4
		test_raw(
			"  - foo\n\n\tbar\n\n\t123",
			"<ul>\n<li>\n<p>foo</p>\n<p>bar</p>\n<p>123</p>\n</li>\n</ul>",
		);
	}

	#[test]
	fn example_5_tabs() {
		// example 5
		test_raw(
			"- foo\n\n\t\tbar\n\t\t123",
			"<ul>\n<li>\n<p>foo</p>\n<pre><code>  bar\n  123</code></pre>\n</li>\n</ul>",
		);
	}

	#[test]
	fn example_6_tabs() {
		// example 6
		test_raw(
			">\t\tfoo\n>\t\tbar",
			"<blockquote>\n<pre><code>  foo\n  bar</code></pre>\n</blockquote>",
		);
	}

	#[test]
	fn example_7_tabs() {
		// example 7
		test_raw(
			"-\t\tfoo\n\t\tbar",
			"<ul>\n<li>\n<pre><code>  foo\n  bar</code></pre>\n</li>\n</ul>",
		);
	}

	#[test]
	fn example_8_tabs() {
		// example 8
		test_raw("    foo\n\tbar\n  \t123", "<pre><code>foo\nbar\n123</code></pre>");
	}

	#[test]
	fn example_9_tabs() {
		// example 9
		test_raw(
			" - foo\n   - bar\n\t - baz",
			"<ul>\n<li>foo\n<ul>\n<li>bar\n<ul>\n<li>baz</li>\n</ul>\n</li>\n</ul>\n</li>\n</ul>",
		);
	}

	#[test]
	fn example_10_tabs() {
		// example 10
		test_raw("#\tFoo", "<h1>Foo</h1>");
	}

	#[test]
	fn example_11_tabs() {
		// example 11
		test_raw("*\t*\t*\t", "<hr/>");
	}

	#[test]
	fn example_12_block_inline_precedence() {
		// example 12
		test(
			r##"
			- `one
			- two`
		"##,
			r##"
			<ul>
			<li>`one</li>
			<li>two`</li>
			</ul>
		"##,
		);
	}
}
