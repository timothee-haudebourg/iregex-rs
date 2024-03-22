//! This library provides a parser for POSIX Extended Regular Expressions (ERE).
//! Once parsed into an abstract syntax tree ([`Ast`]), regular expressions can
//! then be compiled into a finite automaton running on Unicode scalar values
//! ([`char`] type) using the [`ere-automata`] library.
//!
//! [`ere-automata`]: <https://crates.io/crates/ere-automata>
use ere_automata::{RangeSet, DFA, NFA};
use replace_with::replace_with_or_abort;
use std::{collections::HashMap, iter::Peekable};

mod parsing;
pub use parsing::*;

mod display;
pub use display::*;

/// Abstract syntax tree of an Extended Regular Expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ast {
	pub start_anchor: bool,
	pub end_anchor: bool,
	pub inner: UnanchoredAst
}

impl Ast {
	pub fn empty() -> Self {
		Self {
			start_anchor: false,
			end_anchor: false,
			inner: UnanchoredAst::empty()
		}
	}
}

pub enum Atom {
	/// Any character.
	///
	/// `.`
	Any,

	/// Character set.
	///
	/// `[]` or `[^ ]`
	Set(RangeSet<char>),

	/// Repetition.
	Repeat(Box<Self>, u32, u32),

	/// Capture group.
	Group(Disjunction)
}

/// Regular expression atom sequence.
pub struct Sequence(Vec<Atom>);

/// Regular expression sequence disjunction.
pub struct Disjunction(Vec<Sequence>);

impl UnanchoredAst {
	pub fn empty() -> Self {
		Self::Sequence(Vec::new())
	}

	/// Push the given regexp `e` at the end.
	///
	/// Builds the regexp sequence `self` followed by `e`.
	/// For instance if `self` is `/ab|cd/` then the result is `/(ab|cd)e/`
	pub fn push(&mut self, e: Self) {
		replace_with_or_abort(self, |this| match this {
			Self::Sequence(mut seq) => {
				if seq.is_empty() {
					e
				} else {
					seq.push(e);
					Self::Sequence(seq)
				}
			}
			Self::Union(items) if items.is_empty() => e,
			item => Self::Sequence(vec![item, e]),
		})
	}

	pub fn repeat(&mut self, min: u32, max: u32) {
		replace_with_or_abort(self, |this| Self::Repeat(Box::new(this), min, max))
	}

	pub fn simplified(self) -> Self {
		match self {
			Self::Any => Self::Any,
			Self::Set(set) => Self::Set(set),
			Self::Sequence(seq) => {
				let new_seq: Vec<_> = seq
					.into_iter()
					.filter_map(|e| {
						if e.is_empty() {
							None
						} else {
							Some(e.simplified())
						}
					})
					.collect();

				if new_seq.len() == 1 {
					new_seq.into_iter().next().unwrap()
				} else {
					Self::Sequence(new_seq)
				}
			}
			Self::Union(items) => {
				let new_items: Vec<_> = items.into_iter().map(Self::simplified).collect();

				if new_items.len() == 1 {
					new_items.into_iter().next().unwrap()
				} else {
					Self::Union(new_items)
				}
			}
			Self::Repeat(e, min, max) => Self::Repeat(Box::new(e.simplified()), min, max),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Set(set) => set.is_empty(),
			Self::Sequence(seq) => seq.iter().all(Self::is_empty),
			Self::Union(items) => items.iter().all(Self::is_empty),
			Self::Repeat(r, min, max) => r.is_empty() || (*min == 0 && *max == 0),
			_ => false,
		}
	}

	pub fn is_simple(&self) -> bool {
		matches!(self, Self::Any | Self::Set(_) | Self::Sequence(_))
	}

	/// Checks if this regular expression matches only one value.
	pub fn is_singleton(&self) -> bool {
		match self {
			Self::Any => false,
			Self::Set(charset) => charset.len() == 1,
			Self::Sequence(seq) => seq.iter().all(Self::is_singleton),
			Self::Repeat(e, min, max) => min == max && e.is_singleton(),
			Self::Union(items) => items.len() == 1 && items[0].is_singleton(),
		}
	}

	fn build_singleton(&self, s: &mut String) {
		match self {
			Self::Any => unreachable!(),
			Self::Set(charset) => s.push(charset.iter().next().unwrap().first().unwrap()),
			Self::Sequence(seq) => {
				for e in seq {
					e.build_singleton(s)
				}
			}
			Self::Repeat(e, _, _) => e.build_singleton(s),
			Self::Union(items) => items[0].build_singleton(s),
		}
	}

