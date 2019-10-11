use super::*;

// spell-checker: disable

mod markdown_spec_autolinks {
	use super::*;

	#[test]
	fn example_602_should_parse() {
		// example 602
		test(
			r##"
				<http://foo.bar.baz>
			"##,
			r##"
				<p><a href="http://foo.bar.baz">http://foo.bar.baz</a></p>
			"##,
		);
	}

	#[test]
	fn example_603_should_parse() {
		// example 603
		test(
			r##"
				<http://foo.bar.baz/test?q=hello&id=22&boolean>
			"##,
			r##"
				<p><a href="http://foo.bar.baz/test?q=hello&amp;id=22&amp;boolean">http://foo.bar.baz/test?q=hello&amp;id=22&amp;boolean</a></p>
			"##,
		);
	}

	#[test]
	fn example_604_should_parse() {
		// example 604
		test(
			r##"
				<irc://foo.bar:2233/baz>
			"##,
			r##"
				<p><a href="irc://foo.bar:2233/baz">irc://foo.bar:2233/baz</a></p>
			"##,
		);
	}

	#[test]
	fn example_605_uppercase() {
		// example 605
		test(
			r##"
				<MAILTO:FOO@BAR.BAZ>
			"##,
			r##"
				<p><a href="MAILTO:FOO@BAR.BAZ">MAILTO:FOO@BAR.BAZ</a></p>
			"##,
		);
	}

	#[test]
	fn example_606_absolute_uris_may_not_be_valid() {
		// example 606
		test(
			r##"
				<a+b+c:d>
			"##,
			r##"
				<p><a href="a+b+c:d">a+b+c:d</a></p>
			"##,
		);
	}

