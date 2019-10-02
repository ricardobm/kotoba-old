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

		< http://foo.bar >

		<m:abc>

		<foo.bar.baz>

		foo@bar.example.com

		<foo\+@bar.example.com>
	"#, r#"
		<p>&lt;&gt;</p>
		<p>&lt; http://foo.bar &gt;</p>
		<p>&lt;m:abc&gt;</p>
		<p>&lt;foo.bar.baz&gt;</p>
		<p>foo@bar.example.com</p>
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

}}
