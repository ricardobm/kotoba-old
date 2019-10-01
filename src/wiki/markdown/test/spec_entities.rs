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

}}
