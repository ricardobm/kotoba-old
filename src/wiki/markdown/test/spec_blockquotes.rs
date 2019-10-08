use super::*;

// spell-checker: disable

mod markdown_spec_blockquotes {
	use super::*;

	#[test]
	fn example_206_simple() {
		// example 206
		test(
			r##"
				> # Foo
				> bar
				> baz
			"##,
			r##"
				<blockquote>
				<h1>Foo</h1>
				<p>bar
				baz</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_207_spaces_after_marker_can_be_omitted() {
		// example 207
		test(
			r##"
				># Foo
				>bar
				> baz
			"##,
			r##"
				<blockquote>
				<h1>Foo</h1>
				<p>bar
				baz</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_208_can_be_indented() {
		// example 208
		test_raw(
			"   > # Foo\n   > bar\n > baz",
			"<blockquote>\n<h1>Foo</h1>\n<p>bar\nbaz</p>\n</blockquote>",
		);
	}

	#[test]
	fn example_209_four_spaces_is_a_code_block() {
		// example 209
		test_raw(
			"    > # Foo\n    > bar\n    > baz",
			"<pre><code>&gt; # Foo\n&gt; bar\n&gt; baz</code></pre>",
		);
	}

	#[test]
	fn example_210_allows_lazyness() {
		// example 210
		test(
			r##"
				> # Foo
				> bar
				baz
			"##,
			r##"
				<blockquote>
				<h1>Foo</h1>
				<p>bar
				baz</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_211_can_mix_lazy_non_lazy() {
		// example 211
		test(
			r##"
				> bar
				baz
				> foo
			"##,
			r##"
				<blockquote>
				<p>bar
				baz
				foo</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_212_lazyness_only_applies_to_continuations() {
		// example 212
		test(
			r##"
				> foo
				---
			"##,
			r##"
				<blockquote>
				<p>foo</p>
				</blockquote>
				<hr/>
			"##,
		);
	}

	#[test]
	fn example_213_lazyness_only_applies_to_continuations() {
		// example 213
		test(
			r##"
				> - foo
				- bar
			"##,
			r##"
				<blockquote>
				<ul>
				<li>foo</li>
				</ul>
				</blockquote>
				<ul>
				<li>bar</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_214_lazyness_and_indented_block() {
		// example 214
		test_raw(
			">     foo\n    bar",
			"<blockquote>\n<pre><code>foo</code></pre>\n</blockquote>\n<pre><code>bar</code></pre>",
		);
	}

	#[test]
	fn example_215_lazyness_and_fenced_block() {
		// example 215
		test(
			r##"
				> ```
				foo
				```
			"##,
			r##"
				<blockquote>
				<pre><code>
				</code></pre>
				</blockquote>
				<p>foo</p>
				<pre><code></code></pre>
			"##,
		);
	}

	#[test]
	fn example_216_lazy_continuation_line() {
		// example 216
		test_raw("> foo\n    - bar", "<blockquote>\n<p>foo\n- bar</p>\n</blockquote>");
	}

	#[test]
	fn example_217_can_be_empty() {
		// example 217
		test(
			r##"
				>
			"##,
			r##"
				<blockquote>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_218_can_be_empty() {
		// example 218
		test_raw(">\n>  \n> ", "<blockquote>\n</blockquote>");
	}

	#[test]
	fn example_219_can_have_initial_and_final_blank_lines() {
		// example 219
		test_raw(">\n> foo\n>  ", "<blockquote>\n<p>foo</p>\n</blockquote>");
	}

	#[test]
	fn example_220_are_separated_by_blank_lines() {
		// example 220
		test(
			r##"
				> foo

				> bar
			"##,
			r##"
				<blockquote>
				<p>foo</p>
				</blockquote>
				<blockquote>
				<p>bar</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_221_respect_consecutiveness() {
		// example 221
		test(
			r##"
				> foo
				> bar
			"##,
			r##"
				<blockquote>
				<p>foo
				bar</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_222_with_two_paragraphs() {
		// example 222
		test(
			r##"
				> foo
				>
				> bar
			"##,
			r##"
				<blockquote>
				<p>foo</p>
				<p>bar</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_223_can_interrupt_paragraphs() {
		// example 223
		test(
			r##"
				foo
				> bar
			"##,
			r##"
				<p>foo</p>
				<blockquote>
				<p>bar</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_224_blank_lines_not_needed() {
		// example 224
		test(
			r##"
				> aaa
				***
				> bbb
			"##,
			r##"
				<blockquote>
				<p>aaa</p>
				</blockquote>
				<hr/>
				<blockquote>
				<p>bbb</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_225_blockquote_and_new_paragraph_requires_blank() {
		// example 225
		test(
			r##"
				> bar
				baz
			"##,
			r##"
				<blockquote>
				<p>bar
				baz</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_226_blockquote_and_new_paragraph_requires_blank() {
		// example 226
		test(
			r##"
				> bar

				baz
			"##,
			r##"
				<blockquote>
				<p>bar</p>
				</blockquote>
				<p>baz</p>
			"##,
		);
	}

	#[test]
	fn example_227_blockquote_and_new_paragraph_requires_blank() {
		// example 227
		test(
			r##"
				> bar
				>
				baz
			"##,
			r##"
				<blockquote>
				<p>bar</p>
				</blockquote>
				<p>baz</p>
			"##,
		);
	}

	#[test]
	fn example_228_lazyness_can_omit_any_number_of_markers() {
		// example 228
		test(
			r##"
				> > > foo
				bar
			"##,
			r##"
				<blockquote>
				<blockquote>
				<blockquote>
				<p>foo
				bar</p>
				</blockquote>
				</blockquote>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_229_lazyness_can_omit_any_number_of_markers() {
		// example 229
		test(
			r##"
				>>> foo
				> bar
				>>baz
			"##,
			r##"
				<blockquote>
				<blockquote>
				<blockquote>
				<p>foo
				bar
				baz</p>
				</blockquote>
				</blockquote>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_230_need_five_spaces_for_code() {
		// example 230
		test(
			r##"
				>     code

				>    not code
			"##,
			r##"
				<blockquote>
				<pre><code>code</code></pre>
				</blockquote>
				<blockquote>
				<p>not code</p>
				</blockquote>
			"##,
		);
	}
}
