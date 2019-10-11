use super::*;

// spell-checker: disable

mod markdown_spec_text {
	use super::*;

	#[test]
	fn example_671_plain_text() {
		// example 671
		test(
			r##"
				hello $.;'there
			"##,
			r##"
				<p>hello $.;&apos;there</p>
			"##,
		);
	}

	#[test]
	fn example_672_plain_text() {
		// example 672
		test(
			r##"
				Foo χρῆν
			"##,
			r##"
				<p>Foo χρῆν</p>
			"##,
		);
	}

	#[test]
	fn example_673_preserves_internal_spacing() {
		// example 673
		test(
			r##"
				Multiple     spaces
			"##,
			r##"
				<p>Multiple     spaces</p>
			"##,
		);
	}
}
