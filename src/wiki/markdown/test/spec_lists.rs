use super::*;

// spell-checker: disable

mod markdown_spec_lists {
	use super::*;

	#[test]
	fn example_281_bullet_list() {
		// example 281
		test(
			r##"
				- foo
				- bar
				+ baz
			"##,
			r##"
				<ul>
				<li>foo</li>
				<li>bar</li>
				</ul>
				<ul>
				<li>baz</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_282_ordered_list() {
		// example 282
		test(
			r##"
				1. foo
				2. bar
				3) baz
			"##,
			r##"
				<ol>
				<li>foo</li>
				<li>bar</li>
				</ol>
				<ol start="3">
				<li>baz</li>
				</ol>
			"##,
		);
	}

	#[test]
	fn example_283_can_interrupt_a_paragraph() {
		// example 283
		test(
			r##"
				Foo
				- bar
				- baz
			"##,
			r##"
				<p>Foo</p>
				<ul>
				<li>bar</li>
				<li>baz</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_284_can_only_interrupt_a_paragraph_starting_with_one() {
		// example 284
		test(
			r##"
				The number of windows in my house is
				14.  The number of doors is 6.
			"##,
			r##"
				<p>The number of windows in my house is
				14.  The number of doors is 6.</p>
			"##,
		);
	}

	#[test]
	fn example_285_can_only_interrupt_a_paragraph_starting_with_one() {
		// example 285
		test(
			r##"
				The number of windows in my house is
				1.  The number of doors is 6.
			"##,
			r##"
				<p>The number of windows in my house is</p>
				<ol>
				<li>The number of doors is 6.</li>
				</ol>
			"##,
		);
	}

	#[test]
	fn example_286_can_have_blank_lines_between_items() {
		// example 286
		test(
			r##"
				- foo

				- bar


				- baz
			"##,
			r##"
				<ul>
				<li>
				<p>foo</p>
				</li>
				<li>
				<p>bar</p>
				</li>
				<li>
				<p>baz</p>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_287_can_have_blank_lines_between_items() {
		// example 287
		test_raw_in(
			"- foo\n  - bar\n    - baz\n\n\n      bim",
			r##"
				<ul>
				<li>foo
				<ul>
				<li>bar
				<ul>
				<li>
				<p>baz</p>
				<p>bim</p>
				</li>
				</ul>
				</li>
				</ul>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_288_using_a_comment_to_separate_lists() {
		// example 288
		test(
			r##"
				- foo
				- bar

				<!-- -->

				- baz
				- bim
			"##,
			r##"
				<ul>
				<li>foo</li>
				<li>bar</li>
				</ul>
				<!-- -->
				<ul>
				<li>baz</li>
				<li>bim</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_289_using_a_comment_to_separate_lists() {
		// example 289
		test_raw_in(
			"-   foo\n\n    notcode\n\n-   foo\n\n<!-- -->\n\n    code",
			r##"
			<ul>
			<li>
			<p>foo</p>
			<p>notcode</p>
			</li>
			<li>
			<p>foo</p>
			</li>
			</ul>
			<!-- -->
			<pre><code>code</code></pre>
			"##,
		);
	}

	#[test]
	fn example_290_does_not_require_same_indent_level() {
		// example 290
		test_raw_in(
			"- a\n - b\n  - c\n   - d\n  - e\n - f\n- g",
			r##"
				<ul>
				<li>a</li>
				<li>b</li>
				<li>c</li>
				<li>d</li>
				<li>e</li>
				<li>f</li>
				<li>g</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_291_does_not_require_same_indent_level() {
		// example 291
		test_raw_in(
			"1. a\n\n  2. b\n\n   3. c",
			r##"
				<ol>
				<li>
				<p>a</p>
				</li>
				<li>
				<p>b</p>
				</li>
				<li>
				<p>c</p>
				</li>
				</ol>
			"##,
		);
	}

	#[test]
	fn example_292_four_indent_as_a_paragraph_continuation() {
		// example 292
		test_raw_in(
			"- a\n - b\n  - c\n   - d\n    - e",
			r##"
				<ul>
				<li>a</li>
				<li>b</li>
				<li>c</li>
				<li>d
				- e</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_293_four_indent_as_code_block() {
		// example 293
		test_raw_in(
			"1. a\n\n  2. b\n\n    3. c",
			r##"
				<ol>
				<li>
				<p>a</p>
				</li>
				<li>
				<p>b</p>
				</li>
				</ol>
				<pre><code>3. c</code></pre>
			"##,
		);
	}

	#[test]
	fn example_294_loose_list() {
		// example 294
		test(
			r##"
				- a
				- b

				- c
			"##,
			r##"
				<ul>
				<li>
				<p>a</p>
				</li>
				<li>
				<p>b</p>
				</li>
				<li>
				<p>c</p>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_295_loose_list() {
		// example 295
		test(
			r##"
				* a
				*

				* c
			"##,
			r##"
				<ul>
				<li>
				<p>a</p>
				</li>
				<li></li>
				<li>
				<p>c</p>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_296_loose_list_with_block_spacing() {
		// example 296
		test(
			r##"
				- a
				- b

				  c
				- d
			"##,
			r##"
				<ul>
				<li>
				<p>a</p>
				</li>
				<li>
				<p>b</p>
				<p>c</p>
				</li>
				<li>
				<p>d</p>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_297_loose_list_with_block_spacing() {
		// example 297
		test(
			r##"
				- a
				- b

				  [ref]: /url
				- d
			"##,
			r##"
				<ul>
				<li>
				<p>a</p>
				</li>
				<li>
				<p>b</p>
				</li>
				<li>
				<p>d</p>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_298_tight_list_with_blanks_in_code_block() {
		// example 298
		test(
			r##"
				- a
				- ```
				  b


				  ```
				- c
			"##,
			r##"
				<ul>
				<li>a</li>
				<li>
				<pre><code>b


				</code></pre>
				</li>
				<li>c</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_299_tight_list_with_blanks_in_sublist() {
		// example 299
		test_raw_in(
			"- a\n  - b\n\n    c\n- d",
			r##"
				<ul>
				<li>a
				<ul>
				<li>
				<p>b</p>
				<p>c</p>
				</li>
				</ul>
				</li>
				<li>d</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_300_tight_list_with_blank_in_bloquote() {
		// example 300
		test_raw_in(
			"* a\n  > b\n  >\n* c",
			r##"
				<ul>
				<li>a
				<blockquote>
				<p>b</p>
				</blockquote>
				</li>
				<li>c</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_301_tight_list_with_consecutive_block_elements() {
		// example 301
		test_raw_in(
			"- a\n  > b\n  ```\n  c\n  ```\n- d",
			r##"
				<ul>
				<li>a
				<blockquote>
				<p>b</p>
				</blockquote>
				<pre><code>c
				</code></pre>
				</li>
				<li>d</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_302_single_paragraph_list_is_tight() {
		// example 302
		test(
			r##"
				- a
			"##,
			r##"
				<ul>
				<li>a</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_303_single_paragraph_lists_are_tight() {
		// example 303
		test(
			r##"
				- a
				  - b
			"##,
			r##"
				<ul>
				<li>a
				<ul>
				<li>b</li>
				</ul>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_304_single_item_loose_list() {
		// example 304
		test_raw_in(
			"1. ```\n   foo\n   ```\n\n   bar",
			r##"
				<ol>
				<li>
				<pre><code>foo
				</code></pre>
				<p>bar</p>
				</li>
				</ol>
			"##,
		);
	}

	#[test]
	fn example_305_outer_loose_inner_tight() {
		// example 305
		test_raw_in(
			"* foo\n  * bar\n\n  baz",
			r##"
				<ul>
				<li>
				<p>foo</p>
				<ul>
				<li>bar</li>
				</ul>
				<p>baz</p>
				</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_306_outer_loose_inner_tight() {
		// example 306
		test_raw_in(
			"- a\n  - b\n  - c\n\n- d\n  - e\n  - f",
			r##"
				<ul>
				<li>
				<p>a</p>
				<ul>
				<li>b</li>
				<li>c</li>
				</ul>
				</li>
				<li>
				<p>d</p>
				<ul>
				<li>e</li>
				<li>f</li>
				</ul>
				</li>
				</ul>
			"##,
		);
	}
}
