use super::*;

// spell-checker: disable

mod markdown_spec_fenced_code {
	use super::*;

	#[test]
	fn example_89_should_parse() {
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
	}

	#[test]
	fn example_90_should_parse() {
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
	}

	#[test]
	fn example_91_should_parse() {
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
	}

	#[test]
	fn example_92_should_parse() {
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
	}

	#[test]
	fn example_93_should_parse() {
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
	fn example_94_closing_fence_should_be_at_least_as_long_as_opening() {
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
	}

	#[test]
	fn example_95_closing_fence_should_be_at_least_as_long_as_opening() {
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
	fn example_96_unclosed_blocks_end_with_the_enclosing_context() {
		// example 96
		test(
			r##"
				```
			"##,
			r##"
				<pre><code></code></pre>
			"##,
		);
	}

	#[test]
	fn example_97_unclosed_blocks_end_with_the_enclosing_context() {
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
	}

	#[test]
	fn example_98_unclosed_blocks_end_with_the_enclosing_context() {
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
	fn example_99_allow_empty_lines_and_content() {
		// example 99
		test_raw("```\n\n  \n```", "<pre><code>\n  \n</code></pre>");
	}

	#[test]
	fn example_100_allow_empty_lines_and_content() {
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
	fn example_101_allows_indentation() {
		// example 101
		test_raw(" ```\n aaa\naaa\n```", "<pre><code>aaa\naaa\n</code></pre>");
	}

	#[test]
	fn example_102_allows_indentation() {
		// example 102
		test_raw(
			"  ```\naaa\n  aaa\naaa\n  ```",
			"<pre><code>aaa\naaa\naaa\n</code></pre>",
		);
	}

	#[test]
	fn example_103_allows_indentation() {
		// example 103
		test_raw(
			"   ```\n   aaa\n    aaa\n  aaa\n   ```",
			"<pre><code>aaa\n aaa\naaa\n</code></pre>",
		);
	}

	#[test]
	fn example_104_allows_indentation() {
		// example 104
		test_raw("    ```\n    aaa\n    ```", "<pre><code>```\naaa\n```</code></pre>");
	}

	#[test]
	fn example_105_allows_indentation() {
		// example 105
		test_raw("```\naaa\n  ```", "<pre><code>aaa\n</code></pre>");
	}

	#[test]
	fn example_106_allows_indentation() {
		// example 106
		test_raw("   ```\naaa\n  ```", "<pre><code>aaa\n</code></pre>");
	}

	#[test]
	fn example_107_allows_indentation() {
		// example 107
		test_raw("```\naaa\n    ```\n", "<pre><code>aaa\n    ```\n</code></pre>");
	}

	#[test]
	fn example_108_does_not_allow_spaces_in_fences() {
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
	}

	#[test]
	fn example_109_does_not_allow_spaces_in_fences() {
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
	fn example_110_can_interrupt_paragraphs_and_be_follow_by_them() {
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
	fn example_111_do_not_require_blank_lines() {
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
	fn example_112_supports_info_string() {
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
				<pre><code class="language-ruby">def foo(x)
					return 3
				end
				</code></pre>
			"##,
		);
	}

	#[test]
	fn example_113_supports_info_string() {
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
				<pre><code class="language-ruby" data-info="startline=&apos;3&apos; $%@#$&amp;">def foo(x)
					return 4
				end
				</code></pre>
			"##,
		);
	}

	#[test]
	fn example_114_supports_info_string() {
		// example 114
		test(
			r##"
				````;
				````
			"##,
			r##"
				<pre><code data-info=";"></code></pre>
			"##,
		);
	}

	#[test]
	fn example_115_info_strings_for_backtick_code_blocks_cannot_contain_backticks() {
		// example 115
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
	}

	#[test]
	fn example_116_info_strings_for_tilde_code_blocks_can_contain_backticks_and_tilde() {
		// example 116
		test(
			r##"
				~~~ aa ``` ~~~
				foo
				~~~
			"##,
			r##"
				<pre><code class="language-aa" data-info="``` ~~~">foo
				</code></pre>
			"##,
		);
	}

	#[test]
	fn example_117_closing_code_fences_cannot_have_info_strings() {
		// example 117
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
