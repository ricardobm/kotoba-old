use speculate::speculate;

use super::*;

speculate! { describe "markdown breaks" {

it "should parse" {
	test(r#"
		***
		---
		___
	"#, r#"
		<hr/>
		<hr/>
		<hr/>
	"#)
}

it "wrong characters" {
	test(r#"
		+++

		===
	"#, r#"
		<p>+++</p>
		<p>===</p>
	"#)
}

it "not enough characters" {
	test(r#"
		--
		**
		__
	"#, r#"
		<p>--
		**
		__</p>
	"#)
}

it "one to three spaces indent are allowed" {
	test(r#"
		 ***
		  ---
		   ___
	"#, r#"
		<hr/>
		<hr/>
		<hr/>
	"#)
}

it "four spaces is too many" {
	test(r#"
		!

		    ---

			---

		Foo
		    ---

		Bar
			---
	"#, r#"
		<p>!</p>
		<pre><code>---

		---</code></pre>
		<p>Foo
		    ---</p>
		<p>Bar
			---</p>
	"#)
}

it "more than three characters may be used" {
	test(r#"
		----------------------------------

		**********************************

		__________________________________

	"#, r#"
		<hr/>
		<hr/>
		<hr/>
	"#)
}

it "spaces are allowed between characters" {
	test(r#"
		!
		   -    -    -

		   **  *  * **

		_      _      _
	"#, r#"
		<p>!</p>
		<hr/>
		<hr/>
		<hr/>
	"#)
}

it "spaces are allowed at the end" {
	test("- - -  ", r#"
		<hr/>
	"#)
}

it "no other characters may occur on the line" {
	test(r#"
		_ _ _ _ a

		a________

		****a****
	"#, r#"
		<p>_ _ _ _ a</p>
		<p>a________</p>
		<p>****a****</p>
	"#)
}

it "non whitespace characters must be the same" {
	test(r#"
		_-_
	"#, r#"
		<p>_-_</p>
	"#)
}

#[ignore]
it "do not need blank lines before or after" {
	test(r#"
		- foo
		---
		- bar
	"#, r#"
		<ul>
		<li>foo</li>
		</ul>
		<hr/>
		<ul>
		<li>bar</li>
		</ul>
	"#)
}

it "can interrupt a paragraph" {
	test(r#"
		foo
		___
		bar
	"#, r#"
		<p>foo</p>
		<hr/>
		<p>bar</p>
	"#)
}

it "has lower precedence than setext" {
	test(r#"
		foo
		---
		bar
	"#, r#"
		<h2>foo</h2>
		<p>bar</p>
	"#)
}

#[ignore]
it "has higher precedence than a list item" {
	test(r#"
		* foo
		* * *
		* bar
	"#, r#"
		<ul>
		<li>foo</li>
		</ul>
		<hr/>
		<ul>
		<li>bar</li>
		</ul>
	"#)
}

#[ignore]
it "can be used in a list item" {
	test(r#"
		- foo
		- * * *
		- bar
	"#, r#"
		<ul>
		<li>foo</li>
		<li>
		<hr/>
		</li>
		<li>bar</li>
		</ul>
	"#)
}

}}
