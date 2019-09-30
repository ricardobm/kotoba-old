use speculate::speculate;

use super::*;

speculate! { describe "markdown basics" {

it "should support simple paragraphs" {
	test(r#"
		Paragraph 1

		Paragraph 2

		3.1
		3.2
	"#, r#"
		<p>Paragraph 1</p>
		<p>Paragraph 2</p>
		<p>3.1
		3.2</p>
	"#);
}

it "should support lists" {
	test(r#"
		- Item 1
		- Item 2
		- Item 3
	"#, r#"
		<ul>
		<li>Item 1</li>
		<li>Item 2</li>
		<li>Item 3</li>
		</ul>
	"#);
}

it "should support ATX headings" {
	test(r#"
		# H 1
		## H 2
		### H 3
		#### H 4
		##### H 5
		###### H 6

		P1
		# H1 # ##############
		## H2##
		### H3 # # #
		P2
		####### H7
	"#, r#"
		<h1>H 1</h1>
		<h2>H 2</h2>
		<h3>H 3</h3>
		<h4>H 4</h4>
		<h5>H 5</h5>
		<h6>H 6</h6>
		<p>P1</p>
		<h1>H1 #</h1>
		<h2>H2##</h2>
		<h3>H3 # #</h3>
		<p>P2
		####### H7</p>
	"#);
}

it "should support setext headings" {
	test(r#"
		Title 1
		=======

		Title 2
		-------

		Multi-line
		Title 2
		---

		L1
		L2
		==
		===
		L3
		--
		---
	"#, r#"
		<h1>Title 1</h1>
		<h2>Title 2</h2>
		<h2>Multi-line
		Title 2</h2>
		<h1>L1
		L2
		==</h1>
		<h2>L3
		--</h2>
	"#);
}

it "should handle setext in blocks properly" {
	test(r#"
		setext heading
		--------------

		- this is not a title
		-------------
		- semantic break above
	"#, r#"
		<h2>setext heading</h2>
		<ul>
		<li>this is not a title</li>
		</ul>
		<hr/>
		<ul>
		<li>semantic break above</li>
		</ul>
	"#);

	test(r#"
		> quote title
		> -----------
		> bellow is not a title
		-----------------------
		> another quote
	"#, r#"
		<blockquote>
		<h2>quote title</h2>
		<p>bellow is not a title</p>
		</blockquote>
		<hr/>
		<blockquote>
		<p>another quote</p>
		</blockquote>
	"#);

	test(r#"
		- heading
		  -------
		- in a list
	"#, r#"
		<ul>
		<li>
		<h2>heading</h2>
		</li>
		<li>in a list</li>
		</ul>
	"#);
}

}}
