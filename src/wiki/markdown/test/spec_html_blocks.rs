use super::*;

// spell-checker: disable

mod markdown_spec_html_blocks {
	use super::*;

	#[test]
	fn should_parse() {
		// example 118 - ends with a blank line
		test(
			r##"
				<table><tr><td>
				<pre>
				**Hello**,

				_world_.
				</pre>
				</td></tr></table>
			"##,
			r##"
				<table><tr><td>
				<pre>
				**Hello**,
				<p><em>world</em>.
				</pre></p>
				</td></tr></table>
			"##,
		);

		// example 119
		test(
			r##"
				<table>
					<tr>
						<td>
							hi
						</td>
					</tr>
				</table>

				okay.
			"##,
			r##"
				<table>
					<tr>
						<td>
							hi
						</td>
					</tr>
				</table>
				<p>okay.</p>
			"##,
		);

		// example 120
		test_raw(
			" <div>\n  *hello*\n         <foo><a>",
			" <div>\n  *hello*\n         <foo><a>",
		);

		// example 121
		test(
			r##"
				</div>
				*foo*
			"##,
			r##"
				</div>
				*foo*
			"##,
		);

		// example 122
		test(
			r##"
				<DIV CLASS="foo">

				*Markdown*

				</DIV>
			"##,
			r##"
				<DIV CLASS="foo">
				<p><em>Markdown</em></p>
				</DIV>
			"##,
		);

		// example 123
		test(
			r##"
				<div id="foo"
				  class="bar">
				</div>
			"##,
			r##"
				<div id="foo"
				  class="bar">
				</div>
			"##,
		);

		// example 124
		test(
			r##"
				<div id="foo" class="bar
				  baz">
				</div>
			"##,
			r##"
				<div id="foo" class="bar
				  baz">
				</div>
			"##,
		);

		// example 125
		test(
			r##"
				<div>
				*foo*

				*bar*
			"##,
			r##"
				<div>
				*foo*
				<p><em>bar</em></p>
			"##,
		);

		// example 126
		test(
			r##"
				<div id="foo"
				*hi*
			"##,
			r##"
				<div id="foo"
				*hi*
			"##,
		);

		// example 127
		test(
			r##"
				<div class
				foo
			"##,
			r##"
				<div class
				foo
			"##,
		);

		// example 128
		test(
			r##"
				<div *???-&&&-<---
				*foo*
			"##,
			r##"
				<div *???-&&&-<---
				*foo*
			"##,
		);

		// example 129
		test(
			r##"
				<div><a href="bar">*foo*</a></div>
			"##,
			r##"
				<div><a href="bar">*foo*</a></div>
			"##,
		);

		// example 130
		test(
			r##"
				<table><tr><td>
				foo
				</td></tr></table>
			"##,
			r##"
				<table><tr><td>
				foo
				</td></tr></table>
			"##,
		);

		// example 131
		test(
			r##"
				<div></div>
				``` c
				int x = 33;
				```
			"##,
			r##"
				<div></div>
				``` c
				int x = 33;
				```
			"##,
		);
	}

	#[test]
	fn should_parse_type_7() {
		// example 132
		test(
			r##"
				<a href="foo">
				*bar*
				</a>
			"##,
			r##"
				<a href="foo">
				*bar*
				</a>
			"##,
		);

		// example 133
		test(
			r##"
				<Warning>
				*bar*
				</Warning>
			"##,
			r##"
				<Warning>
				*bar*
				</Warning>
			"##,
		);

		// example 134
		test(
			r##"
				<i class="foo">
				*bar*
				</i>
			"##,
			r##"
				<i class="foo">
				*bar*
				</i>
			"##,
		);

		// example 135
		test(
			r##"
				</ins>
				*bar*
			"##,
			r##"
				</ins>
				*bar*
			"##,
		);

		// example 136
		test(
			r##"
				<del>
				*foo*
				</del>
			"##,
			r##"
				<del>
				*foo*
				</del>
			"##,
		);

		// example 137
		test(
			r##"
				<del>

				*foo*

				</del>
			"##,
			r##"
				<del>
				<p><em>foo</em></p>
				</del>
			"##,
		);

		// example 138
		test(
			r##"
				<del>*foo*</del>
			"##,
			r##"
				<p><del><em>foo</em></del></p>
			"##,
		);
	}

	#[test]
	fn should_parse_type_1() {
		// example 139
		test(
			r##"
				<pre language="haskell"><code>
				import Text.HTML.TagSoup

				main :: IO ()
				main = print $ parseTags tags
				</code></pre>
				okay
			"##,
			r##"
				<pre language="haskell"><code>
				import Text.HTML.TagSoup

				main :: IO ()
				main = print $ parseTags tags
				</code></pre>
				<p>okay</p>
			"##,
		);

		// example 140
		test(
			r##"
				<script type="text/javascript">
				// JavaScript example

				document.getElementById("demo").innerHTML = "Hello JavaScript!";
				</script>
				okay
			"##,
			r##"
				<script type="text/javascript">
				// JavaScript example

				document.getElementById("demo").innerHTML = "Hello JavaScript!";
				</script>
				<p>okay</p>
			"##,
		);

		// example 141
		test(
			r##"
				<style
					type="text/css">
				h1 {color:red;}

				p {color:blue;}
				</style>
				okay
			"##,
			r##"
				<style
					type="text/css">
				h1 {color:red;}

				p {color:blue;}
				</style>
				<p>okay</p>
			"##,
		);
	}

