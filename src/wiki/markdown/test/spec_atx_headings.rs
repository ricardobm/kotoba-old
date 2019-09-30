use speculate::speculate;

use super::*;

speculate! { describe "markdown ATX headings" {

it "should parse" {
	test(r#"
		# H1
		## H2
		### H3
		#### H4
		##### H5
		###### H6
	"#, r#"
		<h1>H1</h1>
		<h2>H2</h2>
		<h3>H3</h3>
		<h4>H4</h4>
		<h5>H5</h5>
		<h6>H6</h6>
	"#);
}

it "more than 6 is not a heading" {
	test(r#"
		####### not a heading
	"#, r#"
		<p>####### not a heading</p>
	"#);
}

it "at least one space is required" {
	test(r#"
		#5 bolt

		#hashtag
	"#, r#"
		<p>#5 bolt</p>
		<p>#hashtag</p>
	"#);
}

}}
