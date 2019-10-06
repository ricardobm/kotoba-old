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
	}
}
