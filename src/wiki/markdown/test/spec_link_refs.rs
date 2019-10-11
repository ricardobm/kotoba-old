use super::*;

// spell-checker: disable

mod markdown_spec_link_refs {
	use super::*;

	#[test]
	fn example_161_should_parse() {
		// example 161
		test(
			r##"
				[foo]: /url "title"

				[foo]
			"##,
			r##"
				<p><a href="/url" title="title">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_162_should_parse() {
		// example 162
		test_raw(
			"   [foo]: \n      /url  \n           'the title'  \n\n[foo]",
			r#"<p><a href="/url" title="the title">foo</a></p>"#,
		);
	}

	#[test]
	fn example_163_should_parse() {
		// example 163
		test(
			r##"
				[Foo*bar\]]:my_(url) 'title (with parens)'

				[Foo*bar\]]
			"##,
			r##"
				<p><a href="my_(url)" title="title (with parens)">Foo*bar]</a></p>
			"##,
		);
	}

	#[test]
	fn example_164_can_be_multiline() {
		// example 164
		test(
			r##"
				[Foo bar]:
				<my url>
				'title'

				[Foo bar]
			"##,
			r##"
				<p><a href="my url" title="title">Foo bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_165_can_be_multiline() {
		// example 165
		test(
			r##"
				[foo]: /url '
				title
				line1
				line2
				'

				[foo]
			"##,
			r##"
				<p><a href="/url" title="
				title
				line1
				line2
				">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_166_can_be_multiline() {
		// example 166
		test(
			r##"
				[foo]: /url 'title

				with blank line'

				[foo]
			"##,
			r##"
				<p>[foo]: /url &apos;title</p>
				<p>with blank line&apos;</p>
				<p>[foo]</p>
			"##,
		);
	}

	#[test]
	fn example_167_title_can_be_omitted() {
		// example 167
		test(
			r##"
				[foo]:
				/url

				[foo]
			"##,
			r##"
				<p><a href="/url">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_168_destination_cannot_be_omitted() {
		// example 168
		test(
			r##"
				[foo]:

				[foo]
			"##,
			r##"
				<p>[foo]:</p>
				<p>[foo]</p>
			"##,
		);
	}

	#[test]
	fn example_169_destination_cannot_be_omitted() {

		// example 169
		test(
			r##"
				[foo]: <>

				[foo]
			"##,
			r##"
				<p><a href="">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_170_title_must_be_separated_by_space() {
		// example 170
		test(
			r##"
				[foo]: <bar>(baz)

				[foo]
			"##,
			r##"
				<p>[foo]: <bar>(baz)</p>
				<p>[foo]</p>
			"##,
		);
	}

	#[test]
	fn example_171_title_and_destination_can_contain_escapes() {
		// example 171
		test(
			r##"
				[foo]: /url\bar\*baz "foo\"bar\baz"

				[foo]
			"##,
			r##"
				<p><a href="/url\bar*baz" title="foo&quot;bar\baz">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_172_link_can_come_before_definition() {
		// example 172
		test(
			r##"
				[foo]

				[foo]: url
			"##,
			r##"
				<p><a href="url">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_173_first_definition_takes_precedence() {
		// example 173
		test(
			r##"
				[foo]

				[foo]: first
				[foo]: second
			"##,
			r##"
				<p><a href="first">foo</a></p>
			"##,
		);

		test(
			r##"
				[foo]

				[FOO]: first
				[foo]: second
			"##,
			r##"
				<p><a href="first">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_174_label_matching_is_case_insensitive() {
		// example 174
		test(
			r##"
				[FOO]: /url

				[Foo]
			"##,
			r##"
				<p><a href="/url">Foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_175_label_matching_is_case_insensitive() {
		// example 175
		test(
			r##"
				[ΑΓΩ]: /φου

				[αγω]
			"##,
			r##"
				<p><a href="/φου">αγω</a></p>
			"##,
		);
	}

	#[test]
	fn example_176_generates_empty_markup() {
		// example 176
		test(
			r##"
				[foo]: /url
			"##,
			r##"
			"##,
		);
	}

	#[test]
	fn example_177_generates_empty_markup() {
		// example 177
		test(
			r##"
				[
				foo
				]: /url
				bar
			"##,
			r##"
				<p>bar</p>
			"##,
		);
	}

	#[test]
	fn example_178_is_not_one_of_these() {
		// example 178
		test(
			r##"
				[foo]: /url "title" ok
			"##,
			r##"
				<p>[foo]: /url &quot;title&quot; ok</p>
			"##,
		);
	}

	#[test]
	fn example_179_is_not_one_of_these() {
		// example 179
		test(
			r##"
				[foo]: /url
				"title" ok
			"##,
			r##"
				<p>&quot;title&quot; ok</p>
			"##,
		);
	}

	#[test]
	fn example_180_is_not_one_of_these() {
		// example 180
		test_raw(
			"    [foo]: /url \"title\"\n\n[foo]",
			"<pre><code>[foo]: /url &quot;title&quot;</code></pre>\n<p>[foo]</p>",
		);
		test_raw(
			"	[foo]: /url \"title\"\n\n[foo]",
			"<pre><code>[foo]: /url &quot;title&quot;</code></pre>\n<p>[foo]</p>",
		);
	}

	#[test]
	fn example_181_is_not_one_of_these() {
		// example 181
		test(
			r##"
				```
				[foo]: /url
				```

				[foo]
			"##,
			r##"
				<pre><code>[foo]: /url
				</code></pre>
				<p>[foo]</p>
			"##,
		);
	}

	#[test]
	fn example_182_cannot_interrupt_paragraph() {
		// example 182
		test(
			r##"
				Foo
				[bar]: /baz

				[bar]
			"##,
			r##"
				<p>Foo
				[bar]: /baz</p>
				<p>[bar]</p>
			"##,
		);
	}

	#[test]
	fn example_183_does_not_need_blank_lines() {
		// example 183
		test(
			r##"
				# [Foo]
				[foo]: /url
				> bar
			"##,
			r##"
				<h1><a href="/url">Foo</a></h1>
				<blockquote>
				<p>bar</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_184_does_not_need_blank_lines() {
		// example 184
		test(
			r##"
				[foo]: /url
				bar
				===
				[foo]
			"##,
			r##"
				<h1>bar</h1>
				<p><a href="/url">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_185_does_not_need_blank_lines() {
		// example 185
		test(
			r##"
				[foo]: /url
				===
				[foo]
			"##,
			r##"
				<p>===
				<a href="/url">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_186_can_appear_sequentially() {
		// example 186
		test(
			r##"
				[foo]: /foo-url "foo"
				[bar]: /bar-url
				  "bar"
				[baz]: /baz-url

				[foo],
				[bar],
				[baz]
			"##,
			r##"
				<p><a href="/foo-url" title="foo">foo</a>,
				<a href="/bar-url" title="bar">bar</a>,
				<a href="/baz-url">baz</a></p>
			"##,
		);
	}

	#[test]
	fn example_187_can_occur_in_container() {
		// example 187
		test(
			r##"
				[foo]

				> [foo]: /url
			"##,
			r##"
				<p><a href="/url">foo</a></p>
				<blockquote>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn example_188_does_not_depend_on_usage() {
		// example 188
		test(
			r##"
				[foo]: /url
			"##,
			r##"
			"##,
		);
	}
}
