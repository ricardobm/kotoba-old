use speculate::speculate;

use super::*;

speculate! { describe "markdown escape sequences" {

it "should support backslash escapes" {
	test(r#"
		\!\"\#\$\%\&\'\(\)\*\+\,\-\.\/\:\;\<\=\>\?\@\[\\\]\^\_\`\{\|\}\~
	"#, r#"
		<p>!&quot;#$%&amp;&apos;()*+,-./:;&lt;=&gt;?@[\]^_`{|}~</p>
	"#);
}

it "should generated backslash when not recognized" {
	test(r#"
		\→\A\a\ \3\φ\«
	"#, r#"
		<p>\→\A\a\ \3\φ\«</p>
	"#);
}

it "should support hard line breaks" {
	test(r#"
		foo\
		bar

		bla\
	"#, r#"
		<p>foo<br/>
		bar</p>
		<p>bla\</p>
	"#);
}

}}
