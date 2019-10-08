use super::*;

// spell-checker: disable

mod markdown_spec_tables {
	use super::*;

	#[test]
	fn should_parse() {
		// example 198
		test(
			r##"
				| foo | bar |
				| --- | --- |
				| baz | bim |
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th>foo</th>
				<th>bar</th>
				</tr>
				</thead>
				<tbody>
				<tr>
				<td>baz</td>
				<td>bim</td>
				</tr>
				</tbody>
				</table>
			"##,
		);

		test(
			r##"
				| foo | bar |
				| --- | --- |
				| baz | bim |
				| abc | def |
				| 123 | 456 |
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th>foo</th>
				<th>bar</th>
				</tr>
				</thead>
				<tbody>
				<tr>
				<td>baz</td>
				<td>bim</td>
				</tr>
				<tr>
				<td>abc</td>
				<td>def</td>
				</tr>
				<tr>
				<td>123</td>
				<td>456</td>
				</tr>
				</tbody>
				</table>
			"##,
		);
	}

	#[test]
	fn should_parse_199() {
		// example 199
		test(
			r##"
				| abc | defghi |
				:-: | -----------:
				bar | baz
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th align="center">abc</th>
				<th align="right">defghi</th>
				</tr>
				</thead>
				<tbody>
				<tr>
				<td align="center">bar</td>
				<td align="right">baz</td>
				</tr>
				</tbody>
				</table>
			"##,
		);
	}

	#[test]
	fn should_parse_200() {
		// example 200
		test(
			r##"
				| f\|oo  |
				| ------ |
				| b `\|` az |
				| b **\|** im |
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th>f|oo</th>
				</tr>
				</thead>
				<tbody>
				<tr>
				<td>b <code>|</code> az</td>
				</tr>
				<tr>
				<td>b <strong>|</strong> im</td>
				</tr>
				</tbody>
				</table>
			"##,
		);
	}

	#[test]
	fn should_parse_201() {
		// example 201
		test(
			r##"
				| abc | def |
				| --- | --- |
				| bar | baz |
				> bar
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th>abc</th>
				<th>def</th>
				</tr>
				</thead>
				<tbody>
				<tr>
				<td>bar</td>
				<td>baz</td>
				</tr>
				</tbody>
				</table>
				<blockquote>
				<p>bar</p>
				</blockquote>
			"##,
		);
	}

	#[test]
	fn should_parse_202() {
		// example 202
		test(
			r##"
				| abc | def |
				| --- | --- |
				| bar | baz |
				bar

				bar
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th>abc</th>
				<th>def</th>
				</tr>
				</thead>
				<tbody>
				<tr>
				<td>bar</td>
				<td>baz</td>
				</tr>
				<tr>
				<td>bar</td>
				<td></td>
				</tr>
				</tbody>
				</table>
				<p>bar</p>
			"##,
		);
	}

	#[test]
	fn should_parse_203() {
		// example 203
		test(
			r##"
				| abc | def |
				| --- |
				| bar |
			"##,
			r##"
				<p>| abc | def |
				| --- |
				| bar |</p>
			"##,
		);
	}

	#[test]
	fn should_parse_204() {
		// example 204
		test(
			r##"
				| abc | def |
				| --- | --- |
				| bar |
				| bar | baz | boo |
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th>abc</th>
				<th>def</th>
				</tr>
				</thead>
				<tbody>
				<tr>
				<td>bar</td>
				<td></td>
				</tr>
				<tr>
				<td>bar</td>
				<td>baz</td>
				</tr>
				</tbody>
				</table>
			"##,
		);
	}

	#[test]
	fn should_parse_205() {
		// example 205
		test(
			r##"
				| abc | def |
				| --- | --- |
			"##,
			r##"
				<table>
				<thead>
				<tr>
				<th>abc</th>
				<th>def</th>
				</tr>
				</thead>
				</table>
			"##,
		);
	}

	#[test]
	fn should_parse_with_no_head() {
		test(
			r##"
				| --- | --- |
				| abc | def |
			"##,
			r##"
				<table>
				<tbody>
				<tr>
				<td>abc</td>
				<td>def</td>
				</tr>
				</tbody>
				</table>
			"##,
		);
	}
}
