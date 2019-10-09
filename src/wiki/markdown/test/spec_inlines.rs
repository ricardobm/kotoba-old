use super::*;

// spell-checker: disable

mod markdown_spec_inlines {
	use super::*;

	#[test]
	fn example_307_are_parsed_left_to_right() {
		// example 307
		test(
			r##"
				`hi`lo`
			"##,
			r##"
				<p><code>hi</code>lo`</p>
			"##,
		);
	}

	#[test]
	fn example_308_backslash_escapes() {
		// example 308
		test(
			r##"
				\!\"\#\$\%\&\'\(\)\*\+\,\-\.\/\:\;\<\=\>\?\@\[\\\]\^\_\`\{\|\}\~
			"##,
			r##"
				<p>!&quot;#$%&amp;&apos;()*+,-./:;&lt;=&gt;?@[\]^_`{|}~</p>
			"##,
		);
	}

	#[test]
	fn example_309_unrecognized_backslash_escapes_are_literal() {
		// example 309
		test(
			r##"
				\→\A\a\ \3\φ\«
			"##,
			r##"
				<p>\→\A\a\ \3\φ\«</p>
			"##,
		);
	}

	#[test]
	fn example_310_escaped_characters_are_treated_as_regular() {
		// example 310
		test(
			r##"
				\*not emphasized*
				\<br/> not a tag
				\[not a link](/foo)
				\`not code`
				1\. not a list
				\* not a list
				\# not a heading
				\[foo]: /url "not a reference"
				\&ouml; not a character entity
			"##,
			r##"
				<p>*not emphasized*
				&lt;br/&gt; not a tag
				[not a link](/foo)
				`not code`
				1. not a list
				* not a list
				# not a heading
				[foo]: /url &quot;not a reference&quot;
				&amp;ouml; not a character entity</p>
			"##,
		);
	}

	#[test]
	fn example_311_escaped_backslash() {
		// example 311
		test(
			r##"
				\\*emphasis*
			"##,
			r##"
				<p>\<em>emphasis</em></p>
			"##,
		);
	}

	#[test]
	fn example_312_backslash_hard_break() {
		// example 312
		test(
			r##"
				foo\
				bar
			"##,
			r##"
				<p>foo<br/>
				bar</p>
			"##,
		);
	}

	#[test]
	fn example_313_backslashes_do_not_work_in_code_blocks() {
		// example 313
		test(
			r##"
				`` \[\` ``
			"##,
			r##"
				<p><code>\[\`</code></p>
			"##,
		);
	}

	#[test]
	fn example_314_backslashes_do_not_work_in_indented_code() {
		// example 314
		test_raw_in(
			r##"    \[\]"##,
			r##"
				<pre><code>\[\]</code></pre>
			"##,
		);
	}

	#[test]
	fn example_315_backslashes_do_not_work_in_code_blocks() {
		// example 315
		test(
			r##"
				~~~
				\[\]
				~~~
			"##,
			r##"
				<pre><code>\[\]
				</code></pre>
			"##,
		);
	}

	#[test]
	fn example_316_backslashes_do_not_work_in_autolinks() {
		// example 316
		test(
			r##"
				<http://example.com?find=\*>
			"##,
			r##"
				<p><a href="http://example.com?find=\*">http://example.com?find=\*</a></p>
			"##,
		);
	}

	#[test]
	fn example_317_backslashes_do_not_work_in_raw_html() {
		// example 317
		test(
			r##"
				<a href="/bar\/)">
			"##,
			r##"
				<a href="/bar\/)">
			"##,
		);
	}

	#[test]
	fn example_318_backslashes_work_in_links() {
		// example 318
		test(
			r##"
				[foo](/bar\* "ti\*tle")
			"##,
			r##"
				<p><a href="/bar*" title="ti*tle">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_319_backslashes_work_in_link_references() {
		// example 319
		test(
			r##"
				[foo]

				[foo]: /bar\* "ti\*tle"
			"##,
			r##"
				<p><a href="/bar*" title="ti*tle">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_320_backslashes_work_in_info_strings() {
		// example 320
		test(
			r##"
				``` foo\+bar
				foo
				```
			"##,
			r##"
				<pre><code data-info="foo+bar">foo
				</code></pre>
			"##,
		);
	}
}
