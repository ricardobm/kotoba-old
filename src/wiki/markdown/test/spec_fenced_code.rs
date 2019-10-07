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
	}

	#[test]
	fn allows_indentation() {
		// example 101
		test_raw(" ```\n aaa\naaa\n```", "<pre><code>aaa\naaa\n</code></pre>");

		// example 102
		test_raw(
			"  ```\naaa\n  aaa\naaa\n  ```",
			"<pre><code>aaa\naaa\naaa\n</code></pre>",
		);

		// example 103
		test_raw(
			"   ```\n   aaa\n    aaa\n  aaa\n   ```",
			"<pre><code>aaa\n aaa\naaa\n</code></pre>",
		);

		// example 104
		test_raw("    ```\n    aaa\n    ```", "<pre><code>```\naaa\n```</code></pre>");

		// example 105
		test_raw("```\naaa\n  ```", "<pre><code>aaa\n</code></pre>");

		// example 106
		test_raw("   ```\naaa\n  ```", "<pre><code>aaa\n</code></pre>");

		// example 107
		test_raw("```\naaa\n    ```\n", "<pre><code>aaa\n    ```\n</code></pre>");
	}

	#[test]
	fn does_not_allow_spaces_in_fences() {
		// example 108
		test(
			r##"
				``` ```
				aaa
			"##,
			r##"
				<p><code> </code>
				aaa</p>
			"##,
		);

		// example 109
		test(
			r##"
				~~~~~~
				aaa
				~~~ ~~
			"##,
			r##"
				<pre><code>aaa
				~~~ ~~</code></pre>
			"##,
		);
	}

	#[test]
	fn can_interrupt_paragraphs_and_be_follow_by_them() {
		// example 110
		test(
			r##"
				foo
				```
				bar
				```
				baz
			"##,
			r##"
				<p>foo</p>
				<pre><code>bar
				</code></pre>
				<p>baz</p>
			"##,
		);
	}

	#[test]
	fn do_not_require_blank_lines() {
		// example 111
		test(
			r##"
				foo
				---
				~~~
				bar
				~~~
				# baz
			"##,
			r##"
				<h2>foo</h2>
				<pre><code>bar
				</code></pre>
				<h1>baz</h1>
			"##,
		);
	}

	#[test]
	fn support_info_string() {
		// example 112
		test(
			r##"
				```ruby
				def foo(x)
					return 3
				end
				```
			"##,
			r##"
				<pre><code class='language-ruby'>def foo(x)
					return 3
				end
				</code></pre>
			"##,
		);

		// example 113
		test(
			r##"
				~~~~    ruby startline='3' $%@#$&
				def foo(x)
					return 4
				end
				~~~~~~~
			"##,
			r##"
				<pre><code class='language-ruby' data-info='startline=&apos;3&apos; $%@#$&amp;'>def foo(x)
					return 4
				end
				</code></pre>
			"##,
		);

		// example 114
		test(
			r##"
				````;
				````
			"##,
			r##"
				<pre><code data-info=';'></code></pre>
			"##,
		);

		// example 115 - info strings for backtick code blocks cannot contain backticks
		test(
			r##"
				``` aa ```
				foo
			"##,
			r##"
				<p><code>aa</code>
				foo</p>
			"##,
		);

		// example 116 - info strings for tilde code blocks can contain backticks and tilde
		test(
			r##"
				~~~ aa ``` ~~~
				foo
				~~~
			"##,
			r##"
				<pre><code class='language-aa' data-info='``` ~~~'>foo
				</code></pre>
			"##,
		);

		// example 117 - closing code fences cannot have info strings
		test(
			r##"
				```
				``` aaa
				```
			"##,
			r##"
				<pre><code>``` aaa
				</code></pre>
			"##,
		);
	}
}
