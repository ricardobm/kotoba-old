use super::*;

// spell-checker: disable

mod markdown_spec_entities {
	use super::*;

	#[test]
	fn example_321_named_entities() {
		// example 321
		test_raw(
			"&nbsp; &amp; &copy; &AElig; &Dcaron;\n&frac34; &HilbertSpace; &DifferentialD;\n&ClockwiseContourIntegral; &ngE;",
			"<p>&nbsp; &amp; © Æ Ď\n¾ ℋ ⅆ\n∲ ≧̸</p>",
		);
	}

	#[test]
	fn example_322_decimal() {
		// example 322
		test(
			r##"
				&#35; &#1234; &#992; &#0;
			"##,
			r##"
				<p># Ӓ Ϡ �</p>
			"##,
		);
	}

	#[test]
	fn example_323_hexadecimal() {
		// example 323
		test(
			r##"
				&#X22; &#XD06; &#xcab;
			"##,
			r##"
				<p>&quot; ആ ಫ</p>
			"##,
		);
	}

	#[test]
	fn example_324_non_entities() {
		// example 324
		test(
			r##"
				&nbsp &x; &#; &#x;
				&#87654321;
				&#abcdef0;
				&ThisIsNotDefined; &hi?;
			"##,
			r##"
				<p>&amp;nbsp &amp;x; &amp;#; &amp;#x;
				&amp;#87654321;
				&amp;#abcdef0;
				&amp;ThisIsNotDefined; &amp;hi?;</p>
			"##,
		);
	}

	#[test]
	fn example_325_without_trailing_semi_colon() {
		// example 325
		test(
			r##"
				&copy
			"##,
			r##"
				<p>&amp;copy</p>
			"##,
		);
	}

	#[test]
	fn example_326_made_up_entity() {
		// example 326
		test(
			r##"
				&MadeUpEntity;
			"##,
			r##"
				<p>&amp;MadeUpEntity;</p>
			"##,
		);
	}

	#[test]
	fn example_327_in_html() {
		// example 327
		test(
			r##"
				<a href="&ouml;&ouml;.html">
			"##,
			r##"
				<a href="&ouml;&ouml;.html">
			"##,
		);
	}

	#[test]
	fn example_328_in_link() {
		// example 328
		test(
			r##"
				[foo](/f&ouml;&ouml; "f&ouml;&ouml;")
			"##,
			r##"
				<p><a href="/föö" title="föö">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_329_in_link_reference() {
		// example 329
		test(
			r##"
				[foo]

				[foo]: /f&ouml;&ouml; "f&ouml;&ouml;"
			"##,
			r##"
				<p><a href="/föö" title="föö">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_330_in_info_string() {
		// example 330
		test(
			r##"
				``` f&ouml;&ouml;
				foo
				```
			"##,
			r##"
				<pre><code data-info="föö">foo
				</code></pre>
			"##,
		);
	}

	#[test]
	fn example_331_literal_text_in_code_spans() {
		// example 331
		test(
			r##"
				`f&ouml;&ouml;`
			"##,
			r##"
				<p><code>f&amp;ouml;&amp;ouml;</code></p>
			"##,
		);
	}

	#[test]
	fn example_332_literal_text_in_code_block() {
		// example 332
		test_raw_in(
			"    f&ouml;f&ouml;",
			r##"
				<pre><code>f&amp;ouml;f&amp;ouml;</code></pre>
			"##,
		);

		test(
			r##"
				```
				f&ouml;f&ouml;
				```
			"##,
			r##"
				<pre><code>f&amp;ouml;f&amp;ouml;
				</code></pre>
			"##,
		);
	}

	#[test]
	fn example_333_are_textual_elements() {
		// example 333
		test(
			r##"
				&#42;foo&#42;
				*foo*
			"##,
			r##"
				<p>*foo*
				<em>foo</em></p>
			"##,
		);
	}

	#[test]
	fn example_334_are_textual_elements() {
		// example 334
		test(
			r##"
				&#42; foo

				* foo
			"##,
			r##"
				<p>* foo</p>
				<ul>
				<li>foo</li>
				</ul>
			"##,
		);
	}

	#[test]
	fn example_335_are_textual_elements() {
		// example 335
		test(
			r##"
				foo&#10;&#10;bar
			"##,
			r##"
				<p>foo

				bar</p>
			"##,
		);
	}

	#[test]
	fn example_336_are_textual_elements() {
		// example 336
		test(
			r##"
				&#9;foo
			"##,
			r##"
				<p>	foo</p>
			"##,
		);
	}

	#[test]
	fn example_337_are_textual_elements() {
		// example 337
		test(
			r##"
				[a](url &quot;tit&quot;)
			"##,
			r##"
				<p>[a](url &quot;tit&quot;)</p>
			"##,
		);
	}
}
