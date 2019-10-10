use super::*;

// spell-checker: disable

mod markdown_spec_inline_code {
	use super::*;

	#[test]
	fn example_338_simple() {
		// example 338
		test(
			r##"
				`foo`
			"##,
			r##"
				<p><code>foo</code></p>
			"##,
		);
	}

	#[test]
	fn example_339_two_backticks_and_trimming() {
		// example 339
		test(
			r##"
				`` foo ` bar ``
			"##,
			r##"
				<p><code>foo ` bar</code></p>
			"##,
		);
	}

	#[test]
	fn example_340_trimming() {
		// example 340
		test(
			r##"
				` `` `
			"##,
			r##"
				<p><code>``</code></p>
			"##,
		);
	}

	#[test]
	fn example_341_only_one_space_is_trimmed() {
		// example 341
		test(
			r##"
				`  ``  `
			"##,
			r##"
				<p><code> `` </code></p>
			"##,
		);
	}

	#[test]
	fn example_342_trimming_only_on_both_sides() {
		// example 342
		test(
			r##"
				` a`
			"##,
			r##"
				<p><code> a</code></p>
			"##,
		);
	}

	#[test]
	fn example_343_only_ascii_whitespace_is_trimmed() {
		// example 343
		test_raw("`\u{00A0}b\u{00A0}`", "<p><code>\u{00A0}b\u{00A0}</code></p>");
	}

	#[test]
	fn example_344_no_trimming_if_space_only() {
		// example 344
		test(
			r##"
				` `
				`  `
			"##,
			r##"
				<p><code> </code>
				<code>  </code></p>
			"##,
		);
	}

	#[test]
	fn example_345_line_endings_are_spaces() {
		// example 345
		test_raw_in(
			"``\nfoo\nbar  \nbaz\n``",
			r##"
				<p><code>foo bar   baz</code></p>
			"##,
		);

		test_raw_in(
			"``\rfoo\rbar  \rbaz\r``",
			r##"
				<p><code>foo bar   baz</code></p>
			"##,
		);

		test_raw_in(
			"``\r\nfoo\r\nbar  \r\nbaz\r\n``",
			r##"
				<p><code>foo bar   baz</code></p>
			"##,
		);
	}

	#[test]
	fn example_346_line_endings_are_spaces() {
		// example 346
		test_raw_in(
			"``\nfoo \n``",
			r##"
				<p><code>foo </code></p>
			"##,
		);
	}

	#[test]
	fn example_347_interior_spaces_not_collapsed() {
		// example 347
		test_raw_in(
			"`foo   bar \nbaz`",
			r##"
				<p><code>foo   bar  baz</code></p>
			"##,
		);
	}

	#[test]
	fn example_348_backslash_do_nothing() {
		// example 348
		test(
			r##"
				`foo\`bar`
			"##,
			r##"
				<p><code>foo\</code>bar`</p>
			"##,
		);
	}

	#[test]
	fn example_349_more_ticks() {
		// example 349
		test(
			r##"
				``foo`bar``
			"##,
			r##"
				<p><code>foo`bar</code></p>
			"##,
		);
	}

	#[test]
	fn example_350_less_ticks() {
		// example 350
		test(
			r##"
				` foo `` bar `
			"##,
			r##"
				<p><code>foo `` bar</code></p>
			"##,
		);
	}

	#[test]
	fn example_351_code_is_higher_precedence_than_emphasis() {
		// example 351
		test(
			r##"
				*foo`*`
			"##,
			r##"
				<p>*foo<code>*</code></p>
			"##,
		);
	}

	#[test]
	fn example_352_code_is_higher_precedence_than_links() {
		// example 352
		test(
			r##"
				[not a `link](/foo`)
			"##,
			r##"
				<p>[not a <code>link](/foo</code>)</p>
			"##,
		);
	}

	#[test]
	fn example_353_same_precedence_as_html() {
		// example 353
		test(
			r##"
				`<a href="`">`
			"##,
			r##"
				<p><code>&lt;a href=&quot;</code>&quot;&gt;`</p>
			"##,
		);
	}

	#[test]
	fn example_354_same_precedence_as_html() {
		// example 354
		test(
			r##"
				<a href="`">`
			"##,
			r##"
				<p><a href="`">`</p>
			"##,
		);
	}

	#[test]
	fn example_355_same_precedence_as_autolink() {
		// example 355
		test(
			r##"
				`<http://foo.bar.`baz>`
			"##,
			r##"
				<p><code>&lt;http://foo.bar.</code>baz&gt;`</p>
			"##,
		);
	}

	#[test]
	fn example_356_same_precedence_as_autolink() {
		// example 356
		test(
			r##"
				<http://foo.bar.`baz>`
			"##,
			r##"
				<p><a href="http://foo.bar.`baz">http://foo.bar.`baz</a>`</p>
			"##,
		);
	}

	#[test]
	fn example_357_must_be_properly_closed() {
		// example 357
		test(
			r##"
				```foo``
			"##,
			r##"
				<p>```foo``</p>
			"##,
		);
	}

	#[test]
	fn example_358_must_be_properly_closed() {
		// example 358
		test(
			r##"
				`foo
			"##,
			r##"
				<p>`foo</p>
			"##,
		);
	}

	#[test]
	fn example_359_delims_must_be_equal_in_length() {
		// example 359
		test(
			r##"
				`foo``bar``
			"##,
			r##"
				<p>`foo<code>bar</code></p>
			"##,
		);
	}
}
