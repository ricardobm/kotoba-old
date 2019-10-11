use super::*;

// spell-checker: disable

mod markdown_spec_breaks {
	use super::*;

	#[test]
	fn example_654_hard_line_breaks_spaces() {
		// example 654
		test_raw_in(
			"foo  \nbaz",
			r##"
				<p>foo<br/>
				baz</p>
			"##,
		);
	}

	#[test]
	fn example_655_hard_line_breaks_backslash() {
		// example 655
		test_raw_in(
			"foo\\\nbaz",
			r##"
				<p>foo<br/>
				baz</p>
			"##,
		);
	}

	#[test]
	fn example_656_hard_line_breaks_more_than_two_spaces() {
		// example 656
		test_raw_in(
			"foo       \nbaz",
			r##"
				<p>foo<br/>
				baz</p>
			"##,
		);
	}

	#[test]
	fn example_657_hard_line_breaks_ignores_leading_spaces() {
		// example 657
		test_raw_in(
			"foo  \n     bar",
			r##"
				<p>foo<br/>
				bar</p>
			"##,
		);
	}

	#[test]
	fn example_658_hard_line_breaks_ignores_leading_spaces() {
		// example 658
		test_raw_in(
			"foo\\\n     bar",
			r##"
				<p>foo<br/>
				bar</p>
			"##,
		);
	}

	#[test]
	fn example_659_hard_line_breaks_can_occur_inside_emphasis() {
		// example 659
		test_raw_in(
			"*foo  \nbar*",
			r##"
				<p><em>foo<br/>
				bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_660_hard_line_breaks_can_occur_inside_emphasis() {
		// example 660
		test_raw_in(
			"*foo\\\nbar*",
			r##"
				<p><em>foo<br/>
				bar</em></p>
			"##,
		);
	}

	#[test]
	fn example_661_hard_line_breaks_cannot_occur_inside_code_spans() {
		// example 661
		test_raw_in(
			"`code  \nspan`",
			r##"
				<p><code>code   span</code></p>
			"##,
		);
	}

	#[test]
	fn example_662_hard_line_breaks_cannot_occur_inside_code_spans() {
		// example 662
		test_raw_in(
			"`code\\\nspan`",
			r##"
				<p><code>code\ span</code></p>
			"##,
		);
	}

	#[test]
	fn example_663_hard_line_breaks_cannot_occur_inside_html() {
		// example 663
		test_raw("<a href=\"foo  \nbar\">", "<p><a href=\"foo  \nbar\"></p>");
	}

	#[test]
	fn example_664_hard_line_breaks_cannot_occur_inside_html() {
		// example 664
		test_raw_in(
			"<a href=\"foo\\\nbar\">",
			r##"
				<p><a href="foo\
				bar"></p>
			"##,
		);
	}

	#[test]
	fn example_665_hard_line_breaks_do_not_work_at_the_end_of_a_block() {
		// example 665
		test_raw_in(
			"foo\\",
			r##"
				<p>foo\</p>
			"##,
		);
	}

	#[test]
	fn example_666_hard_line_breaks_do_not_work_at_the_end_of_a_block() {
		// example 666
		test_raw_in(
			"foo  ",
			r##"
				<p>foo</p>
			"##,
		);
	}

	#[test]
	fn example_667_hard_line_breaks_do_not_work_at_the_end_of_a_block() {
		// example 667
		test_raw_in(
			"### foo\\",
			r##"
				<h3>foo\</h3>
			"##,
		);
	}

	#[test]
	fn example_668_hard_line_breaks_do_not_work_at_the_end_of_a_block() {
		// example 668
		test_raw_in(
			"### foo  ",
			r##"
				<h3>foo</h3>
			"##,
		);
	}

	#[test]
	fn example_669_soft_break() {
		// example 669
		test_raw_in(
			"foo\nbaz",
			r##"
				<p>foo
				baz</p>
			"##,
		);
	}

	#[test]
	fn example_670_soft_break_ignores_trailing_and_leading_spaces() {
		// example 670
		test_raw_in(
			"foo \n baz",
			r##"
				<p>foo
				baz</p>
			"##,
		);
	}
}
