use std::collections::HashSet;

use super::DefinitionRow;
use super::FormRow;
use super::TermRow;
use super::{TagId, TagRow};

/// Attempt to merge `b` with `a`, modifying `a`.
///
/// Returns `true` if the merge was successful.
pub fn merge_term(tags: &Vec<TagRow>, a: &mut TermRow, b: &TermRow) -> bool {
	let same_expr = a.expression == b.expression;
	let same_read = a.reading == b.reading;

	// If entries are the same, we try extra hard to merge.
	let same = (same_expr && same_read)
		|| (b.forms.iter()).any(|f| f.expression == a.expression && f.reading == a.reading)
		|| (a.forms.iter()).any(|f| f.expression == b.expression && f.reading == b.reading);

	if !same {
		// The only possibility to merge if the `(expr, read)` pair is not the
		// same is for definitions to be equivalent, in which case we add B
		// as a form of A.
		if are_definitions_equivalent(&a.definition, &b.definition) {
			merge_definitions(tags, a, b, None);
			merge_tags(tags, &mut a.tags, &b.tags);
			merge_sources(a, b);
			a.forms.push(FormRow {
				expression: b.expression.clone(),
				reading:    b.reading.clone(),
				romaji:     b.romaji.clone(),
				frequency:  b.frequency,
			});
			merge_forms(a, b);
			true
		} else {
			false
		}
	} else {
		// The `(expr, read)` pair for entries is the same, so merge away...

		// If tags between A and B are compatible, we merge them at the root
		// term, otherwise we add them to each definition of B.
		let do_merge_tags = can_merge_tags(tags, &a.tags, &b.tags);
		let tags_from_b = if do_merge_tags {
			merge_tags(tags, &mut a.tags, &b.tags);
			None
		} else {
			Some(&b.tags)
		};

		merge_definitions(tags, a, b, tags_from_b);
		merge_sources(a, b);
		merge_forms(a, b);

		true
	}
}

fn merge_sources(a: &mut TermRow, b: &TermRow) {
	for it in b.source.iter() {
		if !a.source.contains(it) {
			a.source.push(it.clone());
		}
	}
}

fn merge_forms(a: &mut TermRow, b: &TermRow) {
	for it in b.forms.iter() {
		if !a.forms.contains(it) {
			a.forms.push(it.clone());
		}
	}
}

/// Check if the two sets of definitions are considered equivalent.
///
/// Equivalent in this context means that one set of definitions is a subset
/// of the other.
///
/// Note that for simplicity sake and performance reasons, we are ignoring tags
/// between the two sets, assuming that if two definitions share the same text
/// their tags can be safely merged.
fn are_definitions_equivalent(a: &Vec<DefinitionRow>, b: &Vec<DefinitionRow>) -> bool {
	// Take the smallest set to use as base for comparison
	let (a, b) = if a.len() < b.len() { (a, b) } else { (b, a) };

	// Make sure that all definitions in A are in B. Since A is the smallest set
	// it must be by definition the subset.
	a.iter().all(|a| b.iter().any(|b| b.text == a.text))
}

/// Merge the definitions of `b` into `a`.
fn merge_definitions(tags: &Vec<TagRow>, a: &mut TermRow, b: &TermRow, additional_b_tags: Option<&HashSet<TagId>>) {
	// Compute the equivalency of items from A and B
	let mut eq = Vec::new();
	for (index_a, a) in a.definition.iter().enumerate() {
		for (index_b, b) in b.definition.iter().enumerate() {
			if b.text == a.text {
				eq.push((index_a, index_b));
				break;
			}
		}
	}

	// Append additional items from B into A
	for (index, b) in b.definition.iter().enumerate() {
		if !eq.iter().any(|(_, ib)| *ib == index) {
			let mut it = b.clone();
			if let Some(b_tags) = additional_b_tags {
				merge_tags(tags, &mut it.tags, b_tags);
			}
			a.definition.push(it);
		}
	}

	// Merge equivalent definitions.
	for (index_a, index_b) in eq.into_iter() {
		let def_a = &mut a.definition[index_a];
		let def_b = &b.definition[index_b];

		// Merge info field
		for it in def_b.info.iter() {
			if !def_a.info.contains(it) {
				def_a.info.push(it.clone());
			}
		}

		// Merge links
		for it in def_b.link.iter() {
			if !def_a.link.contains(it) {
				def_a.link.push(it.clone());
			}
		}

		// Merge tags
		merge_tags(tags, &mut def_a.tags, &def_b.tags);
	}
}

/// Check if the two sets of tags can be safely merged.
///
/// The tag sets can be merged if either of them is a subset of the other.
///
/// When checking tags for equality, this will consider tags with the same
/// name as equal (to allow for cross-database merging).
fn can_merge_tags(tags: &Vec<TagRow>, a: &HashSet<TagId>, b: &HashSet<TagId>) -> bool {
	if a.len() == 0 || b.len() == 0 {
		return true;
	}

	// Take the smallest set to use as base for comparison
	let (a, b) = if a.len() < b.len() { (a, b) } else { (b, a) };

	// First we compare by the tag sets by ID. This will only work for entries
	// within a single dictionary.
	if a.iter().all(|x| b.contains(x)) {
		return true;
	}

	// The usual case will be that tags need to be compared by name. Here we
	// are making a big assumption that tags with the same name are compatible
	// across dictionaries.
	let names = tag_names(tags, a);
	a.iter()
		.map(|&TagId(i)| tags[i].name.as_str())
		.all(|name| names.contains(name))
}

/// Merge two sets of tags, assuming they are compatible as per [can_merge_tags].
fn merge_tags(tags: &Vec<TagRow>, a: &mut HashSet<TagId>, b: &HashSet<TagId>) {
	let names = tag_names(tags, a.iter());
	for &TagId(index) in b.iter() {
		let name = tags[index].name.as_str();
		if !names.contains(name) {
			a.insert(TagId(index));
		}
	}
}

fn tag_names<'a, S, T>(tags: &'a Vec<TagRow>, set: S) -> HashSet<&'a str>
where
	S: IntoIterator<Item = T>,
	T: std::borrow::Borrow<TagId>,
{
	set.into_iter()
		.map(|it| {
			let &TagId(index) = it.borrow();
			tags[index].name.as_str()
		})
		.collect()
}
