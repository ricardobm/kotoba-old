use super::*;

// spell-checker: disable

mod markdown_spec_list_items {
	use super::*;

	#[test]
	fn example_231_non_list_sample() {
		// example 231
		test(
			r##"
				A paragraph
				with two lines.

					indented code

				> A block quote.
			"##,
			r##"
				<p>A paragraph
				with two lines.</p>
				<pre><code>indented code</code></pre>
				<blockquote>
				<p>A block quote.</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_232_list_sample() {
		// example 232
		test_raw_in(
			"1.  A paragraph\n    with two lines.\n\n        indented code\n\n    > A block quote.",
			r#"
				<ol>
				<li>
				<p>A paragraph
				with two lines.</p>
				<pre><code>indented code</code></pre>
				<blockquote>
				<p>A block quote.</p>
				</blockquote>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_233_list_content_indent_out() {
		// example 233
		test_raw_in(
			"- one\n\n two",
			r#"
				<ul>
				<li>one</li>
				</ul>
				<p>two</p>
			"#,
		);
	}

	#[test]
	fn example_234_list_content_indent_in() {
		// example 234
		test_raw_in(
			"- one\n\n  two",
			r#"
				<ul>
				<li>
				<p>one</p>
				<p>two</p>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_235_list_content_text_indent_out() {
		// example 235
		test_raw_in(
			" -    one\n\n     two",
			r#"
				<ul>
				<li>one</li>
				</ul>
				<pre><code> two</code></pre>
			"#,
		);
	}

	#[test]
	fn example_236_list_content_text_indent_in() {
		// example 236
		test_raw_in(
			" -    one\n\n      two",
			r#"
				<ul>
				<li>
				<p>one</p>
				<p>two</p>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_237_embedded_indentation_in() {
		// example 237
		test_raw_in(
			"   > > 1.  one\n>>\n>>     two",
			r#"
				<blockquote>
				<blockquote>
				<ol>
				<li>
				<p>one</p>
				<p>two</p>
				</li>
				</ol>
				</blockquote>
				</blockquote>
			"#,
		);
	}

	#[test]
	fn example_238_embedded_indentation_out() {
		// example 238
		test_raw_in(
			">>- one\n>>\n  >  > two",
			r#"
				<blockquote>
				<blockquote>
				<ul>
				<li>one</li>
				</ul>
				<p>two</p>
				</blockquote>
				</blockquote>
			"#,
		);
	}

	#[test]
	fn example_239_needs_space_after_marker() {
		// example 239
		test_raw_in(
			"-one\n\n2.two",
			r#"
				<p>-one</p>
				<p>2.two</p>
			"#,
		);
	}

	#[test]
	fn example_240_child_blocks_can_be_separated() {
		// example 240
		test_raw_in(
			"- foo\n\n\n  bar",
			r#"
				<ul>
				<li>
				<p>foo</p>
				<p>bar</p>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_241_supports_any_block() {
		// example 241
		test_raw_in(
			"1.  foo\n\n    ```\n    bar\n    ```\n\n    baz\n\n    > bam",
			r#"
				<ol>
				<li>
				<p>foo</p>
				<pre><code>bar
				</code></pre>
				<p>baz</p>
				<blockquote>
				<p>bam</p>
				</blockquote>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_242_child_indented_code_preserves_new_lines() {
		// example 242
		test_raw_in(
			"- Foo\n\n      bar\n\n\n      baz",
			r#"
				<ul>
				<li>
				<p>Foo</p>
				<pre><code>bar


				baz</code></pre>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_243_ordered_digits_max_length_ok() {
		// example 243
		test_raw_in(
			"123456789. ok",
			r#"
				<ol start="123456789">
				<li>ok</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_244_ordered_digits_max_length_not_ok() {
		// example 244
		test_raw_in(
			"1234567890. not ok",
			r#"
				<p>1234567890. not ok</p>
			"#,
		);
	}

	#[test]
	fn example_245_ordered_may_start_with_zero() {
		// example 245
		test_raw_in(
			"0. ok",
			r#"
				<ol start="0">
				<li>ok</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_246_ordered_left_zeroes() {
		// example 246
		test_raw_in(
			"003. ok",
			r#"
				<ol start="3">
				<li>ok</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_247_ordered_cannot_be_negative() {
		// example 247
		test_raw_in(
			"-1. not ok",
			r#"
				<p>-1. not ok</p>
			"#,
		);
	}

	#[test]
	fn example_248_indented_code_inside_list() {
		// example 248
		test_raw_in(
			"- foo\n\n      bar",
			r#"
				<ul>
				<li>
				<p>foo</p>
				<pre><code>bar</code></pre>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_249_indented_code_inside_list() {
		// example 249
		test_raw_in(
			"  10.  foo\n\n           bar",
			r#"
				<ol start="10">
				<li>
				<p>foo</p>
				<pre><code>bar</code></pre>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_250_indented_code_first_child() {
		// example 250
		test_raw_in(
			"    indented code\n\nparagraph\n\n    more code",
			r#"
				<pre><code>indented code</code></pre>
				<p>paragraph</p>
				<pre><code>more code</code></pre>
			"#,
		);
	}

	#[test]
	fn example_251_indented_code_continuation() {
		// example 251
		test_raw_in(
			"1.     indented code\n\n   paragraph\n\n       more code",
			r#"
				<ol>
				<li>
				<pre><code>indented code</code></pre>
				<p>paragraph</p>
				<pre><code>more code</code></pre>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_252_indented_code_additional_space() {
		// example 252
		test_raw_in(
			"1.      indented code\n\n   paragraph\n\n       more code",
			r#"
				<ol>
				<li>
				<pre><code> indented code</code></pre>
				<p>paragraph</p>
				<pre><code>more code</code></pre>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_253_base_indentation() {
		// example 253
		test_raw_in(
			"   foo\n\nbar",
			r#"
				<p>foo</p>
				<p>bar</p>
			"#,
		);
	}

	#[test]
	fn example_254_base_indentation() {
		// example 254
		test_raw_in(
			"-    foo\n\n  bar",
			r#"
				<ul>
				<li>foo</li>
				</ul>
				<p>bar</p>
			"#,
		);
	}

	#[test]
	fn example_255_base_indentation() {
		// example 255
		test_raw_in(
			"-  foo\n\n   bar",
			r#"
				<ul>
				<li>
				<p>foo</p>
				<p>bar</p>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_256_starting_with_blank_line() {
		// example 256
		test_raw_in(
			"-\n  foo\n-\n  ```\n  bar\n  ```\n-\n      baz",
			r#"
				<ul>
				<li>foo</li>
				<li>
				<pre><code>bar
				</code></pre>
				</li>
				<li>
				<pre><code>baz</code></pre>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_257_blank_line_spaces_dont_change_indentation() {
		// example 257
		test_raw_in(
			"-   \n  foo",
			r#"
				<ul>
				<li>foo</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_258_empty_list_item_can_begin_with_at_most_one_line() {
		// example 258
		test_raw_in(
			"-\n\n  foo",
			r#"
				<ul>
				<li></li>
				</ul>
				<p>foo</p>
			"#,
		);
	}

	#[test]
	fn example_259_empty_bullet_item() {
		// example 259
		test_raw_in(
			"- foo\n-\n- bar",
			r#"
				<ul>
				<li>foo</li>
				<li></li>
				<li>bar</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_260_empty_bullet_item_with_spaces() {
		// example 260
		test_raw_in(
			"- foo\n-   \n- bar",
			r#"
				<ul>
				<li>foo</li>
				<li></li>
				<li>bar</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_261_empty_ordered_item() {
		// example 261
		test_raw_in(
			"1. foo\n2.\n3. bar",
			r#"
				<ol>
				<li>foo</li>
				<li></li>
				<li>bar</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_262_list_item_can_be_empty() {
		// example 262
		test_raw_in(
			"*",
			r#"
				<ul>
				<li></li>
				</ul>
			"#,
		);

		test_raw_in(
			"1.",
			r#"
				<ol>
				<li></li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_263_empty_list_item_cannot_interrupt_paragraph() {
		// example 263
		test_raw_in(
			"foo\n*\n\nfoo\n1.",
			r#"
				<p>foo
				*</p>
				<p>foo
				1.</p>
			"#,
		);
	}

	#[test]
	fn example_264_indented_one_space() {
		// example 264
		test_raw_in(
			" 1.  A paragraph\n     with two lines.\n\n         indented code\n\n     > A block quote.",
			r#"
				<ol>
				<li>
				<p>A paragraph
				with two lines.</p>
				<pre><code>indented code</code></pre>
				<blockquote>
				<p>A block quote.</p>
				</blockquote>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_265_indented_two_spaces() {
		// example 265
		test_raw_in(
			"  1.  A paragraph\n      with two lines.\n\n          indented code\n\n      > A block quote.",
			r#"
				<ol>
				<li>
				<p>A paragraph
				with two lines.</p>
				<pre><code>indented code</code></pre>
				<blockquote>
				<p>A block quote.</p>
				</blockquote>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_266_indented_three_spaces() {
		// example 266
		test_raw_in(
			"   1.  A paragraph\n       with two lines.\n\n           indented code\n\n       > A block quote.",
			r#"
				<ol>
				<li>
				<p>A paragraph
				with two lines.</p>
				<pre><code>indented code</code></pre>
				<blockquote>
				<p>A block quote.</p>
				</blockquote>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_267_four_spaces_gives_a_code_block() {
		// example 267
		test_raw(
			"    1.  A paragraph\n        with two lines.\n\n            indented code\n\n        > A block quote.",
			"<pre><code>1.  A paragraph\n    with two lines.\n\n        indented code\n\n    &gt; A block quote.</code></pre>",
		);
	}

	#[test]
	fn example_268_lazy_continuation() {
		// example 268
		test_raw_in(
			"  1.  A paragraph\nwith two lines.\n\n          indented code\n\n      > A block quote.",
			r#"
				<ol>
				<li>
				<p>A paragraph
				with two lines.</p>
				<pre><code>indented code</code></pre>
				<blockquote>
				<p>A block quote.</p>
				</blockquote>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_269_lazy_continuation_with_partially_deleted_indentation() {
		// example 269
		test_raw_in(
			"  1.  A paragraph\n    with two lines.",
			r#"
				<ol>
				<li>A paragraph
				with two lines.</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_270_nested_lazyness() {
		// example 270
		test_raw_in(
			"> 1. > Blockquote\ncontinued here.",
			r#"
				<blockquote>
				<ol>
				<li>
				<blockquote>
				<p>Blockquote
				continued here.</p>
				</blockquote>
				</li>
				</ol>
				</blockquote>
			"#,
		);
	}

	#[test]
	fn example_271_nested_lazyness() {
		// example 271
		test_raw_in(
			"> 1. > Blockquote\n> continued here.",
			r#"
				<blockquote>
				<ol>
				<li>
				<blockquote>
				<p>Blockquote
				continued here.</p>
				</blockquote>
				</li>
				</ol>
				</blockquote>
			"#,
		);
	}

	#[test]
	fn example_272_sublist_must_be_indented_by_two() {
		// example 272
		test_raw_in(
			"- foo\n  - bar\n    - baz\n      - boo",
			r#"
				<ul>
				<li>foo
				<ul>
				<li>bar
				<ul>
				<li>baz
				<ul>
				<li>boo</li>
				</ul>
				</li>
				</ul>
				</li>
				</ul>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_273_sublist_indent_by_one_is_not_enough() {
		// example 273
		test_raw_in(
			"- foo\n - bar\n  - baz\n   - boo",
			r#"
				<ul>
				<li>foo</li>
				<li>bar</li>
				<li>baz</li>
				<li>boo</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_274_list_marker_width_affects_sublist_indent() {
		// example 274
		test_raw_in(
			"10) foo\n    - bar",
			r#"
				<ol start="10">
				<li>foo
				<ul>
				<li>bar</li>
				</ul>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_275_list_marker_width_affects_sublist_indent() {
		// example 275
		test_raw_in(
			"10) foo\n   - bar",
			r#"
				<ol start="10">
				<li>foo</li>
				</ol>
				<ul>
				<li>bar</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_276_list_can_be_first_item_in_an_item() {
		// example 276
		test_raw_in(
			"- - foo",
			r#"
				<ul>
				<li>
				<ul>
				<li>foo</li>
				</ul>
				</li>
				</ul>
			"#,
		);
	}

	#[test]
	fn example_277_list_can_be_first_item_in_an_item() {
		// example 277
		test_raw_in(
			"1. - 2. foo",
			r#"
				<ol>
				<li>
				<ul>
				<li>
				<ol start="2">
				<li>foo</li>
				</ol>
				</li>
				</ul>
				</li>
				</ol>
			"#,
		);
	}

	#[test]
	fn example_278_list_item_can_contain_heading() {
		// example 278
		test_raw_in(
			"- # Foo\n- Bar\n  ---\n  baz",
			r#"
				<ul>
				<li>
				<h1>Foo</h1>
				</li>
				<li>
				<h2>Bar</h2>
				baz</li>
				</ul>
			"#,
		);
	}
}
