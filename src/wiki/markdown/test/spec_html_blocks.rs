use super::*;

// spell-checker: disable

mod markdown_spec_html_blocks {
	use super::*;

	#[test]
	fn example_118() {
		// example 118
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
	}

	#[test]
	fn example_119() {
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
	}

	#[test]
	fn example_120() {
		// example 120
		test_raw(
			" <div>\n  *hello*\n         <foo><a>",
			" <div>\n  *hello*\n         <foo><a>",
		);
	}

	#[test]
	fn example_121() {
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
	}

	#[test]
	fn example_122() {
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
	}

	#[test]
	fn example_123() {
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
	}

	#[test]
	fn example_124() {
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
	}

	#[test]
	fn example_125() {
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
	}

	#[test]
	fn example_126() {
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
	}

	#[test]
	fn example_127() {
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
	}

	#[test]
	fn example_128() {
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
	}

	#[test]
	fn example_129() {
		// example 129
		test(
			r##"
				<div><a href="bar">*foo*</a></div>
			"##,
			r##"
				<div><a href="bar">*foo*</a></div>
			"##,
		);
	}

	#[test]
	fn example_130() {
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
	}

	#[test]
	fn example_131() {
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
	fn example_132_should_parse_type_7() {
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
	}

	#[test]
	fn example_133_should_parse_type_7() {
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
	}

	#[test]
	fn example_134_should_parse_type_7() {
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
	}

	#[test]
	fn example_135_should_parse_type_7() {
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
	}

	#[test]
	fn example_136_should_parse_type_7() {
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
	}

	#[test]
	fn example_137_should_parse_type_7() {
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
	}

	#[test]
	fn example_138_should_parse_type_7() {
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
	fn example_139_should_parse_type_1() {
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
	}

	#[test]
	fn example_140_should_parse_type_1() {
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
	}

	#[test]
	fn example_141_should_parse_type_1() {
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
	fn example_142_should_handle_unclosed_type_1() {
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
	}

	#[test]
	fn example_143_should_handle_unclosed_type_1() {
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
	}

	#[test]
	fn example_144_should_handle_unclosed_type_1() {
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
	fn example_145_type_1_end_tag_can_occur_on_the_same_line() {
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
	}

	#[test]
	fn example_146_type_1_end_tag_can_occur_on_the_same_line() {
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
	fn example_147_includes_anything_after_end_tag() {
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
	fn example_148_supports_type_2_comments() {
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
	fn example_149_supports_type_3_processing_instructions() {
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
	fn example_150_supports_type_4_declarations() {
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
	fn example_151_supports_type_5_cdata() {
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
	fn example_152_can_be_indented() {
		// example 152
		test_raw(
			"   <!-- foo -->\n\n    <!-- foo -->",
			"   <!-- foo -->\n<pre><code>&lt;!-- foo --&gt;</code></pre>",
		);
	}

	#[test]
	fn example_153_can_be_indented() {
		// example 153
		test_raw("   <div>\n\n    <div>", "   <div>\n<pre><code>&lt;div&gt;</code></pre>");
	}

	#[test]
	fn example_154_handles_paragraphs_and_blanks_correctly() {
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
	}

	#[test]
	fn example_155_handles_paragraphs_and_blanks_correctly() {
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
	}

	#[test]
	fn example_156_handles_paragraphs_and_blanks_correctly() {
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
	}

	#[test]
	fn example_157_handles_paragraphs_and_blanks_correctly() {
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
	}

	#[test]
	fn example_158_handles_paragraphs_and_blanks_correctly() {
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
	}

	#[test]
	fn example_159_handles_paragraphs_and_blanks_correctly() {
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
	}

	#[test]
	fn example_160_handles_paragraphs_and_blanks_correctly() {
		// example 160 - blank lines and indentation
		test_raw(
			"<table>\n\n  <tr>\n\n    <td>\n      Hi\n    </td>\n\n  </tr>\n\n</table>",
			"<table>\n  <tr>\n<pre><code>&lt;td&gt;\n  Hi\n&lt;/td&gt;</code></pre>\n  </tr>\n</table>",
		);
	}
}
