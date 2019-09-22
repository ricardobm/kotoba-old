use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;

#[derive(Serialize)]
pub struct Inflection {
	pub term: String,
	pub from: Vec<&'static str>,
}

impl std::fmt::Display for Inflection {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let rules = if self.from.len() > 0 {
			format!(" [{}]", self.from.join(" / "))
		} else {
			format!("")
		};
		write!(f, "{}{}", self.term, rules)
	}
}

pub struct Rule {
	pub kana_src: &'static str,
	pub kana_dst: &'static str,
}

/// Check if the suffix of the input can possibly be de-inflected.
pub fn can_deinflect(input: &str) -> bool {
	struct M {
		suffixes: HashMap<char, Vec<&'static str>>,
	}

	lazy_static! {
		static ref MAP: M = {
			let mut map = M {
				suffixes: Default::default(),
			};

			for (_, entries) in get_rules().iter() {
				for rule in entries.iter() {
					let suffix: &'static str = rule.kana_src;
					let last = suffix.chars().last().unwrap();
					map.suffixes
						.entry(last)
						.and_modify(|e| e.push(suffix))
						.or_insert(vec![suffix]);
				}
			}

			for (_, entries) in map.suffixes.iter_mut() {
				entries.sort_by_key(|x| x.len());
			}

			map
		};
	}

	let m: &M = &MAP;
	if let Some(chr) = input.chars().last() {
		if let Some(entries) = m.suffixes.get(&chr) {
			for it in entries.iter() {
				if input.ends_with(it) {
					return true;
				}
			}
		}
	}

	false
}

/// Attempts to de-inflect the input term and return all possible inflected
/// forms.
pub fn deinflect(input: &str) -> Vec<Inflection> {
	let rules = get_rules();

	let mut has_inflection = HashSet::new();
	let mut pending = LinkedList::new();
	pending.push_back(Inflection {
		term: input.to_string(),
		from: vec![],
	});
	has_inflection.insert(input.to_string());

	let mut result = Vec::new();
	while let Some(next) = pending.pop_front() {
		for (&name, rules) in rules.iter() {
			for rule in rules.iter() {
				let rule_suffix = rule.kana_src;
				if next.term.ends_with(rule_suffix) {
					let suffix_offset = next.term.len() - rule_suffix.len();
					let new_term = format!("{}{}", &next.term[..suffix_offset], rule.kana_dst);
					if new_term.len() > 0 && !has_inflection.contains(&new_term) {
						has_inflection.insert(new_term.clone());
						let mut inflection = Inflection {
							term: new_term,
							from: next.from.clone(),
						};
						inflection.from.push(name);
						pending.push_back(inflection);
					}
				}
			}
		}
		result.push(next);
	}

	result
}

macro_rules! inflect {
	(
		$(
			$name:literal => {
				$(
					$item:expr
				)*
			}
		)*
	) => {
		{
			let mut map = HashMap::new();
			$(
				map.insert($name, vec![
					$( $item , )*
				]);
			)*
			map
		}
	}
}

macro_rules! r {
	(
		$kana_i:literal => $kana_o:literal
	) => {{
		Rule {
			kana_src: $kana_i,
			kana_dst: $kana_o,
			}
		}};
}

