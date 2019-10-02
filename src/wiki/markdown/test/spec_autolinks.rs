use speculate::speculate;

use super::*;

speculate! { describe "markdown autolinks" {

// spell-checker: disable

it "should parse" {
	test(r#"
		Some link: <http://foo.bar.baz>!
	"#, r#"
		<p>Some link: <a href="http://foo.bar.baz">http://foo.bar.baz</a>!</p>
	"#);

	test(r#"
		<http://foo.bar.baz/test?q=hello&id=22&boolean>
	"#, r#"
		<p><a href="http://foo.bar.baz/test?q=hello&amp;id=22&amp;boolean">http://foo.bar.baz/test?q=hello&amp;id=22&amp;boolean</a></p>
	"#);

	test(r#"
		<irc://foo.bar:2233/baz>
	"#, r#"
		<p><a href="irc://foo.bar:2233/baz">irc://foo.bar:2233/baz</a></p>
	"#);

	test(r#"
		<MAILTO:FOO@BAR.BAZ>
	"#, r#"
		<p><a href="MAILTO:FOO@BAR.BAZ">MAILTO:FOO@BAR.BAZ</a></p>
	"#);

	test(r#"
		<a+b+c:d>
	"#, r#"
		<p><a href="a+b+c:d">a+b+c:d</a></p>
	"#);

	test(r#"
		<made-up-scheme://foo,bar>
	"#, r#"
		<p><a href="made-up-scheme://foo,bar">made-up-scheme://foo,bar</a></p>
	"#);

	test(r#"
		<http://../>
	"#, r#"
		<p><a href="http://../">http://../</a></p>
	"#);

	test(r#"
		<localhost:5001/foo>
	"#, r#"
		<p><a href="localhost:5001/foo">localhost:5001/foo</a></p>
	"#);
}

it "do not allow spaces" {
	test(r#"
		<http://foo.bar/baz bim>

		<http://foo.bar/baz	bim>
	"#, r#"
		<p>&lt;http://foo.bar/baz bim&gt;</p>
		<p>&lt;http://foo.bar/baz	bim&gt;</p>
	"#);
}

it "do not have backslash escapes" {
	test(r#"
		<http://example.com/\[\>
	"#, r#"
		<p><a href="http://example.com/\[\">http://example.com/\[\</a></p>
	"#);
}

it "is not one of these" {
	test(r#"
		<>

		<http://foo.bar >

		<m:abc>

		<foo.bar.baz>

		<foo\+@bar.example.com>
	"#, r#"
		<p>&lt;&gt;</p>
		<p>&lt;http://foo.bar &gt;</p>
		<p>&lt;m:abc&gt;</p>
		<p>&lt;foo.bar.baz&gt;</p>
		<p>&lt;foo+@bar.example.com&gt;</p>
	"#);
}

it "supports email autolinks" {
	test(r#"
		<foo@bar.example.com>
	"#, r#"
		<p><a href="mailto:foo@bar.example.com">foo@bar.example.com</a></p>
	"#);

	test(r#"
		<foo+special@Bar.baz-bar0.com>
	"#, r#"
		<p><a href="mailto:foo+special@Bar.baz-bar0.com">foo+special@Bar.baz-bar0.com</a></p>
	"#);
}

it "supports GFM extension autolinks" {
	test(r#"
		http://example.com

		foo@bar.example.com
	"#, r#"
		<p><a href="http://example.com">http://example.com</a></p>
		<p><a href="mailto:foo@bar.example.com">foo@bar.example.com</a></p>
	"#);

	test(r#"
		www.commonmark.org

		Visit www.commonmark.org/help for more information.
	"#, r#"
		<p><a href="http://www.commonmark.org">www.commonmark.org</a></p>
		<p>Visit <a href="http://www.commonmark.org/help">www.commonmark.org/help</a> for more information.</p>
	"#);

	// Trailing punctuation is not considered part of the autolink:

	test(r#"
		Visit www.commonmark.org.

		Visit www.commonmark.org/a.b.
	"#, r#"
		<p>Visit <a href="http://www.commonmark.org">www.commonmark.org</a>.</p>
		<p>Visit <a href="http://www.commonmark.org/a.b">www.commonmark.org/a.b</a>.</p>
	"#);

	test(r#"www.commonmark.org/?/??"#, r#"<p><a href="http://www.commonmark.org/?/">www.commonmark.org/?/</a>??</p>"#);
	test(r#"www.commonmark.org/!/!!"#, r#"<p><a href="http://www.commonmark.org/!/">www.commonmark.org/!/</a>!!</p>"#);
	test(r#"www.commonmark.org/,/,,"#, r#"<p><a href="http://www.commonmark.org/,/">www.commonmark.org/,/</a>,,</p>"#);
	test(r#"www.commonmark.org/:/::"#, r#"<p><a href="http://www.commonmark.org/:/">www.commonmark.org/:/</a>::</p>"#);
	test(r#"www.commonmark.org/*/**"#, r#"<p><a href="http://www.commonmark.org/*/">www.commonmark.org/*/</a>**</p>"#);
	test(r#"www.commonmark.org/_/__"#, r#"<p><a href="http://www.commonmark.org/_/">www.commonmark.org/_/</a>__</p>"#);
	test(r#"www.commonmark.org/~/~~"#, r#"<p><a href="http://www.commonmark.org/~/">www.commonmark.org/~/</a>~~</p>"#);

	// Parenthesis handling:

	test(r#"
		www.google.com/search?q=Markup+(business)
	"#, r#"
		<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
	"#);

	test(r#"
		www.google.com/search?q=Markup+(business)))
	"#, r#"
		<p><a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>))</p>
	"#);

	test(r#"
		(www.google.com/search?q=Markup+(business))
	"#, r#"
		<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a>)</p>
	"#);

	test(r#"
		(www.google.com/search?q=Markup+(business)
	"#, r#"
		<p>(<a href="http://www.google.com/search?q=Markup+(business)">www.google.com/search?q=Markup+(business)</a></p>
	"#);

	test(r#"
		www.google.com/search?q=(business))+ok
	"#, r#"
		<p><a href="http://www.google.com/search?q=(business))+ok">www.google.com/search?q=(business))+ok</a></p>
	"#);

	// Entity trimming:

	test(r#"
		www.google.com/search?q=commonmark&hl=en
	"#, r#"
		<p><a href="http://www.google.com/search?q=commonmark&amp;hl=en">www.google.com/search?q=commonmark&amp;hl=en</a></p>
	"#);

	test(r#"
		www.google.com/search?q=commonmark&hl;
	"#, r#"
		<p><a href="http://www.google.com/search?q=commonmark">www.google.com/search?q=commonmark</a>&amp;hl;</p>
	"#);

	// `<` ends the autolink:

	test(r#"
		www.commonmark.org/he<lp
	"#, r#"
		<p><a href="http://www.commonmark.org/he">www.commonmark.org/he</a>&lt;lp</p>
	"#);

	// Extended URL:

	test(r#"
		http://commonmark.org

		(Visit https://encrypted.google.com/search?q=Markup+(business))
	"#, r#"
		<p><a href="http://commonmark.org">http://commonmark.org</a></p>
		<p>(Visit <a href="https://encrypted.google.com/search?q=Markup+(business)">https://encrypted.google.com/search?q=Markup+(business)</a>)</p>
	"#);

	// Email autolinks:

	test(r#"
		foo@bar.baz
	"#, r#"
		<p><a href="mailto:foo@bar.baz">foo@bar.baz</a></p>
	"#);

	test(r#"
		hello@mail+xyz.example is not valid, but hello+xyz@mail.example is.
	"#, r#"
		<p>hello@mail+xyz.example is not valid, but <a href="mailto:hello+xyz@mail.example">hello+xyz@mail.example</a> is.</p>
	"#);

	test(r#"
		a.b-c_d@a.b

		a.b-c_d@a.b.

		a.b-c_d@a.b-

		a.b-c_d@a.b_

		Sequence: a.b-c_d@a.b_ a.b-c_d@a.b

	"#, r#"
		<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a></p>
		<p><a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a>.</p>
		<p>a.b-c_d@a.b-</p>
		<p>a.b-c_d@a.b_</p>
		<p>Sequence: a.b-c_d@a.b_ <a href="mailto:a.b-c_d@a.b">a.b-c_d@a.b</a></p>
	"#);
}

}}
