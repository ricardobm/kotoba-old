use super::*;

// spell-checker: disable

mod markdown_spec_images {
	use super::*;

	#[test]
	fn example_580_should_parse() {
		// example 580
		test(
			r##"
				![foo](/url "title")
			"##,
			r##"
				<p><img src="/url" alt="foo" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_581_should_parse() {
		// example 581
		test(
			r##"
				![foo *bar*]

				[foo *bar*]: train.jpg "train & tracks"
			"##,
			r##"
				<p><img src="train.jpg" alt="foo bar" title="train &amp; tracks"/></p>
			"##,
		);
	}

	#[test]
	fn example_582_should_parse() {
		// example 582
		test(
			r##"
				![foo ![bar](/url)](/url2)
			"##,
			r##"
				<p><img src="/url2" alt="foo bar"/></p>
			"##,
		);
	}

	#[test]
	fn example_583_should_parse() {
		// example 583
		test(
			r##"
				![foo [bar](/url)](/url2)
			"##,
			r##"
				<p><img src="/url2" alt="foo bar"/></p>
			"##,
		);
	}

	#[test]
	fn example_584_alt_text() {
		// example 584
		test(
			r##"
				![foo *bar*][]

				[foo *bar*]: train.jpg "train & tracks"
			"##,
			r##"
				<p><img src="train.jpg" alt="foo bar" title="train &amp; tracks"/></p>
			"##,
		);
	}

	#[test]
	fn example_585_alt_text() {
		// example 585
		test(
			r##"
				![foo *bar*][foobar]

				[FOOBAR]: train.jpg "train & tracks"
			"##,
			r##"
				<p><img src="train.jpg" alt="foo bar" title="train &amp; tracks"/></p>
			"##,
		);
	}

	#[test]
	fn example_586_alt_text() {
		// example 586
		test(
			r##"
				![foo](train.jpg)
			"##,
			r##"
				<p><img src="train.jpg" alt="foo"/></p>
			"##,
		);
	}

	#[test]
	fn example_587_spaces() {
		// example 587
		test(
			r##"
				My ![foo bar](/path/to/train.jpg  "title"   )
			"##,
			r##"
				<p>My <img src="/path/to/train.jpg" alt="foo bar" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_588_pointy_brackets() {
		// example 588
		test(
			r##"
				![foo](<url>)
			"##,
			r##"
				<p><img src="url" alt="foo"/></p>
			"##,
		);
	}

	#[test]
	fn example_589_no_alt_text() {
		// example 589
		test(
			r##"
				![](/url)
			"##,
			r##"
				<p><img src="/url" alt=""/></p>
			"##,
		);
	}

	#[test]
	fn example_590_reference_style() {
		// example 590
		test(
			r##"
				![foo][bar]

				[bar]: /url
			"##,
			r##"
				<p><img src="/url" alt="foo"/></p>
			"##,
		);
	}

	#[test]
	fn example_591_reference_style() {
		// example 591
		test(
			r##"
				![foo][bar]

				[BAR]: /url
			"##,
			r##"
				<p><img src="/url" alt="foo"/></p>
			"##,
		);
	}

	#[test]
	fn example_592_collapsed() {
		// example 592
		test(
			r##"
				![foo][]

				[foo]: /url "title"
			"##,
			r##"
				<p><img src="/url" alt="foo" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_593_collapsed() {
		// example 593
		test(
			r##"
				![*foo* bar][]

				[*foo* bar]: /url "title"
			"##,
			r##"
				<p><img src="/url" alt="foo bar" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_594_case_insensitive_reference_labels() {
		// example 594
		test(
			r##"
				![Foo][]

				[foo]: /url "title"
			"##,
			r##"
				<p><img src="/url" alt="Foo" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_595_spaces_not_allowed() {
		// example 595
		test(
			r##"
				![foo]
				[]

				[foo]: /url "title"
			"##,
			r##"
				<p><img src="/url" alt="foo" title="title"/>
				[]</p>
			"##,
		);
	}

	#[test]
	fn example_596_shortcut() {
		// example 596
		test(
			r##"
				![foo]

				[foo]: /url "title"
			"##,
			r##"
				<p><img src="/url" alt="foo" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_597_shortcut() {
		// example 597
		test(
			r##"
				![*foo* bar]

				[*foo* bar]: /url "title"
			"##,
			r##"
				<p><img src="/url" alt="foo bar" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_598_cannot_contain_unescaped_brackets() {
		// example 598
		test(
			r##"
				![[foo]]

				[[foo]]: /url "title"
			"##,
			r##"
				<p>![[foo]]</p>
				<p>[[foo]]: /url &quot;title&quot;</p>
			"##,
		);
	}

	#[test]
	fn example_599_case_insensitive() {
		// example 599
		test(
			r##"
				![Foo]

				[foo]: /url "title"
			"##,
			r##"
				<p><img src="/url" alt="Foo" title="title"/></p>
			"##,
		);
	}

	#[test]
	fn example_600_escaped() {
		// example 600
		test(
			r##"
				!\[foo]

				[foo]: /url "title"
			"##,
			r##"
				<p>![foo]</p>
			"##,
		);
	}

	#[test]
	fn example_601_escaped_link() {
		// example 601
		test(
			r##"
				\![foo]

				[foo]: /url "title"
			"##,
			r##"
				<p>!<a href="/url" title="title">foo</a></p>
			"##,
		);
	}
}
