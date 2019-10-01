use speculate::speculate;

use super::*;

speculate! { describe "markdown HTML entities" {

// spell-checker: disable

it "should generate named entities" {
	test(r#"
		&nbsp; &amp; &copy; &AElig; &Dcaron;
		&frac34; &HilbertSpace; &DifferentialD;
		&ClockwiseContourIntegral; &ngE;
	"#, r#"
		<p>&nbsp; &amp; © Æ Ď
		¾ ℋ ⅆ
		∲ ≧̸</p>
	"#);
}

it "should output non entities" {
	test(r#"
		&nbsp &x; &#; &#x;
		&#87654321;
		&#abcdef0;
		&ThisIsNotDefined; &hi?;
	"#, r#"
		<p>&amp;nbsp &amp;x; &amp;#; &amp;#x;
		&amp;#87654321;
		&amp;#abcdef0;
		&amp;ThisIsNotDefined; &amp;hi?;</p>
	"#);
}

it "should support decimal entities" {
	test(
		"&#35; &#1234; &#992; &#0; &#1114111; &#1114112; &#9999999; &#10000000;",
		"<p># Ӓ Ϡ � \u{10FFFF} � � &amp;#10000000;</p>",
	);
}

it "should support hexadecimal entities" {
	test(
		"&#X22; &#XD06; &#xcab; &#x0; &#x10FFFF; &#x110000; &#xFFFFFF; &#x1000000;",
		"<p>&quot; ആ ಫ � \u{10FFFF} � � &amp;#x1000000;</p>",
	);
}

it "should generate U+FFFD or \\0" {
	test("\0 x\0x \0", r#"
		<p>� x�x �</p>
	"#);
}

}}
