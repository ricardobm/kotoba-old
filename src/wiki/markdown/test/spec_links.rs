use super::*;

// spell-checker: disable

mod markdown_spec_links {
	use super::*;

	#[test]
	fn example_493_simple() {
		// example 493
		test(
			r##"
				[link](/uri "title")
			"##,
			r##"
				<p><a href="/uri" title="title">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_494_no_title() {
		// example 494
		test(
			r##"
				[link](/uri)
			"##,
			r##"
				<p><a href="/uri">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_495_no_title_or_destination() {
		// example 495
		test(
			r##"
				[link]()
			"##,
			r##"
				<p><a href="">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_496_no_title_or_destination() {
		// example 496
		test(
			r##"
				[link](<>)
			"##,
			r##"
				<p><a href="">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_497_spaces_only_in_enclosed_destination() {
		// example 497
		test(
			r##"
				[link](/my uri)
			"##,
			r##"
				<p>[link](/my uri)</p>
			"##,
		);
	}

	#[test]
	fn example_498_spaces_only_in_enclosed_destination() {
		// example 498
		test(
			r##"
				[link](</my uri>)
			"##,
			r##"
				<p><a href="/my uri">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_499_cannot_contain_line_breaks() {
		// example 499
		test(
			r##"
				[link](foo
				bar)
			"##,
			r##"
				<p>[link](foo
				bar)</p>
			"##,
		);
	}

	#[test]
	fn example_500_cannot_contain_line_breaks() {
		// example 500
		test(
			r##"
				[link](<foo
				bar>)
			"##,
			r##"
				<p>[link](<foo
				bar>)</p>
			"##,
		);
	}

	#[test]
	fn example_501_can_contain_parenthesis_if_enclosed() {
		// example 501
		test(
			r##"
				[a](<b)c>)
			"##,
			r##"
				<p><a href="b)c">a</a></p>
			"##,
		);
	}

	#[test]
	fn example_502_pointy_brackets_must_be_unescaped() {
		// example 502
		test(
			r##"
				[link](<foo\>)
			"##,
			r##"
				<p>[link](&lt;foo&gt;)</p>
			"##,
		);
	}

	#[test]
	fn example_503_not_links_unmatched_pointy_brackets() {
		// example 503
		test(
			r##"
				[a](<b)c
				[a](<b)c>
				[a](<b>c)
			"##,
			r##"
				<p>[a](&lt;b)c
				[a](&lt;b)c&gt;
				[a](<b>c)</p>
			"##,
		);
	}

	#[test]
	fn example_504_parenthesis_in_destination_must_be_escaped() {
		// example 504
		test(
			r##"
				[link](\(foo\))
			"##,
			r##"
				<p><a href="(foo)">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_505_parenthesis_are_allowed_if_balanced() {
		// example 505
		test(
			r##"
				[link](foo(and(bar)))
			"##,
			r##"
				<p><a href="foo(and(bar))">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_506_unbalanced_parenthesis_must_be_escaped() {
		// example 506
		test(
			r##"
				[link](foo\(and\(bar\))
			"##,
			r##"
				<p><a href="foo(and(bar)">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_507_unbalanced_parenthesis_must_be_escaped() {
		// example 507
		test(
			r##"
				[link](<foo(and(bar)>)
			"##,
			r##"
				<p><a href="foo(and(bar)">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_508_allow_backslash_escapes() {
		// example 508
		test(
			r##"
				[link](foo\)\:)
			"##,
			r##"
				<p><a href="foo):">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_509_can_contain_fragment_identifiers() {
		// example 509
		test(
			r##"
				[link](#fragment)

				[link](http://example.com#fragment)

				[link](http://example.com?foo=3#frag)
			"##,
			r##"
				<p><a href="#fragment">link</a></p>
				<p><a href="http://example.com#fragment">link</a></p>
				<p><a href="http://example.com?foo=3#frag">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_510_unrecognized_backslash_is_literal() {
		// example 510
		test(
			r##"
				[link](foo\bar)
			"##,
			r##"
				<p><a href="foo\bar">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_511_should_allow_url_and_html_escaping() {
		// example 511
		test(
			r##"
				[link](foo%20b&auml;)
			"##,
			r##"
				<p><a href="foo%20bä">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_512_title_is_destination() {
		// example 512
		test(
			r##"
				[link]("title")
			"##,
			r##"
				<p><a href="&quot;title&quot;">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_513_title_delimiters() {
		// example 513
		test(
			r##"
				[link](/url "title")
				[link](/url 'title')
				[link](/url (title))
			"##,
			r##"
				<p><a href="/url" title="title">link</a>
				<a href="/url" title="title">link</a>
				<a href="/url" title="title">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_514_escapes_and_entities_in_titles() {
		// example 514
		test(
			r##"
				[link](/url "title \"&quot;")
			"##,
			r##"
				<p><a href="/url" title="title &quot;&quot;">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_515_titles_must_be_separated_by_non_unicode_space() {
		// example 515
		test_raw(
			"[link](/url\u{00A0}\"title\")",
			"<p><a href=\"/url\u{00A0}&quot;title&quot;\">link</a></p>",
		);
	}

	#[test]
	fn example_516_nested_balanced_quotes_are_not_allowed() {
		// example 516
		test(
			r##"
				[link](/url "title "and" title")
			"##,
			r##"
				<p>[link](/url &quot;title &quot;and&quot; title&quot;)</p>
			"##,
		);
	}

	#[test]
	fn example_517_quotes_in_quotes() {
		// example 517
		test(
			r##"
				[link](/url 'title "and" title')
			"##,
			r##"
				<p><a href="/url" title="title &quot;and&quot; title">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_518_allowed_whitespace() {
		// example 518
		test_raw_in(
			"[link](   /uri\n  \"title\"  )",
			r##"
				<p><a href="/uri" title="title">link</a></p>
			"##,
		);
	}

	#[test]
	fn example_519_non_allowed_whitespace() {
		// example 519
		test(
			r##"
				[link] (/uri)
			"##,
			r##"
				<p>[link] (/uri)</p>
			"##,
		);
	}

	#[test]
	fn example_520_may_contain_balanced_brackets() {
		// example 520
		test(
			r##"
				[link [foo [bar]]](/uri)
			"##,
			r##"
				<p><a href="/uri">link [foo [bar]]</a></p>
			"##,
		);
	}

	#[test]
	fn example_521_may_contain_balanced_brackets() {
		// example 521
		test(
			r##"
				[link] bar](/uri)
			"##,
			r##"
				<p>[link] bar](/uri)</p>
			"##,
		);
	}

	#[test]
	fn example_522_may_contain_balanced_brackets() {
		// example 522
		test(
			r##"
				[link [bar](/uri)
			"##,
			r##"
				<p>[link <a href="/uri">bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_523_may_contain_balanced_brackets() {
		// example 523
		test(
			r##"
				[link \[bar](/uri)
			"##,
			r##"
				<p><a href="/uri">link [bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_524_link_text_can_contain_inlines() {
		// example 524
		test(
			r##"
				[link *foo **bar** `#`*](/uri)
			"##,
			r##"
				<p><a href="/uri">link <em>foo <strong>bar</strong> <code>#</code></em></a></p>
			"##,
		);
	}

	#[test]
	fn example_525_link_text_can_contain_inlines() {
		// example 525
		test(
			r##"
				[![moon](moon.jpg)](/uri)
			"##,
			r##"
				<p><a href="/uri"><img src="moon.jpg" alt="moon"/></a></p>
			"##,
		);
	}

	#[test]
	fn example_526_may_not_contain_other_links() {
		// example 526
		test(
			r##"
				[foo [bar](/uri)](/uri)
			"##,
			r##"
				<p>[foo <a href="/uri">bar</a>](/uri)</p>
			"##,
		);
	}

	#[test]
	fn example_527_may_not_contain_other_links() {
		// example 527
		test(
			r##"
				[foo *[bar [baz](/uri)](/uri)*](/uri)
			"##,
			r##"
				<p>[foo <em>[bar <a href="/uri">baz</a>](/uri)</em>](/uri)</p>
			"##,
		);
	}

	#[test]
	fn example_528_may_not_contain_other_links() {
		// example 528
		test(
			r##"
				![[[foo](uri1)](uri2)](uri3)
			"##,
			r##"
				<p><img src="uri3" alt="[foo](uri2)"/></p>
			"##,
		);
	}

	#[test]
	fn example_529_has_precedence_over_emphasis() {
		// example 529
		test(
			r##"
				*[foo*](/uri)
			"##,
			r##"
				<p>*<a href="/uri">foo*</a></p>
			"##,
		);
	}

	#[test]
	fn example_530_has_precedence_over_emphasis() {
		// example 530
		test(
			r##"
				[foo *bar](baz*)
			"##,
			r##"
				<p><a href="baz*">foo *bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_531_only_brackets_in_links_have_precedence() {
		// example 531
		test(
			r##"
				*foo [bar* baz]
			"##,
			r##"
				<p><em>foo [bar</em> baz]</p>
			"##,
		);
	}

	#[test]
	fn example_532_has_less_precedence_than_tags() {
		// example 532
		test(
			r##"
				[foo <bar attr="](baz)">
			"##,
			r##"
				<p>[foo <bar attr="](baz)"></p>
			"##,
		);
	}

	#[test]
	fn example_533_has_less_precedence_than_code_spans() {
		// example 533
		test(
			r##"
				[foo`](/uri)`
			"##,
			r##"
				<p>[foo<code>](/uri)</code></p>
			"##,
		);
	}

	#[test]
	fn example_534_has_less_precedence_than_autolinks() {
		// example 534
		test(
			r##"
				[foo<http://example.com/?search=](uri)>
			"##,
			r##"
				<p>[foo<a href="http://example.com/?search=](uri)">http://example.com/?search=](uri)</a></p>
			"##,
		);
	}

	#[test]
	fn example_535_reference_link() {
		// example 535
		test(
			r##"
				[foo][bar]

				[bar]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_536_reference_link_balanced_brackets() {
		// example 536
		test(
			r##"
				[link [foo [bar]]][ref]

				[ref]: /uri
			"##,
			r##"
				<p><a href="/uri">link [foo [bar]]</a></p>
			"##,
		);
	}

	#[test]
	fn example_537_reference_link_balanced_brackets() {
		// example 537
		test(
			r##"
				[link \[bar][ref]

				[ref]: /uri
			"##,
			r##"
				<p><a href="/uri">link [bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_538_reference_link_inline_content() {
		// example 538
		test(
			r##"
				[link *foo **bar** `#`*][ref]

				[ref]: /uri
			"##,
			r##"
				<p><a href="/uri">link <em>foo <strong>bar</strong> <code>#</code></em></a></p>
			"##,
		);
	}

	#[test]
	fn example_539_reference_link_inline_content() {
		// example 539
		test(
			r##"
				[![moon](moon.jpg)][ref]

				[ref]: /uri
			"##,
			r##"
				<p><a href="/uri"><img src="moon.jpg" alt="moon"/></a></p>
			"##,
		);
	}

	#[test]
	fn example_540_reference_link_cannot_contain_links() {
		// example 540
		test(
			r##"
				[foo [bar](/uri)][ref]

				[ref]: /uri
			"##,
			r##"
				<p>[foo <a href="/uri">bar</a>]<a href="/uri">ref</a></p>
			"##,
		);
	}

	#[test]
	fn example_541_reference_link_cannot_contain_links() {
		// example 541
		test(
			r##"
				[foo *bar [baz][ref]*][ref]

				[ref]: /uri
			"##,
			r##"
				<p>[foo <em>bar <a href="/uri">baz</a></em>]<a href="/uri">ref</a></p>
			"##,
		);
	}

	#[test]
	fn example_542_reference_link_precedence_over_emphasis() {
		// example 542
		test(
			r##"
				*[foo*][ref]

				[ref]: /uri
			"##,
			r##"
				<p>*<a href="/uri">foo*</a></p>
			"##,
		);
	}

	#[test]
	fn example_543_reference_link_precedence_over_emphasis() {
		// example 543
		test(
			r##"
				[foo *bar][ref]*

				[ref]: /uri
			"##,
			r##"
				<p><a href="/uri">foo *bar</a>*</p>
			"##,
		);
	}

	#[test]
	fn example_544_reference_link_has_less_precedence_than_tags() {
		// example 544
		test(
			r##"
				[foo <bar attr="][ref]">

				[ref]: /uri
			"##,
			r##"
				<p>[foo <bar attr="][ref]"></p>
			"##,
		);
	}

	#[test]
	fn example_545_reference_link_has_less_precedence_than_code_spans() {
		// example 545
		test(
			r##"
				[foo`][ref]`

				[ref]: /uri
			"##,
			r##"
				<p>[foo<code>][ref]</code></p>
			"##,
		);
	}

	#[test]
	fn example_546_reference_link_has_less_precedence_than_autolinks() {
		// example 546
		test(
			r##"
				[foo<http://example.com/?search=][ref]>

				[ref]: /uri
			"##,
			r##"
				<p>[foo<a href="http://example.com/?search=][ref]">http://example.com/?search=][ref]</a></p>
			"##,
		);
	}

	#[test]
	fn example_547_reference_link_matching_is_case_insensitive() {
		// example 547
		test(
			r##"
				[foo][BaR]

				[bar]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_548_reference_link_matching_case_folds() {
		// example 548
		test(
			r##"
				[ẞ]

				[SS]: /url
			"##,
			r##"
				<p><a href="/url">ẞ</a></p>
			"##,
		);
	}

	#[test]
	fn example_549_reference_link_matching_collapse_spaces() {
		// example 549
		test(
			r##"
				[Foo
				  bar]: /url

				[Baz][Foo bar]
			"##,
			r##"
				<p><a href="/url">Baz</a></p>
			"##,
		);
	}

	#[test]
	fn example_550_reference_link_unallowed_spaces() {
		// example 550
		test(
			r##"
				[foo] [bar]

				[bar]: /url "title"
			"##,
			r##"
				<p>[foo] <a href="/url" title="title">bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_551_reference_link_unallowed_spaces() {
		// example 551
		test(
			r##"
				[foo]
				[bar]

				[bar]: /url "title"
			"##,
			r##"
				<p>[foo]
				<a href="/url" title="title">bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_552_reference_link_uses_first_definition() {
		// example 552
		test(
			r##"
				[foo]: /url1

				[foo]: /url2

				[bar][foo]
			"##,
			r##"
				<p><a href="/url1">bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_553_reference_link_matching_is_unparsed() {
		// example 553
		test(
			r##"
				[bar][foo\!]

				[foo!]: /url
			"##,
			r##"
				<p>[bar][foo!]</p>
			"##,
		);
	}

	#[test]
	fn example_554_reference_link_can_contain_escaped_brackets_only() {
		// example 554
		test(
			r##"
				[foo][ref[]

				[ref[]: /uri
			"##,
			r##"
				<p>[foo][ref[]</p>
				<p>[ref[]: /uri</p>
			"##,
		);
	}

	#[test]
	fn example_555_reference_link_can_contain_escaped_brackets_only() {
		// example 555
		test(
			r##"
				[foo][ref[bar]]

				[ref[bar]]: /uri
			"##,
			r##"
				<p>[foo][ref[bar]]</p>
				<p>[ref[bar]]: /uri</p>
			"##,
		);
	}

	#[test]
	fn example_556_reference_link_can_contain_escaped_brackets_only() {
		// example 556
		test(
			r##"
				[[[foo]]]

				[[[foo]]]: /url
			"##,
			r##"
				<p>[[[foo]]]</p>
				<p>[[[foo]]]: /url</p>
			"##,
		);
	}

	#[test]
	fn example_557_reference_link_can_contain_escaped_brackets_only() {
		// example 557
		test(
			r##"
				[foo][ref\[]

				[ref\[]: /uri
			"##,
			r##"
				<p><a href="/uri">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_558_reference_link_can_contain_escaped_brackets_only() {
		// example 558
		test(
			r##"
				[bar\\]: /uri

				[bar\\]
			"##,
			r##"
				<p><a href="/uri">bar\</a></p>
			"##,
		);
	}

	#[test]
	fn example_559_label_must_contain_non_whitespace_characters() {
		// example 559
		test(
			r##"
				[]

				[]: /uri
			"##,
			r##"
				<p>[]</p>
				<p>[]: /uri</p>
			"##,
		);
	}

	#[test]
	fn example_560_must_contain_non_whitespace_characters() {
		// example 560
		test(
			r##"
				[
				 ]

				[
				 ]: /uri
			"##,
			r##"
				<p>[
				]</p>
				<p>[
				]: /uri</p>
			"##,
		);
	}

	#[test]
	fn example_561_collapsed_reference_link() {
		// example 561
		test(
			r##"
				[foo][]

				[foo]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_562_collapsed_reference_link() {
		// example 562
		test(
			r##"
				[*foo* bar][]

				[*foo* bar]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title"><em>foo</em> bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_563_collapsed_reference_link_case_insensitive() {
		// example 563
		test(
			r##"
				[Foo][]

				[foo]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title">Foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_564_collapsed_reference_link_non_allowed_whitespace() {
		// example 564
		test(
			r##"
				[foo]
				[]

				[foo]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title">foo</a>
				[]</p>
			"##,
		);
	}

	#[test]
	fn example_565_shortcut_reference_link() {
		// example 565
		test(
			r##"
				[foo]

				[foo]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_566_shortcut_reference_link() {
		// example 566
		test(
			r##"
				[*foo* bar]

				[*foo* bar]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title"><em>foo</em> bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_567_shortcut_reference_link() {
		// example 567
		test(
			r##"
				[[*foo* bar]]

				[*foo* bar]: /url "title"
			"##,
			r##"
				<p>[<a href="/url" title="title"><em>foo</em> bar</a>]</p>
			"##,
		);
	}

	#[test]
	fn example_568_shortcut_reference_link() {
		// example 568
		test(
			r##"
				[[bar [foo]

				[foo]: /url
			"##,
			r##"
				<p>[[bar <a href="/url">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_569_shortcut_reference_link_case_insensitive() {
		// example 569
		test(
			r##"
				[Foo]

				[foo]: /url "title"
			"##,
			r##"
				<p><a href="/url" title="title">Foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_570_shortcut_reference_link_space_after_is_preserved() {
		// example 570
		test(
			r##"
				[foo] bar

				[foo]: /url
			"##,
			r##"
				<p><a href="/url">foo</a> bar</p>
			"##,
		);
	}

	#[test]
	fn example_571_shortcut_reference_link_escaping() {
		// example 571
		test(
			r##"
				\[foo]

				[foo]: /url "title"
			"##,
			r##"
				<p>[foo]</p>
			"##,
		);
	}

	#[test]
	fn example_572_shortcut_reference_link_is_a_link() {
		// example 572
		test(
			r##"
				[foo*]: /url

				*[foo*]
			"##,
			r##"
				<p>*<a href="/url">foo*</a></p>
			"##,
		);
	}

	#[test]
	fn example_573_full_references_take_precedence_over_shortcut() {
		// example 573
		test(
			r##"
				[foo][bar]

				[foo]: /url1
				[bar]: /url2
			"##,
			r##"
				<p><a href="/url2">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_574_compact_references_take_precedence_over_shortcut() {
		// example 574
		test(
			r##"
				[foo][]

				[foo]: /url1
			"##,
			r##"
				<p><a href="/url1">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_575_inline_links_take_precedence_over_shortcut() {
		// example 575
		test(
			r##"
				[foo]()

				[foo]: /url1
			"##,
			r##"
				<p><a href="">foo</a></p>
			"##,
		);
	}

	#[test]
	fn example_576_inline_links_take_precedence_over_shortcut() {
		// example 576
		test(
			r##"
				[foo](not a link)

				[foo]: /url1
			"##,
			r##"
				<p><a href="/url1">foo</a>(not a link)</p>
			"##,
		);
	}

	#[test]
	fn example_577_tricky_shortcut_parsing() {
		// example 577
		test(
			r##"
				[foo][bar][baz]

				[baz]: /url
			"##,
			r##"
				<p>[foo]<a href="/url">bar</a></p>
			"##,
		);
	}

	#[test]
	fn example_578_tricky_shortcut_parsing() {
		// example 578
		test(
			r##"
				[foo][bar][baz]

				[baz]: /url1
				[bar]: /url2
			"##,
			r##"
				<p><a href="/url2">foo</a><a href="/url1">baz</a></p>
			"##,
		);
	}

	#[test]
	fn example_579() {
		// example 579
		test(
			r##"
				[foo][bar][baz]

				[baz]: /url1
				[foo]: /url2
			"##,
			r##"
				<p>[foo]<a href="/url1">bar</a></p>
			"##,
		);
	}
}
