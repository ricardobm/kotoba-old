use speculate::speculate;

use super::*;

speculate! { describe "markdown images" {

it "should parse basic" {
	// basic
	test(r#"
		![foo](/url "title")
	"#, r#"
		<p><img src="/url" alt="foo" title="title"/></p>
	"#);

	// no title
	test(r#"
		![foo](train.jpg)
	"#, r#"
		<p><img src="train.jpg" alt="foo"/></p>
	"#);

	// <URL> syntax
	test(r#"
		![foo](<url>)
	"#, r#"
		<p><img src="url" alt="foo"/></p>
	"#);

	// no alt text
	test(r#"
		![](/url)
	"#, r#"
		<p><img src="/url" alt=""/></p>
	"#);

	// in paragraph
	test(r#"
		My ![foo bar](/path/to/train.jpg  "title"   )
	"#, r#"
		<p>My <img src="/path/to/train.jpg" alt="foo bar" title="title"/></p>
	"#);
}

it "should parse with reference" {
	test(r#"
		![foo][bar]

		[bar]: /url
	"#, r#"
		<p><img src="/url" alt="foo"/></p>
	"#);

	test(r#"
		![foo][bar]

		[BAR]: /url
	"#, r#"
		<p><img src="/url" alt="foo"/></p>
	"#);

	test(r#"
		![foo][]

		[foo]: /url "title"
	"#, r#"
		<p><img src="/url" alt="foo" title="title"/></p>
	"#);

	test(r#"
		![*foo* bar][]

		[*foo* bar]: /url "title"
	"#, r#"
		<p><img src="/url" alt="foo bar" title="title"/></p>
	"#);

	test(r#"
		![Foo][]

		[foo]: /url "title"
	"#, r#"
		<p><img src="/url" alt="Foo" title="title"/></p>
	"#);

	test(r#"
		![foo]
		[]

		[foo]: /url "title"
	"#, r#"
		<p><img src="/url" alt="foo" title="title"/>
		[]</p>
	"#);

	test(r#"
		![foo]

		[foo]: /url "title"
	"#, r#"
		<p><img src="/url" alt="foo" title="title"/></p>
	"#);

	test(r#"
		![*foo* bar]

		[*foo* bar]: /url "title"
	"#, r#"
		<p><img src="/url" alt="foo bar" title="title"/></p>
	"#);

	test(r#"
		![[foo]]

		[[foo]]: /url "title"
	"#, r#"
		<p>![[foo]]</p>
		<p>[[foo]]: /url &quot;title&quot;</p>
	"#);

	test(r#"
		!\[foo]

		[foo]: /url "title"
	"#, r#"
		<p>![foo]</p>
	"#);

	test(r#"
		\![foo]

		[foo]: /url "title"
	"#, r#"
		<p>!<a href="/url" title="title">foo</a></p>
	"#);

	test(r#"
		![foo *bar*]

		[foo *bar*]: train.jpg "train & tracks"
	"#, r#"
		<p><img src="train.jpg" alt="foo bar" title="train &amp; tracks"/></p>
	"#);

	test(r#"
		![foo *bar*][]

		[foo *bar*]: train.jpg "train & tracks"
	"#, r#"
		<p><img src="train.jpg" alt="foo bar" title="train &amp; tracks"/></p>
	"#);

	test(r#"
		![foo *bar*][foobar]

		[FOOBAR]: train.jpg "train & tracks"
	"#, r#"
		<p><img src="train.jpg" alt="foo bar" title="train &amp; tracks"/></p>
	"#);
}

it "can contain links and images" {
	test(r#"
		![foo ![bar](/url)](/url2)
	"#, r#"
		<p><img src="/url2" alt="foo bar"/></p>
	"#);

	test(r#"
		![foo [bar](/url)](/url2)
	"#, r#"
		<p><img src="/url2" alt="foo bar"/></p>
	"#);
}

}}
