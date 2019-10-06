use super::*;

// spell-checker: disable

mod markdown_spec_fenced_code {
	use super::*;

	#[test]
	fn should_parse() {
		// example 89
		test(
			r##"
				```
				<
				>
				```
			"##,
			r##"
				<pre><code>&lt;
				&gt;
				</code></pre>
			"##,
		);

		// example 90
		test(
			r##"
				~~~
				<
				 >
				~~~
			"##,
			r##"
				<pre><code>&lt;
				 &gt;
				</code></pre>
			"##,
		);

		// example 91
		test(
			r##"
				``
				foo
				``
			"##,
			r##"
				<p><code>foo</code></p>
			"##,
		);

		// example 92
		test(
			r##"
				```
				aaa
				~~~
				```
			"##,
			r##"
				<pre><code>aaa
				~~~
				</code></pre>
			"##,
		);

		// example 93
		test(
			r##"
				~~~
				aaa
				```
				~~~
			"##,
			r##"
				<pre><code>aaa
				```
				</code></pre>
			"##,
		);
	}

	#[test]
	fn closing_fence_should_be_at_least_as_long_as_opening() {
		// example 94
		test(
			r##"
				````
				aaa
				```
				``````
			"##,
			r##"
				<pre><code>aaa
				```
				</code></pre>
			"##,
		);

		// example 95
		test(
			r##"
				~~~~
				aaa
				~~~
				~~~~
			"##,
			r##"
				<pre><code>aaa
				~~~
				</code></pre>
			"##,
		);
	}

	#[test]
	fn unclosed_blocks_end_with_the_enclosing_context() {
		// example 96
		test(
			r##"
				```
			"##,
			r##"
				<pre><code></code></pre>
			"##,
		);

		// example 97
		test(
			r##"
				`````

				```
				aaa
			"##,
			r##"
				<pre><code>
				```
				aaa</code></pre>
			"##,
		);

		// example 98
		test(
			r##"
				> ```
				> aaa

				bbb
			"##,
			r##"
				<blockquote>
				<pre><code>aaa
				</code></pre>
				</blockquote>
				<p>bbb</p>
			"##,
		);
	}

	#[test]
	fn allow_empty_lines_and_content() {
		// example 99
		test_raw("```\n\n  \n```", "<pre><code>\n  \n</code></pre>");

		// example 100
		test(
			r##"
				```
				```
			"##,
			r##"
				<pre><code></code></pre>
			"##,
		);

		// example 00
		test(
			r##"
			"##,
			r##"
			"##,
		);
	}
}