	#[test]
	fn should_handle_unclosed_type_1() {
		// example 142
		test(
			r##"
				<style
				  type="text/css">

				foo
			"##,
			r##"
				<style
				  type="text/css">

				foo
			"##,
		);

		// example 143
		test(
			r##"
				> <div>
				> foo

				bar
			"##,
			r##"
				<blockquote>
				<div>
				foo
				</blockquote>
				<p>bar</p>
			"##,
		);

		// example 144
		test(
			r##"
				- <div>
				- foo
			"##,
			r##"
				<ul>
				<li>
				<div>
				</li>
				<li>foo</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn type_1_end_tag_can_occur_on_the_same_line() {
		// example 145
		test(
			r##"
				<style>p{color:red;}</style>
				*foo*
			"##,
			r##"
				<style>p{color:red;}</style>
				<p><em>foo</em></p>
			"##,
		);

		// example 146
		test(
			r##"
				<!-- foo -->*bar*
				*baz*
			"##,
			r##"
				<!-- foo -->*bar*
				<p><em>baz</em></p>
			"##,
		);
	}

	#[test]
	fn includes_anything_after_end_tag() {
		// example 147
		test(
			r##"
				<script>
				foo
				</script>1. *bar*
			"##,
			r##"
				<script>
				foo
				</script>1. *bar*
			"##,
		);
	}

	#[test]
	fn supports_type_2_comments() {
		// example 148
		test(
			r##"
				<!-- Foo

				bar
					baz -->
				okay
			"##,
			r##"
				<!-- Foo

				bar
					baz -->
				<p>okay</p>
			"##,
		);
	}

	#[test]
	fn supports_type_3_processing_instructions() {
		// example 149
		test(
			r##"
				<?php

					echo '>';

				?>
				okay
			"##,
			r##"
				<?php

					echo '>';

				?>
				<p>okay</p>
			"##,
		);
	}

	#[test]
	fn supports_type_4_declarations() {
		// example 150
		test(
			r##"
				<!DOCTYPE html>
			"##,
			r##"
				<!DOCTYPE html>
			"##,
		);
	}

	#[test]
	fn supports_type_5_cdata() {
		// example 151
		test(
			r##"
				<![CDATA[
				function matchwo(a,b)
				{
					if (a < b && a < 0) then {
						return 1;

					} else {

						return 0;
					}
				}
				]]>
				okay
			"##,
			r##"
				<![CDATA[
				function matchwo(a,b)
				{
					if (a < b && a < 0) then {
						return 1;

					} else {

						return 0;
					}
				}
				]]>
				<p>okay</p>
			"##,
		);
	}

	#[test]
	fn can_be_indented() {
		// example 152
		test_raw(
			"   <!-- foo -->\n\n    <!-- foo -->",
			"   <!-- foo -->\n<pre><code>&lt;!-- foo --&gt;</code></pre>",
		);

		// example 153
		test_raw("   <div>\n\n    <div>", "   <div>\n<pre><code>&lt;div&gt;</code></pre>");
	}

	#[test]
	fn handles_paragraphs_and_blanks_correctly() {
		// example 154 - types 1-6 can interrupt paragraph
		test(
			r##"
				Foo
				<div>
				bar
				</div>
			"##,
			r##"
				<p>Foo</p>
				<div>
				bar
				</div>
			"##,
		);

		// example 155
		test(
			r##"
				<div>
				bar
				</div>
				*foo*
			"##,
			r##"
				<div>
				bar
				</div>
				*foo*
			"##,
		);

		// example 156 - type 7 cannot interrupt paragraph
		test(
			r##"
				Foo
				<a href="bar">
				baz
			"##,
			r##"
				<p>Foo
				<a href="bar">
				baz</p>
			"##,
		);

		// blank lines interrupt the block:

		// example 157
		test(
			r##"
				<div>

				*Emphasized* text.

				</div>
			"##,
			r##"
				<div>
				<p><em>Emphasized</em> text.</p>
				</div>
			"##,
		);

		// example 158
		test(
			r##"
				<div>
				*Emphasized* text.
				</div>
			"##,
			r##"
				<div>
				*Emphasized* text.
				</div>
			"##,
		);

		// example 159
		test(
			r##"
				<table>

				<tr>

				<td>
				Hi
				</td>

				</tr>

				</table>
			"##,
			r##"
				<table>
				<tr>
				<td>
				Hi
				</td>
				</tr>
				</table>
			"##,
		);

		// example 160 - blank lines and indentation
		test_raw(
			"<table>\n\n  <tr>\n\n    <td>\n      Hi\n    </td>\n\n  </tr>\n\n</table>",
			"<table>\n  <tr>\n<pre><code>&lt;td&gt;\n  Hi\n&lt;/td&gt;</code></pre>\n  </tr>\n</table>",
		);
	}
}
