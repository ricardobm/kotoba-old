use speculate::speculate;

use super::*;

speculate! { describe "markdown emphasis" {

it "should parse basic" {
	test(r#"
		*foo bar*
	"#, r#"
		<p><em>foo bar</em></p>
	"#);

	test(r#"
		**foo bar**
	"#, r#"
		<p><strong>foo bar</strong></p>
	"#);

	test(r#"
		_foo bar_
	"#, r#"
		<p><em>foo bar</em></p>
	"#);

	test(r#"
		__foo bar__
	"#, r#"
		<p><strong>foo bar</strong></p>
	"#);

	test(r#"
		*foo **bar** 123*
	"#, r#"
		<p><em>foo <strong>bar</strong> 123</em></p>
	"#);

	test(r#"
		_foo __bar__ 123_
	"#, r#"
		<p><em>foo <strong>bar</strong> 123</em></p>
	"#);
}

}}
