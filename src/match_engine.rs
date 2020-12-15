#![allow(dead_code)]
/// Matches `text` against `to_check`
pub fn check_match(text: &str, to_check: MatchType) -> Option<MatchResult> {
	match to_check {
		MatchType::Literal(matcher) => {
			if text.contains(&matcher) {
				let index = text.find(&matcher).unwrap();
				let rest = text.replacen(&matcher, "", 1);
				return Some(MatchResult {
					matched: matcher.clone(),
					index: index,
					rest: rest,
				});
			} else {
				return None;
			}
		}
		MatchType::Regex(regex) => {
			match regex.find(&text) {
				Some(result) => {
					let matched = String::from(&text[result.start() .. result.end()]);
					let rest = text.replacen(&matched, "", 1);
					let index = result.start();
					return Some(MatchResult { matched, index, rest });
				}
				None => {
					return None;
				}
			}
		}
	}
}

/// The result of [check_match]
/// # Fields
/// * `matched`: The string that matched the input
/// * `rest`: The input without the `matched` part
/// * `index`: Where the match occured
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct MatchResult {
	matched: String,
	rest: String,
	index: usize,
}

pub enum MatchType {
	Literal(String),
	Regex(regex::Regex),
}