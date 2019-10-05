use speculate::speculate;

use super::*;

speculate! { describe "markdown links" {

it "should parse inline" {
	test(r#"
		[link](/uri "title")
	"#, r#"
		<p><a href="/uri" title="title">link</a></p>
	"#);
}

it "both title and destination may be omitted" {
	test(r#"
		[link](/uri)
	"#, r#"
		<p><a href="/uri">link</a></p>
	"#);

	test(r#"
		[link]()
	"#, r#"
		<p><a href="">link</a></p>
	"#);

	test(r#"
		[link](<>)
	"#, r#"
		<p><a href="">link</a></p>
	"#);

	test(r#"
		[link]("title")
	"#, r#"
		<p><a href="&quot;title&quot;">link</a></p>
	"#);
}

it "destination can contain spaces only within brackets" {
	test(r#"
		[link](/my uri)

		[link](</my uri>)
	"#, r#"
		<p>[link](/my uri)</p>
		<p><a href="/my uri">link</a></p>
	"#);
}

#[ignore = "TODO: need support for tags"]
it "destination cannot contain line breaks" {
	test(r#"
		[link](foo
		bar)

		[link](<foo
		bar>)
	"#, r#"
		<p>[link](foo
		bar)</p>
		<p>[link](<foo
		bar>)</p>
	"#);

}

it "destination can contain parenthesis within brackets" {
	test(r#"
		[a](<b)c>)
	"#, r#"
		<p><a href="b)c">a</a></p>
	"#);

}

#[allow(unreachable_code)]
it "pointy brackets must match" {
	test(r#"
		[link](<foo\>)
	"#, r#"
		<p>[link](&lt;foo&gt;)</p>
	"#);

	// TODO: enable me when we support raw HTML
	return;

	test(r#"
		[a](<b)c
		[a](<b)c>
		[a](<b>c)
	"#, r#"
		<p>[a](&lt;b)c
		[a](&lt;b)c&gt;
		[a](<b>c)</p>
	"#);
}

it "parenthesis in destination may be escaped" {
	test(r#"
		[link](\(foo\))
	"#, r#"
		<p><a href="(foo)">link</a></p>
	"#);
}

it "balanced parenthesis in destination are allowed" {
	test(r#"
		[link](foo(and(bar)))
	"#, r#"
		<p><a href="foo(and(bar))">link</a></p>
	"#);
}

it "unbalanced parenthesis in destination must be escaped" {
	test(r#"
		[link](foo\(and\(bar\))
	"#, r#"
		<p><a href="foo(and(bar)">link</a></p>
	"#);

	test(r#"
		[link](<foo(and(bar)>)
	"#, r#"
		<p><a href="foo(and(bar)">link</a></p>
	"#);
}

it "symbols can be escaped" {
	test(r#"
		[link](foo\)\:)

		[link](foo\bar)
	"#, r#"
		<p><a href="foo):">link</a></p>
		<p><a href="foo\bar">link</a></p>
	"#);
}

it "can contain fragment identifiers and queries" {
	test(r#"
		[link](#fragment)

		[link](http://example.com#fragment)

		[link](http://example.com?foo=3#frag)
	"#, r##"
		<p><a href="#fragment">link</a></p>
		<p><a href="http://example.com#fragment">link</a></p>
		<p><a href="http://example.com?foo=3#frag">link</a></p>
	"##);
}

it "should allow URL escaping" {
	test(r#"
		[link](foo%20b&auml;)
	"#, r#"
		<p><a href="foo%20bä">link</a></p>
	"#);
}

it "titles may be in single quotes, double quotes, or parentheses" {
	test(r#"
		[link](/url "title")
		[link](/url 'title')
		[link](/url (title))
	"#, r#"
		<p><a href="/url" title="title">link</a>
		<a href="/url" title="title">link</a>
		<a href="/url" title="title">link</a></p>
	"#);
}

it "entities may be used on title" {
	test(r#"
		[link](/url "title \"&quot;&copy;&#35;&#X22;")
	"#, r#"
		<p><a href="/url" title="title &quot;&quot;©#&quot;">link</a></p>
	"#);
}

it "title must be separated from link using whitespace" {
	test("[link](/url \"title\")", r#"<p><a href="/url" title="title">link</a></p>"#);
	test("[link](/url\t\"title\")", r#"<p><a href="/url" title="title">link</a></p>"#);
	test("[link](/url\u{00A0}\"title\")", "<p><a href=\"/url\u{00A0}&quot;title&quot;\">link</a></p>");
}

it "nested balanced quotes are not allowed" {
	test(r#"
		[link](/url "title "and" title")
	"#, r#"
		<p>[link](/url &quot;title &quot;and&quot; title&quot;)</p>
	"#);

	test(r#"
		[link](/url 'title "and" title')
	"#, r#"
		<p><a href="/url" title="title &quot;and&quot; title">link</a></p>
	"#);
}

it "whitespace is allowed around destination and title" {
	test(r#"
		[link](   /uri
			"title"
			)
	"#, r#"
		<p><a href="/uri" title="title">link</a></p>
	"#);

	test(r#"
		[link] (/uri)
	"#, r#"
		<p>[link] (/uri)</p>
	"#);
}

it "may contain balanced and escaped brackets" {
	test(r#"
		[link [foo [bar]]](/uri)
	"#, r#"
		<p><a href="/uri">link [foo [bar]]</a></p>
	"#);

	test(r#"
		[link] bar](/uri)
	"#, r#"
		<p>[link] bar](/uri)</p>
	"#);

	test(r#"
		[link [bar](/uri)
	"#, r#"
		<p>[link <a href="/uri">bar</a></p>
	"#);

	test(r#"
		[link \[bar](/uri)
	"#, r#"
		<p><a href="/uri">link [bar</a></p>
	"#);
}

#[ignore]
it "text may contain inline content" {
	test(r#"
		[link *foo **bar** `#`*](/uri)
	"#, r#"
		<p><a href="/uri">link <em>foo <strong>bar</strong> <code>#</code></em></a></p>
	"#);

	test(r#"
		[![moon](moon.jpg)](/uri)
	"#, r#"
		<p><a href="/uri"><img src="moon.jpg" alt="moon"/></a></p>
	"#);
}

#[allow(unreachable_code)]
it "links may not contain other links" {
	test(r#"
		[foo [bar](/uri)](/uri)
	"#, r#"
		<p>[foo <a href="/uri">bar</a>](/uri)</p>
	"#);

	// TODO: enable me when we support emphasis and images
	return;
	test(r#"
		[foo *[bar [baz](/uri)](/uri)*](/uri)
	"#, r#"
		<p>[foo <em>[bar <a href="/uri">baz</a>](/uri)</em>](/uri)</p>
	"#);

	test(r#"
		![[[foo](uri1)](uri2)](uri3)
	"#, r#"
		<p><img src="uri3" alt="[foo](uri2)" /></p>
	"#);
}

#[allow(unreachable_code)]
it "has precedence over emphasis" {
	test(r#"
		*[foo*](/uri)
	"#, r#"
		<p>*<a href="/uri">foo*</a></p>
	"#);

	test(r#"
		[foo *bar](baz*)
	"#, r#"
		<p><a href="baz*">foo *bar</a></p>
	"#);

	// TODO: enable me when we support emphasis
	return;
	test(r#"
		*foo [bar* baz]
	"#, r#"
		<p><em>foo [bar</em> baz]</p>
	"#);
}

#[ignore]
it "has lower precedence than tags" {
	test(r#"
		[foo <bar attr="](baz)">
	"#, r#"
		<p>[foo <bar attr="](baz)"></p>
	"#);
}

it "has lower precedence than code" {
	test(r#"
		[foo`](/uri)`
	"#, r#"
		<p>[foo<code>](/uri)</code></p>
	"#);

	test(r#"
		[foo`some code`](/uri)`more code`
	"#, r#"
		<p><a href="/uri">foo<code>some code</code></a><code>more code</code></p>
	"#);
}

it "has lower precedence than autolinks" {
	test(r#"
		[foo<http://example.com/?search=](uri)>
	"#, r#"
		<p>[foo<a href="http://example.com/?search=](uri)">http://example.com/?search=](uri)</a></p>
	"#);
}

// TODO: test reference links

}}
