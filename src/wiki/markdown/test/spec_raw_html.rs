use speculate::speculate;

use super::*;

speculate! { describe "markdown raw html" {

it "should parse open tag" {
	test(r#"
		<a><bab><c2c>
	"#, r#"
		<p><a><bab><c2c></p>
	"#);

	test(r#"
		<a/><b2/>
	"#, r#"
		<p><a/><b2/></p>
	"#);

	test(r#"
		<a  /><b2
		data="foo" >
	"#, r#"
		<p><a  /><b2
		data="foo" ></p>
	"#);

	// spell-checker: disable
	test(r#"
		<a foo="bar" bam = 'baz <em>"</em>'
		_boolean zoop:33=zoop:33 />
	"#, r#"
		<p><a foo="bar" bam = 'baz <em>"</em>'
		_boolean zoop:33=zoop:33 /></p>
	"#);
	// spell-checker: enable

	test(r#"
		Foo <responsive-image src="foo.jpg" />
	"#, r#"
		<p>Foo <responsive-image src="foo.jpg" /></p>
	"#);
}

it "should parse closing tags" {
	test(r#"
		</a></foo >
	"#, r#"
		<p></a></foo ></p>
	"#);
}

it "should parse comments" {
	test(r#"
		foo <!-- this is a
		comment - with hyphen -->
	"#, r#"
		<p>foo <!-- this is a
		comment - with hyphen --></p>
	"#);

	test(r#"
		foo <!-- not a comment -- two hyphens -->
	"#, r#"
		<p>foo &lt;!-- not a comment -- two hyphens --&gt;</p>
	"#);

	// not comments:

	test(r#"
		foo <!--> foo -->
	"#, r#"
		<p>foo &lt;!--&gt; foo --&gt;</p>
	"#);

	test(r#"
		foo <!-- foo--->
	"#, r#"
		<p>foo &lt;!-- foo---&gt;</p>
	"#);
}

it "should not parse illegal HTML" {
	test(r#"
		<33> <__>
	"#, r#"
		<p>&lt;33&gt; &lt;__&gt;</p>
	"#);

	test(r#"
		<a h*#ref="hi">
	"#, r#"
		<p>&lt;a h*#ref=&quot;hi&quot;&gt;</p>
	"#);

	test(r#"
		<a href="hi'> <a href=hi'>
	"#, r#"
		<p>&lt;a href=&quot;hi&apos;&gt; &lt;a href=hi&apos;&gt;</p>
	"#);

	test(r#"
		< a><
		foo><bar/ >
		<foo bar=baz
		bim!bop />
	"#, r#"
		<p>&lt; a&gt;&lt;
		foo&gt;&lt;bar/ &gt;
		&lt;foo bar=baz
		bim!bop /&gt;</p>
	"#);

	// spell-checker: disable
	test(r#"
		<a href='bar'title=title>
	"#, r#"
		<p>&lt;a href=&apos;bar&apos;title=title&gt;</p>
	"#);
	// spell-checker: enable

	test(r#"
		</a href="foo">
	"#, r#"
		<p>&lt;/a href=&quot;foo&quot;&gt;</p>
	"#);
}

it "should parse processing instructions" {
	test(r#"
		foo <?php echo $a; ?> bar
	"#, r#"
		<p>foo <?php echo $a; ?> bar</p>
	"#);
}

it "should parse declarations" {
	test(r#"
		foo <!ELEMENT br EMPTY> bar
	"#, r#"
		<p>foo <!ELEMENT br EMPTY> bar</p>
	"#);
}

it "should parse CDATA" {
	test(r#"
		foo <![CDATA[>&<]]> bar
	"#, r#"
		<p>foo <![CDATA[>&<]]> bar</p>
	"#);
}

it "do not parse backslash escapes" {
	test(r#"
		foo <a href="\*">

		<a href="\"">
	"#, r#"
		<p>foo <a href="\*"></p>
		<p>&lt;a href=&quot;&quot;&quot;&gt;</p>
	"#);
}

}}
