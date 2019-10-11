use super::*;

// spell-checker: disable

mod markdown_spec_thematic_breaks {
	use super::*;

	#[test]
	fn example_13_should_parse() {
		// example 13
		test(
			r##"
				***
				---
				___
			"##,
			r##"
				<hr/>
				<hr/>
				<hr/>
			"##,
		);
	}

	#[test]
	fn example_14_wrong_characters() {
		// example 14
		test(
			r##"
				+++
			"##,
			r##"
				<p>+++</p>
			"##,
		);
	}

	#[test]
	fn example_15_wrong_characters() {
		// example 15
		test(
			r##"
				===
			"##,
			r##"
				<p>===</p>
			"##,
		);
	}

	#[test]
	fn example_16_not_enough_characters() {
		// example 16
		test(
			r##"
				--
				**
				__
			"##,
			r##"
				<p>--
				**
				__</p>
			"##,
		);
	}

	#[test]
	fn example_17_allows_one_to_three_spaces() {
		// example 17
		test_raw(" ***\n  ***\n   ***", "<hr/>\n<hr/>\n<hr/>");
		test_raw(" ---\n  ---\n   ---", "<hr/>\n<hr/>\n<hr/>");
		test_raw(" ___\n  ___\n   ___", "<hr/>\n<hr/>\n<hr/>");
	}

	#[test]
	fn example_18_four_spaces_is_too_many() {
		// example 18
		test_raw("    ***", "<pre><code>***</code></pre>");
		test_raw("    ---", "<pre><code>---</code></pre>");
		test_raw("    ___", "<pre><code>___</code></pre>");

		test_raw("\t***", "<pre><code>***</code></pre>");
		test_raw("\t---", "<pre><code>---</code></pre>");
		test_raw("\t___", "<pre><code>___</code></pre>");
	}

	#[test]
	fn example_19_four_spaces_is_too_many() {
		// example 19
		test_raw("Foo 1\n    ***", "<p>Foo 1\n***</p>");
		test_raw("Foo 2\n    ---", "<p>Foo 2\n---</p>");
		test_raw("Foo 3\n    ___", "<p>Foo 3\n___</p>");

		test_raw("Bar 1\n\t***", "<p>Bar 1\n***</p>");
		test_raw("Bar 2\n\t---", "<p>Bar 2\n---</p>");
		test_raw("Bar 3\n\t___", "<p>Bar 3\n___</p>");

		test_raw("Baz 1\n   ***", "<p>Baz 1</p>\n<hr/>");
		test_raw("Baz 2\n   - - -", "<p>Baz 2</p>\n<hr/>");
		test_raw("Baz 3\n   ___", "<p>Baz 3</p>\n<hr/>");
	}

	#[test]
	fn example_20_allows_more_than_three_characters() {
		// example 20
		test("_____________________________________", "<hr/>");
		test("-------------------------------------", "<hr/>");
		test("*************************************", "<hr/>");
	}

	#[test]
	fn example_21_allows_spaces_and_only_spaces() {
		// example 21
		test(" - - -", "<hr/>");
	}

	#[test]
	fn example_22_allows_spaces_and_only_spaces() {
		// example 22
		test(" **  * ** * ** * **", "<hr/>");
	}

	#[test]
	fn example_23_allows_spaces_and_only_spaces() {
		// example 23
		test("-     -      -      -", "<hr/>");
	}

	#[test]
	fn example_24_example_end_spaces_are_allowed_at_the_end() {
		// example 24
		test("- - - -    ", "<hr/>");
	}

	#[test]
	fn example_25_no_other_characters_may_occur() {
		// example 25
		test(
			r##"
				_ _ _ _ a

				a------

				---a---
			"##,
			r##"
				<p>_ _ _ _ a</p>
				<p>a------</p>
				<p>---a---</p>
			"##,
		);
	}

	#[test]
	fn example_26_requires_non_space_characters_to_be_the_same() {
		// example 26
		test(" *-*", "<p><em>-</em></p>");
	}

	#[test]
	fn example_27_should_not_require_blank_lines() {
		// example 27
		test(
			r##"
				- foo
				***
				- bar
			"##,
			r##"
				<ul>
				<li>foo</li>
				</ul>
				<hr/>
				<ul>
				<li>bar</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_28_can_interrupt_a_paragraph() {
		// example 28
		test(
			r##"
				Foo 1
				***
				bar
			"##,
			r##"
				<p>Foo 1</p>
				<hr/>
				<p>bar</p>
			"##,
		);

		test(
			r##"
				Foo 2
				___
				bar
			"##,
			r##"
				<p>Foo 2</p>
				<hr/>
				<p>bar</p>
			"##,
		);

		test(
			r##"
				Foo 3
				- - -
				bar
			"##,
			r##"
				<p>Foo 3</p>
				<hr/>
				<p>bar</p>
			"##,
		);
	}

	#[test]
	fn example_29_has_lower_precedence_than_setext_heading() {
		// example 29
		test(
			r##"
				Foo
				---
				bar
			"##,
			r##"
				<h2>Foo</h2>
				<p>bar</p>
			"##,
		);
	}

	#[test]
	fn example_30_has_higher_precedence_than_lists() {
		// example 30
		test(
			r##"
				* Foo
				* * *
				* Bar
			"##,
			r##"
				<ul>
				<li>Foo</li>
				</ul>
				<hr/>
				<ul>
				<li>Bar</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_31_has_higher_precedence_than_lists() {
		// example 31
		test(
			r##"
				- Foo
				- * * *
			"##,
			r##"
				<ul>
				<li>Foo</li>
				<li>
				<hr/>
				</li>
				</ul>
			"##,
		);
	}
}
