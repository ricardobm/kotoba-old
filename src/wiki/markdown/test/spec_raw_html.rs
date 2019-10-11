use super::*;

// spell-checker: disable

mod markdown_spec_raw_html {
	use super::*;

	#[test]
	fn example_632_open_tags() {
		// example 632
		test(
			r##"
				<a><bab><c2c>
			"##,
			r##"
				<p><a><bab><c2c></p>
			"##,
		);
	}

	#[test]
	fn example_633_empty_elements() {
		// example 633
		test(
			r##"
				<a/><b2/>
			"##,
			r##"
				<p><a/><b2/></p>
			"##,
		);
	}

	#[test]
	fn example_634_whitespace() {
		// example 634
		test(
			r##"
				<a  /><b2
				data="foo" >
			"##,
			r##"
				<p><a  /><b2
				data="foo" ></p>
			"##,
		);
	}

	#[test]
	fn example_635_attributes() {
		// example 635
		test(
			r##"
				<a foo="bar" bam = 'baz <em>"</em>'
				_boolean zoop:33=zoop:33 />
			"##,
			r##"
				<p><a foo="bar" bam = 'baz <em>"</em>'
				_boolean zoop:33=zoop:33 /></p>
			"##,
		);
	}

	#[test]
	fn example_636_custom_tag_names() {
		// example 636
		test(
			r##"
				Foo <responsive-image src="foo.jpg" />
			"##,
			r##"
				<p>Foo <responsive-image src="foo.jpg" /></p>
			"##,
		);
	}

	#[test]
	fn example_637_illegal_tag_name() {
		// example 637
		test(
			r##"
				<33> <__>
			"##,
			r##"
				<p>&lt;33&gt; &lt;__&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_638_illegal_attribute_names() {
		// example 638
		test(
			r##"
				<a h*#ref="hi">
			"##,
			r##"
				<p>&lt;a h*#ref=&quot;hi&quot;&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_639_illegal_attribute_values() {
		// example 639
		test(
			r##"
				<a href="hi'> <a href=hi'>
			"##,
			r##"
				<p>&lt;a href=&quot;hi&apos;&gt; &lt;a href=hi&apos;&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_640_illegal_whitespace() {
		// example 640
		test(
			r##"
				< a><
				foo><bar/ >
				<foo bar=baz
				bim!bop />
			"##,
			r##"
				<p>&lt; a&gt;&lt;
				foo&gt;&lt;bar/ &gt;
				&lt;foo bar=baz
				bim!bop /&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_641_missing_whitespace() {
		// example 641
		test(
			r##"
				<a href='bar'title=title>
			"##,
			r##"
				<p>&lt;a href=&apos;bar&apos;title=title&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_642_closing_tags() {
		// example 642
		test(
			r##"
				</a></foo >
			"##,
			r##"
				<p></a></foo ></p>
			"##,
		);
	}

	#[test]
	fn example_643_illegal_attributes_in_closing_tags() {
		// example 643
		test(
			r##"
				</a href="foo">
			"##,
			r##"
				<p>&lt;/a href=&quot;foo&quot;&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_644_comments() {
		// example 644
		test(
			r##"
				foo <!-- this is a
				comment - with hyphen -->
			"##,
			r##"
				<p>foo <!-- this is a
				comment - with hyphen --></p>
			"##,
		);
	}

	#[test]
	fn example_645_not_a_comment() {
		// example 645
		test(
			r##"
				foo <!-- not a comment -- two hyphens -->
			"##,
			r##"
				<p>foo &lt;!-- not a comment -- two hyphens --&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_646_not_comments() {
		// example 646
		test(
			r##"
				foo <!--> foo -->

				foo <!-- foo--->
			"##,
			r##"
				<p>foo &lt;!--&gt; foo --&gt;</p>
				<p>foo &lt;!-- foo---&gt;</p>
			"##,
		);
	}

	#[test]
	fn example_647_processing_instruction() {
		// example 647
		test(
			r##"
				foo <?php echo $a; ?>
			"##,
			r##"
				<p>foo <?php echo $a; ?></p>
			"##,
		);
	}

	#[test]
	fn example_648_declarations() {
		// example 648
		test(
			r##"
				foo <!ELEMENT br EMPTY>
			"##,
			r##"
				<p>foo <!ELEMENT br EMPTY></p>
			"##,
		);
	}

	#[test]
	fn example_649_cdata() {
		// example 649
		test(
			r##"
				foo <![CDATA[>&<]]>
			"##,
			r##"
				<p>foo <![CDATA[>&<]]></p>
			"##,
		);
	}

	#[test]
	fn example_650_entities_are_preserved() {
		// example 650
		test(
			r##"
				foo <a href="&ouml;">
			"##,
			r##"
				<p>foo <a href="&ouml;"></p>
			"##,
		);
	}

	#[test]
	fn example_651_backslash_escapes_do_not_work() {
		// example 651
		test(
			r##"
				foo <a href="\*">
			"##,
			r##"
				<p>foo <a href="\*"></p>
			"##,
		);
	}

	#[test]
	fn example_652_backslash_escapes_do_not_work() {
		// example 652
		test(
			r##"
				<a href="\"">
			"##,
			r##"
				<p>&lt;a href=&quot;&quot;&quot;&gt;</p>
			"##,
		);
	}
}