/// Map of inflection rules.
///
/// These rules are translated from:
///
/// https://github.com/FooSoft/yomichan/blob/master/ext/bg/lang/deinflect.json
#[inline]
fn get_rules() -> &'static HashMap<&'static str, Vec<Rule>> {
	lazy_static! {
		static ref RULES: HashMap<&'static str, Vec<Rule>> = {
			// spell-checker: disable
			inflect!(
				"-ba" => {
					r!("えば" => "う")
					r!("けば" => "く")
					r!("げば" => "ぐ")
					r!("せば" => "す")
					r!("てば" => "つ")
					r!("ねば" => "ぬ")
					r!("べば" => "ぶ")
					r!("めば" => "む")
					r!("れば" => "る")
					r!("ければ" => "い")
				}
				"-chau" => {
					r!("ちゃう" => "る")
					r!("いじゃう" => "ぐ")
					r!("いちゃう" => "く")
					r!("きちゃう" => "くる")
					r!("しちゃう" => "す")
					r!("しちゃう" => "する")
					r!("っちゃう" => "う")
					r!("っちゃう" => "く")
					r!("っちゃう" => "つ")
					r!("っちゃう" => "る")
					r!("んじゃう" => "ぬ")
					r!("んじゃう" => "ぶ")
					r!("んじゃう" => "む")
				}
				"-nasai" => {
					r!("なさい" => "る")
					r!("いなさい" => "う")
					r!("きなさい" => "く")
					r!("きなさい" => "くる")
					r!("ぎなさい" => "ぐ")
					r!("しなさい" => "す")
					r!("しなさい" => "する")
					r!("ちなさい" => "つ")
					r!("になさい" => "ぬ")
					r!("びなさい" => "ぶ")
					r!("みなさい" => "む")
					r!("りなさい" => "る")
				}
				"-nu" => {
					r!("ぬ" => "る")
					r!("かぬ" => "く")
					r!("がぬ" => "ぐ")
					r!("こぬ" => "くる")
					r!("さぬ" => "す")
					r!("せぬ" => "する")
					r!("たぬ" => "つ")
					r!("なぬ" => "ぬ")
					r!("ばぬ" => "ぶ")
					r!("まぬ" => "む")
					r!("らぬ" => "る")
					r!("わぬ" => "う")
				}
				"-sou" => {
					r!("そう" => "い")
					r!("そう" => "る")
					r!("いそう" => "う")
					r!("きそう" => "く")
					r!("きそう" => "くる")
					r!("ぎそう" => "ぐ")
					r!("しそう" => "す")
					r!("しそう" => "する")
					r!("ちそう" => "つ")
					r!("にそう" => "ぬ")
					r!("びそう" => "ぶ")
					r!("みそう" => "む")
					r!("りそう" => "る")
				}
				"-sugiru" => {
					r!("すぎる" => "い")
					r!("すぎる" => "る")
					r!("いすぎる" => "う")
					r!("きすぎる" => "く")
					r!("きすぎる" => "くる")
					r!("ぎすぎる" => "ぐ")
					r!("しすぎる" => "す")
					r!("しすぎる" => "する")
					r!("ちすぎる" => "つ")
					r!("にすぎる" => "ぬ")
					r!("びすぎる" => "ぶ")
					r!("みすぎる" => "む")
					r!("りすぎる" => "る")
				}
				"-tai" => {
					r!("たい" => "る")
					r!("いたい" => "う")
					r!("きたい" => "く")
					r!("きたい" => "くる")
					r!("ぎたい" => "ぐ")
					r!("したい" => "す")
					r!("したい" => "する")
					r!("ちたい" => "つ")
					r!("にたい" => "ぬ")
					r!("びたい" => "ぶ")
					r!("みたい" => "む")
					r!("りたい" => "る")
				}
				"-tara" => {
					r!("たら" => "る")
					r!("いたら" => "く")
					r!("いだら" => "ぐ")
					r!("きたら" => "くる")
					r!("したら" => "す")
					r!("したら" => "する")
					r!("ったら" => "う")
					r!("ったら" => "つ")
					r!("ったら" => "る")
					r!("んだら" => "ぬ")
					r!("んだら" => "ぶ")
					r!("んだら" => "む")
					r!("かったら" => "い")
					r!("のたもうたら" => "のたまう")
					r!("いったら" => "いく")
					r!("おうたら" => "おう")
					r!("こうたら" => "こう")
					r!("そうたら" => "そう")
					r!("とうたら" => "とう")
					r!("行ったら" => "行く")
					r!("逝ったら" => "逝く")
					r!("往ったら" => "往く")
					r!("請うたら" => "請う")
					r!("乞うたら" => "乞う")
					r!("恋うたら" => "恋う")
					r!("問うたら" => "問う")
					r!("負うたら" => "負う")
					r!("沿うたら" => "沿う")
					r!("添うたら" => "添う")
					r!("副うたら" => "副う")
					r!("厭うたら" => "厭う")
				}
				"-tari" => {
					r!("たり" => "る")
					r!("いたり" => "く")
					r!("いだり" => "ぐ")
					r!("きたり" => "くる")
					r!("したり" => "す")
					r!("したり" => "する")
					r!("ったり" => "う")
					r!("ったり" => "つ")
					r!("ったり" => "る")
					r!("んだり" => "ぬ")
					r!("んだり" => "ぶ")
					r!("んだり" => "む")
					r!("かったり" => "い")
					r!("のたもうたり" => "のたまう")
					r!("いったり" => "いく")
					r!("おうたり" => "おう")
					r!("こうたり" => "こう")
					r!("そうたり" => "そう")
					r!("とうたり" => "とう")
					r!("行ったり" => "行く")
					r!("逝ったり" => "逝く")
					r!("往ったり" => "往く")
					r!("請うたり" => "請う")
					r!("乞うたり" => "乞う")
					r!("恋うたり" => "恋う")
					r!("問うたり" => "問う")
					r!("負うたり" => "負う")
					r!("沿うたり" => "沿う")
					r!("添うたり" => "添う")
					r!("副うたり" => "副う")
					r!("厭うたり" => "厭う")
				}
				"-te" => {
					r!("て" => "る")
					r!("いて" => "く")
					r!("いで" => "ぐ")
					r!("きて" => "くる")
					r!("くて" => "い")
					r!("して" => "す")
					r!("して" => "する")
					r!("って" => "う")
					r!("って" => "つ")
					r!("って" => "る")
					r!("んで" => "ぬ")
					r!("んで" => "ぶ")
					r!("んで" => "む")
					r!("のたもうて" => "のたまう")
					r!("いって" => "いく")
					r!("おうて" => "おう")
					r!("こうて" => "こう")
					r!("そうて" => "そう")
					r!("とうて" => "とう")
					r!("行って" => "行く")
					r!("逝って" => "逝く")
					r!("往って" => "往く")
					r!("請うて" => "請う")
					r!("乞うて" => "乞う")
					r!("恋うて" => "恋う")
					r!("問うて" => "問う")
					r!("負うて" => "負う")
					r!("沿うて" => "沿う")
					r!("添うて" => "添う")
					r!("副うて" => "副う")
					r!("厭うて" => "厭う")
				}
				"-toku" => {
					r!("いとく" => "く")
					r!("いどく" => "ぐ")
					r!("きとく" => "くる")
					r!("しとく" => "す")
					r!("しとく" => "する")
					r!("っとく" => "う")
					r!("っとく" => "つ")
					r!("っとく" => "る")
					r!("んどく" => "ぬ")
					r!("んどく" => "ぶ")
					r!("んどく" => "む")
					r!("とく" => "る")
				}
				"-zu" => {
					r!("ず" => "る")
					r!("かず" => "く")
					r!("がず" => "ぐ")
					r!("こず" => "くる")
					r!("さず" => "す")
					r!("せず" => "する")
					r!("たず" => "つ")
					r!("なず" => "ぬ")
					r!("ばず" => "ぶ")
					r!("まず" => "む")
					r!("らず" => "る")
					r!("わず" => "う")
				}
				"adv" => {
					r!("く" => "い")
				}
				"causative" => {
					r!("かせる" => "く")
					r!("がせる" => "ぐ")
					r!("させる" => "する")
					r!("させる" => "る")
					r!("させる" => "す")
					r!("たせる" => "つ")
					r!("なせる" => "ぬ")
					r!("ばせる" => "ぶ")
					r!("ませる" => "む")
					r!("らせる" => "る")
					r!("わせる" => "う")
					r!("こさせる" => "くる")
				}
				"causative passive" => {
					r!("かされる" => "く")
					r!("がされる" => "ぐ")
					r!("たされる" => "つ")
					r!("なされる" => "ぬ")
					r!("ばされる" => "ぶ")
					r!("まされる" => "む")
					r!("らされる" => "る")
					r!("わされる" => "う")
				}
				"imperative" => {
					r!("い" => "る")
					r!("え" => "う")
					r!("け" => "く")
					r!("げ" => "ぐ")
					r!("せ" => "す")
					r!("て" => "つ")
					r!("ね" => "ぬ")
					r!("べ" => "ぶ")
					r!("め" => "む")
					r!("よ" => "る")
					r!("れ" => "る")
					r!("ろ" => "る")
					r!("こい" => "くる")
					r!("しろ" => "する")
					r!("せよ" => "する")
				}
				// "imperative negative" => {
				// 	r!("な" => "") // too common after any word
				// }
				"masu stem" => {
					r!("い" => "いる")
					r!("い" => "う")
					r!("え" => "える")
					r!("き" => "きる")
					r!("き" => "く")
					r!("き" => "くる")
					r!("ぎ" => "ぎる")
					r!("ぎ" => "ぐ")
					r!("け" => "ける")
					r!("げ" => "げる")
					r!("し" => "す")
					r!("じ" => "じる")
					r!("せ" => "せる")
					r!("ぜ" => "ぜる")
					r!("ち" => "ちる")
					r!("ち" => "つ")
					r!("て" => "てる")
					r!("で" => "でる")
					r!("に" => "にる")
					r!("に" => "ぬ")
					r!("ね" => "ねる")
					r!("ひ" => "ひる")
					r!("び" => "びる")
					r!("び" => "ぶ")
					r!("へ" => "へる")
					r!("べ" => "べる")
					r!("み" => "みる")
					r!("み" => "む")
					r!("め" => "める")
					r!("り" => "りる")
					r!("り" => "る")
					r!("れ" => "れる")
				}
				"negative" => {
					r!("ない" => "る")
					r!("かない" => "く")
					r!("がない" => "ぐ")
					r!("くない" => "い")
					r!("こない" => "くる")
					r!("さない" => "す")
					r!("しない" => "する")
					r!("たない" => "つ")
					r!("なない" => "ぬ")
					r!("ばない" => "ぶ")
					r!("まない" => "む")
					r!("らない" => "る")
					r!("わない" => "う")
				}
				"noun" => {
					r!("さ" => "い")
				}
				"passive" => {
					r!("かれる" => "く")
					r!("がれる" => "ぐ")
					r!("される" => "する")
					r!("される" => "す")
					r!("たれる" => "つ")
					r!("なれる" => "ぬ")
					r!("ばれる" => "ぶ")
					r!("まれる" => "む")
					r!("われる" => "う")
					r!("られる" => "る")
				}
				"past" => {
					r!("た" => "る")
					r!("いた" => "く")
					r!("いだ" => "ぐ")
					r!("きた" => "くる")
					r!("した" => "す")
					r!("した" => "する")
					r!("った" => "う")
					r!("った" => "つ")
					r!("った" => "る")
					r!("んだ" => "ぬ")
					r!("んだ" => "ぶ")
					r!("んだ" => "む")
					r!("かった" => "い")
					r!("のたもうた" => "のたまう")
					r!("いった" => "いく")
					r!("おうた" => "おう")
					r!("こうた" => "こう")
					r!("そうた" => "そう")
					r!("とうた" => "とう")
					r!("行った" => "行く")
					r!("逝った" => "逝く")
					r!("往った" => "往く")
					r!("請うた" => "請う")
					r!("乞うた" => "乞う")
					r!("恋うた" => "恋う")
					r!("問うた" => "問う")
					r!("負うた" => "負う")
					r!("沿うた" => "沿う")
					r!("添うた" => "添う")
					r!("副うた" => "副う")
					r!("厭うた" => "厭う")
				}
				"polite" => {
					r!("ます" => "る")
					r!("います" => "う")
					r!("きます" => "く")
					r!("きます" => "くる")
					r!("ぎます" => "ぐ")
					r!("します" => "す")
					r!("します" => "する")
					r!("ちます" => "つ")
					r!("にます" => "ぬ")
					r!("びます" => "ぶ")
					r!("みます" => "む")
					r!("ります" => "る")
				}
				"polite negative" => {
					r!("ません" => "る")
					r!("いません" => "う")
					r!("きません" => "く")
					r!("きません" => "くる")
					r!("ぎません" => "ぐ")
					r!("しません" => "す")
					r!("しません" => "する")
					r!("ちません" => "つ")
					r!("にません" => "ぬ")
					r!("びません" => "ぶ")
					r!("みません" => "む")
					r!("りません" => "る")
					r!("くありません" => "い")
				}
				"polite past" => {
					r!("ました" => "る")
					r!("いました" => "う")
					r!("きました" => "く")
					r!("きました" => "くる")
					r!("ぎました" => "ぐ")
					r!("しました" => "す")
					r!("しました" => "する")
					r!("ちました" => "つ")
					r!("にました" => "ぬ")
					r!("びました" => "ぶ")
					r!("みました" => "む")
					r!("りました" => "る")
				}
				"polite past negative" => {
					r!("ませんでした" => "る")
					r!("いませんでした" => "う")
					r!("きませんでした" => "く")
					r!("きませんでした" => "くる")
					r!("ぎませんでした" => "ぐ")
					r!("しませんでした" => "す")
					r!("しませんでした" => "する")
					r!("ちませんでした" => "つ")
					r!("にませんでした" => "ぬ")
					r!("びませんでした" => "ぶ")
					r!("みませんでした" => "む")
					r!("りませんでした" => "る")
					r!("くありませんでした" => "い")
				}
				"polite volitional" => {
					r!("ましょう" => "る")
					r!("いましょう" => "う")
					r!("きましょう" => "く")
					r!("きましょう" => "くる")
					r!("ぎましょう" => "ぐ")
					r!("しましょう" => "す")
					r!("しましょう" => "する")
					r!("ちましょう" => "つ")
					r!("にましょう" => "ぬ")
					r!("びましょう" => "ぶ")
					r!("みましょう" => "む")
					r!("りましょう" => "る")
				}
				"potential" => {
					r!("える" => "う")
					r!("ける" => "く")
					r!("げる" => "ぐ")
					r!("せる" => "す")
					r!("てる" => "つ")
					r!("ねる" => "ぬ")
					r!("べる" => "ぶ")
					r!("める" => "む")
					r!("れる" => "る")
					r!("これる" => "くる")
				}
				"potential or passive" => {
					r!("られる" => "る")
					r!("こられる" => "くる")
				}
				"volitional" => {
					r!("おう" => "う")
					r!("こう" => "く")
					r!("ごう" => "ぐ")
					r!("そう" => "す")
					r!("とう" => "つ")
					r!("のう" => "ぬ")
					r!("ぼう" => "ぶ")
					r!("もう" => "む")
					r!("よう" => "る")
					r!("ろう" => "る")
					r!("こよう" => "くる")
					r!("しよう" => "する")
				}
			)
			// spell-checker: enable

			/*
				`RULES` generation script
				=========================

				// From https://github.com/FooSoft/yomichan/blob/master/ext/bg/lang/deinflect.json
				let DATA = {};

				{
					const WITH_RULES = false;
					let keys = Object.keys(DATA).sort();
					let x = '';
					for (let k of keys) {
						x += `\t\t\t\t${s(k)} => {\n`;
						for (let v of DATA[k]) {
							x += `\t\t\t\t\tr!(${s(v.kanaIn)} => ${s(v.kanaOut)}`;
							if (WITH_RULES) {
								x += `,\n\t\t\t\t\t\t`;
								x += `i => [${v.rulesIn.map(x => s(x)).join(', ')}], `;
								x += `o => [${v.rulesOut.map(x => s(x)).join(', ')}]`;
							}
							x += `)\n`;
						}
						x += `\t\t\t\t}\n`;
					}
					console.log(x);
					function s(obj) { return JSON.stringify(obj) }
				}
			*/
		};
	}
	&RULES
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_deinflection() {
		fn check(input: &str, expected: &str) -> bool {
			let list: Vec<_> = deinflect(input).into_iter().map(|x| x.term).collect();
			list.into_iter().any(|x| &x == expected)
		}

		assert!(check("食べていませんでした", "食べている"));
		assert!(check("食べていません", "食べている"));
		assert!(check("食べて", "食べる"));
	}

	#[test]
	fn test_can_deinflect() {
		assert!(!can_deinflect("食"));
		assert!(!can_deinflect(""));
		assert!(can_deinflect("いじゃう"));
		assert!(can_deinflect("食べて"));
	}
}
