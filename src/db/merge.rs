use std::collections::HashMap;
use std::collections::HashSet;

use super::DefinitionRow;
use super::FormRow;
use super::TermRow;
use super::{TagId, TagRow};

use itertools::*;

pub fn merge_term(_tags: &Vec<TagRow>, _a: &mut TermRow, _b: &TermRow) -> bool {
	false
}

pub fn do_merge_term(tags: &Vec<TagRow>, a: &TermRow, b: &TermRow) -> Option<TermRow> {
	let same_expr = a.expression == b.expression;
	let same_read = a.reading == b.reading;
	if !(same_expr || same_read) {
		return None;
	}

	let are_same = (same_expr && same_read)
		|| (b
			.forms
			.iter()
			.any(|f| f.expression == a.expression && f.reading == a.reading))
		|| (a
			.forms
			.iter()
			.any(|f| f.expression == b.expression && f.reading == b.reading));

	// Merge-ability of tags is a pre-requisite for any case
	if can_merge_tags(tags, &a.tags, &b.tags) {
		if are_same {
			// Exactly same expression & reading, even if definitions are
			// different we can just concatenate them
			Some(TermRow {
				expression: a.expression.clone(),
				reading:    a.reading.clone(),
				romaji:     a.romaji.clone(),
				score:      std::cmp::max(a.score, b.score),
				source:     a.source.clone(),
				tags:       merge_tags(&a.tags, &b.tags),
				definition: merge_definitions(tags, &a, &b),
				frequency:  std::cmp::max(a.frequency, b.frequency),
				forms:      a.forms.iter().chain(b.forms.iter()).cloned().unique().collect(),
			})
		} else if can_merge_definitions(tags, a, b) {
			// If the expression is not exactly the same, we can still merge if
			// definitions are
			Some(TermRow {
				expression: a.expression.clone(),
				reading:    a.reading.clone(),
				romaji:     a.romaji.clone(),
				score:      std::cmp::max(a.score, b.score),
				source:     a.source.clone(),
				tags:       merge_tags(&a.tags, &b.tags),
				definition: merge_definitions(tags, &a, &b),

				frequency: if a.frequency.is_some() || b.frequency.is_some() {
					Some(a.frequency.unwrap_or(0) + b.frequency.unwrap_or(0))
				} else {
					None
				},

				forms: (a.forms.iter())
					.chain(b.forms.iter())
					.cloned()
					.chain(std::iter::once(FormRow {
						expression: b.expression.clone(),
						reading:    b.reading.clone(),
						romaji:     b.romaji.clone(),
						frequency:  b.frequency,
					}))
					.unique()
					.collect(),
			})
		} else {
			None
		}
	} else {
		None
	}
}

pub fn merge_definition(a: &DefinitionRow, b: &DefinitionRow) -> Option<DefinitionRow> {
	if a.text == b.text {
		let out = DefinitionRow {
			text: a.text.clone(),
			info: a.info.iter().chain(b.info.iter()).cloned().collect(),
			tags: merge_tags(&a.tags, &b.tags),
			link: a.link.iter().chain(b.link.iter()).cloned().collect(),
		};
		Some(out)
	} else {
		None
	}
}

fn merge_definitions(_tags: &Vec<TagRow>, _a: &TermRow, _b: &TermRow) -> Vec<DefinitionRow> {
	panic!()
}

fn can_merge_definitions(_tags: &Vec<TagRow>, _a: &TermRow, _b: &TermRow) -> bool {
	panic!()
}

fn can_merge_tags(tags: &Vec<TagRow>, a: &HashSet<TagId>, b: &HashSet<TagId>) -> bool {
	if a.len() == 0 || b.len() == 0 {
		// If either of the tag sets is empty we can merge
		true
	} else {
		// Compare the hashsets, first by id. Note that only comparing by id
		// is not enough: there can be duplicated tags accross dictionaries.
		let (a, b) = if a.len() < b.len() { (a, b) } else { (b, a) };
		if a.iter().all(|x| b.contains(x)) {
			// If one of the sets contains the other, then we can merge.
			true
		} else {
			// Compare the tags by name
			let map_a: HashMap<_, _> = a.iter().map(|&TagId(i)| (tags[i].name.as_str(), &tags[i])).collect();
			let map_b: HashMap<_, _> = a.iter().map(|&TagId(i)| (tags[i].name.as_str(), &tags[i])).collect();
			map_a.iter().all(|(name, tag)| {
				if let Some(other_tag) = map_b.get(name) {
					// Make sure tag descriptions match or can be merged
					tag.description.len() == 0
						|| other_tag.description.len() == 0
						|| tag.description.to_lowercase() == other_tag.description.to_lowercase()
				} else {
					false
				}
			})
		}
	}
}

fn merge_tags(a: &HashSet<TagId>, b: &HashSet<TagId>) -> HashSet<TagId> {
	a.union(b).cloned().collect()
}
