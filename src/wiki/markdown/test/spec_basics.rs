use super::*;

// spell-checker: disable

mod markdown_spec_basics {
	use super::*;

	#[test]
	fn allow_empty_document() {
		test("", "");
		test("   \t", "");
		test("\n\r\r\n", "");
		test("    \n    \r    \r\n\t\n", "");
	}

	#[test]
	fn zero_byte_should_be_replaced() {
		test_raw("\0", "<p>\u{FFFD}</p>");
		test_raw("    \0", "<pre><code>\u{FFFD}</code></pre>");
		test_raw("[foo](</url/\0>)", "<p><a href=\"/url/\u{FFFD}\">foo</a></p>");
	}

	#[test]
	fn tabs_can_be_used_instead_of_four_spaces() {
		// example 1
		test_raw("\tfoo\tbaz\t\tbim", "<pre><code>foo\tbaz\t\tbim</code></pre>");
		// example 2
		test_raw("  \tfoo\tbaz\t\tbim", "<pre><code>foo\tbaz\t\tbim</code></pre>");
		// example 3
		test_raw("    a\ta\n    ὐ\ta", "<pre><code>a\ta\nὐ\ta</code></pre>");
		// example 4
		test_raw(
			"  - foo\n\n\tbar\n\n\t123",
			"<ul>\n<li>\n<p>foo</p>\n<p>bar</p>\n<p>123</p>\n</li>\n</ul>",
		);
		// example 5
		test_raw(
			"- foo\n\n\t\tbar\n\t\t123",
			"<ul>\n<li>\n<p>foo</p>\n<pre><code>  bar\n  123</code></pre>\n</li>\n</ul>",
		);
		// example 6
		test_raw(
			">\t\tfoo\n>\t\tbar",
			"<blockquote>\n<pre><code>  foo\n  bar</code></pre>\n</blockquote>",
		);
		// example 7
		test_raw(
			"-\t\tfoo\n\t\tbar",
			"<ul>\n<li>\n<pre><code>  foo\n  bar</code></pre>\n</li>\n</ul>",
		);
		// example 8
		test_raw("    foo\n\tbar\n  \t123", "<pre><code>foo\nbar\n123</code></pre>");
		// example 9
		test_raw(
			" - foo\n   - bar\n\t - baz",
			"<ul>\n<li>foo\n<ul>\n<li>bar\n<ul>\n<li>baz</li>\n</ul>\n</li>\n</ul>\n</li>\n</ul>",
		);
		// example 10
		test_raw("#\tFoo", "<h1>Foo</h1>");
		// example 11
		test_raw("*\t*\t*\t", "<hr/>");
	}

	#[test]
	fn block_inline_precedence() {
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