	pub fn as_singleton(&self) -> Option<String> {
		if self.is_singleton() {
			let mut s = String::new();
			self.build_singleton(&mut s);
			Some(s)
		} else {
			None
		}
	}

	pub fn build_dfa(&self) -> DFA<usize> {
		let nd = self.build_nfa();

		let mut map = HashMap::new();
		let mut n = 0usize;
		let dt = nd.determinize(|q| {
			*map.entry(q.clone()).or_insert_with(|| {
				let i = n;
				n += 1;
				i
			})
		});
		debug_assert!(!dt.final_states().is_empty());
		dt
	}

	pub fn build_nfa(&self) -> NFA<usize> {
		let mut result = NFA::new();

		let mut n = 0;
		let mut new_state = move || {
			let r = n;
			n += 1;
			r
		};

		let (a, b) = self.build_into(&mut new_state, &mut result);
		result.add_initial_state(a);
		result.add_final_state(b);
		debug_assert!(!result.final_states().is_empty());

		result
	}

	fn build_into(
		&self,
		new_state: &mut impl FnMut() -> usize,
		automaton: &mut NFA<usize>,
	) -> (usize, usize) {
		match self {
			Self::Any => {
				let mut charset = RangeSet::new();
				charset.insert('\u{0}'..='\u{d7ff}');
				charset.insert('\u{e000}'..='\u{10ffff}');
				let a = new_state();
				let b = new_state();
				automaton.add(a, Some(charset), b);
				(a, b)
			}
			Self::Repeat(exp, min, max) => exp.build_repeat_into(new_state, automaton, *min, *max),
			Self::Sequence(exps) => {
				let a = new_state();
				let mut b = a;

				for e in exps {
					let (ea, eb) = e.build_into(new_state, automaton);
					automaton.add(b, None, ea);
					b = eb;
				}

				(a, b)
			}
			Self::Set(charset) => {
				let a = new_state();
				let b = new_state();

				automaton.add(a, Some(charset.clone()), b);
				(a, b)
			}
			Self::Union(exps) => {
				let a = new_state();
				let b = new_state();

				for e in exps {
					let (ea, eb) = e.build_into(new_state, automaton);
					automaton.add(a, None, ea);
					automaton.add(eb, None, b);
				}

				(a, b)
			}
		}
	}

	fn build_repeat_into(
		&self,
		new_state: &mut impl FnMut() -> usize,
		automaton: &mut NFA<usize>,
		min: u32,
		max: u32,
	) -> (usize, usize) {
		if max == 0 {
			let a = new_state();
			(a, a)
		} else if min > 0 {
			let (a, b) = self.build_into(new_state, automaton);
			let (rest_a, rest_b) = self.build_repeat_into(
				new_state,
				automaton,
				min - 1,
				if max < u32::MAX { max - 1 } else { u32::MAX },
			);
			automaton.add(b, None, rest_a);
			(a, rest_b)
		} else if max < u32::MAX {
			let (a, b) = self.build_into(new_state, automaton);
			let (c, d) = self.build_repeat_into(new_state, automaton, 0, max - 1);
			automaton.add(a, None, d);
			automaton.add(b, None, c);
			(a, d)
		} else {
			let (a, b) = self.build_into(new_state, automaton);
			automaton.add(a, None, b);
			automaton.add(b, None, a);
			(a, b)
		}
	}
}

#[cfg(test)]
mod tests {
	// Each pair is of the form `(regexp, formatted)`.
	// We check that the regexp is correctly parsed by formatting it and
	// checking that it matches the expected `formatted` string.
	const TESTS: &[(&str, &str)] = &[
		("a*", "a*"),
		("a\\*", "a\\*"),
		("[cab]", "[a-c]"),
		("[^cab]", "[^a-c]"),
		("(abc)|de", "abc|de"),
		("(a|b)?", "(a|b)?"),
		("[A-Za-z0-89]", "[0-9A-Za-z]"),
		("[a|b]", "[ab\\|]"),
	];

	#[test]
	fn test() {
		for (regexp, formatted) in TESTS {
			assert_eq!(super::Ast::parse(regexp).unwrap().to_string(), *formatted)
		}
	}
}