	#[test]
	fn example_607_absolute_uris_may_not_be_valid() {
		// example 607
		test(
			r##"
				<made-up-scheme://foo,bar>
			"##,
			r##"
				<p><a href="made-up-scheme://foo,bar">made-up-scheme://foo,bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_608_absolute_uris_may_not_be_valid() {
		// example 608
		test(
			r##"
				<http://../>
			"##,
			r##"
				<p><a href="http://../">http://../</a></p>
			"##,
		);
	}

	#[test]
	fn example_609_absolute_uris_may_not_be_valid() {
		// example 609
		test(
			r##"
				<localhost:5001/foo>
			"##,
			r##"
				<p><a href="localhost:5001/foo">localhost:5001/foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_610_spaces_not_allowed() {
		// example 610
		test(
			r##"
				<http://foo.bar/baz bim>
			"##,
			r##"
				<p>&lt;http://foo.bar/baz bim&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_611_escapes_do_not_work() {
		// example 611
		test(
			r##"
				<http://example.com/\[\>
			"##,
			r##"
				<p><a href="http://example.com/\[\">http://example.com/\[\</a></p>
			"##,
		);
	}

	#[test]
	fn example_612_email() {
		// example 612
		test(
			r##"
				<foo@bar.example.com>
			"##,
			r##"
				<p><a href="mailto:foo@bar.example.com">foo@bar.example.com</a></p>
			"##,
		);
	}

	#[test]
	fn example_613_email() {
		// example 613
		test(
			r##"
				<foo+special@Bar.baz-bar0.com>
			"##,
			r##"
				<p><a href="mailto:foo+special@Bar.baz-bar0.com">foo+special@Bar.baz-bar0.com</a></p>
			"##,
		);
	}

	#[test]
	fn example_614_not_escaped_email() {
		// example 614
		test(
			r##"
				<foo\+@bar.example.com>
			"##,
			r##"
				<p>&lt;foo+@bar.example.com&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_615_not_autolink() {
		// example 615
		test(
			r##"
				<>
			"##,
			r##"
				<p>&lt;&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_616_autolink_extension() {
		// example 616
		test(
			r##"
				< http://foo.bar >
			"##,
			r##"
				<p>&lt; <a href="http://foo.bar">http://foo.bar</a> &gt;</p>
			"##,
		);
	}

	#[test]
	fn example_617_not_autolink() {
		// example 617
		test(
			r##"
				<m:abc>
			"##,
			r##"
				<p>&lt;m:abc&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_618_not_autolink() {
		// example 618
		test(
			r##"
				<foo.bar.baz>
			"##,
			r##"
				<p>&lt;foo.bar.baz&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_619_autolink_extension() {
		// example 619
		test(
			r##"
				http://example.com
			"##,
			r##"
				<p><a href="http://example.com">http://example.com</a></p>
			"##,
		);
	}

	#[test]
	fn example_620_autolink_email_extension() {
		// example 620
		test(
			r##"
				foo@bar.example.com
			"##,
			r##"
				<p><a href="mailto:foo@bar.example.com">foo@bar.example.com</a></p>
			"##,
		);
	}

	//===========================================//
	// GFM extensions
	//===========================================//

	#[test]
	fn example_621_gfm_extension() {
		// example 621
		test(
			r##"
				www.commonmark.org
			"##,
			r##"
				<p><a href="http://www.commonmark.org">www.commonmark.org</a></p>
			"##,
		);
	}

	#[test]
	fn example_622_gfm_extension() {
		// example 622
		test(
			r##"
				Visit www.commonmark.org/help for more information.
			"##,
			r##"
				<p>Visit <a href="http://www.commonmark.org/help">www.commonmark.org/help</a> for more information.</p>
			"##,
		);
	}

	#[test]
	fn example_623_gfm_extension_trailing_punctuation() {
		// example 623
		test(
			r##"
				Visit www.commonmark.org.

				Visit www.commonmark.org/a.b.
			"##,
			r##"
				<p>Visit <a href="http://www.commonmark.org">www.commonmark.org</a>.</p>
				<p>Visit <a href="http://www.commonmark.org/a.b">www.commonmark.org/a.b</a>.</p>
			"##,
		);
	}

	#[test]
	fn example_624_gfm_extension_balanced_parenthesis_on_ending() {
		// example 624
		test(
			r##"
				www.google.com/search?q=Markup+(business)

				www.google.com/search?q=Markup+(business)))

				(www.google.com/search?q=Markup+(business))

				(www.google.com/search?q=Markup+(business)
			"##,
			r##"
				<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
				<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>))</p>
				<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>)</p>
				<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
			"##,
		);
	}

	#[test]
	fn example_625_gfm_extension_parenthesis_ignored_if_not_ending() {
		// example 625
		test(
			r##"
				www.google.com/search?q=(business))+ok
			"##,
			r##"
				<p><a href="http://www.google.com/search?q=(business))+ok">www.google.com/search?q=(business))+ok</a></p>
			"##,
		);
	}

	#[test]
	fn example_626_gfm_extension_trailing_entity() {
		// example 626
		test(
			r##"
				www.google.com/search?q=commonmark&hl=en

				www.google.com/search?q=commonmark&hl;
			"##,
			r##"
				<p><a href="http://www.google.com/search?q=commonmark&amp;hl=en">www.google.com/search?q=commonmark&amp;hl=en</a></p>
				<p><a href="http://www.google.com/search?q=commonmark">www.google.com/search?q=commonmark</a>&amp;hl;</p>
			"##,
		);
	}

	#[test]
	fn example_627_gfm_extension_ends_on_less_than() {
		// example 627
		test(
			r##"
				www.commonmark.org/he<lp
			"##,
			r##"
				<p><a href="http://www.commonmark.org/he">www.commonmark.org/he</a>&lt;lp</p>
			"##,
		);
	}

	#[test]
	fn example_628_gfm_extension_http() {
		// example 628
		test(
			r##"
				http://commonmark.org

				(Visit https://encrypted.google.com/search?q=Markup+(business))
			"##,
			r##"
				<p><a href="http://commonmark.org">http://commonmark.org</a></p>
				<p>(Visit <a href="https://encrypted.google.com/search?q=Markup+(business)">https://encrypted.google.com/search?q=Markup+(business)</a>)</p>
			"##,
		);
	}

	#[test]
	fn example_629_gfm_extension_email() {
		// example 629
		test(
			r##"
				foo@bar.baz
			"##,
			r##"
				<p><a href="mailto:foo@bar.baz">foo@bar.baz</a></p>
			"##,
		);
	}

	#[test]
	fn example_630_gfm_extension_email_characters() {
		// example 630
		test(
			r##"
				hello@mail+xyz.example isn't valid, but hello+xyz@mail.example is.
			"##,
			r##"
				<p>hello@mail+xyz.example isn&apos;t valid, but <a href="mailto:hello+xyz@mail.example">hello+xyz@mail.example</a> is.</p>
			"##,
		);
	}

	#[test]
	fn example_631_gfm_extension_email_characters() {
		// example 631
		test(
			r##"
				a.b-c_d@a.b

				a.b-c_d@a.b.

				a.b-c_d@a.b-

				a.b-c_d@a.b_
			"##,
			r##"
				<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a></p>
				<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a>.</p>
				<p>a.b-c_d@a.b-</p>
				<p>a.b-c_d@a.b_</p>
			"##,
		);
	}
}
