//! This library provides a parser for POSIX Extended Regular Expressions (iregex).
//! Once parsed into an abstract syntax tree ([`Ast`]), regular expressions can
//! then be compiled into a finite automaton running on Unicode scalar values
//! ([`char`] type) using the [`iregex-automata`] library.
//!
//! [`iregex-automata`]: <https://crates.io/crates/iregex-automata>
use iregex::automata::RangeSet;
use replace_with::replace_with_or_abort;
use std::ops::Deref;

mod parsing;
pub use parsing::*;

mod display;
pub use display::*;

mod build;

/// Abstract syntax tree of an Extended Regular Expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ast {
	pub start_anchor: bool,
	pub end_anchor: bool,
	pub disjunction: Disjunction,
}

impl Ast {
	pub fn empty() -> Self {
		Self {
			start_anchor: false,
			end_anchor: false,
			disjunction: Disjunction::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.disjunction.is_empty()
	}

	// /// Checks if this regular expression matches only one value.
	// pub fn is_singleton(&self) -> bool {
	// 	match self {
	// 		Self::Any => false,
	// 		Self::Set(charset) => charset.len() == 1,
	// 		Self::Sequence(seq) => seq.iter().all(Self::is_singleton),
	// 		Self::Repeat(e, min, max) => min == max && e.is_singleton(),
	// 		Self::Union(items) => items.len() == 1 && items[0].is_singleton(),
	// 	}
	// }

	// pub fn to_singleton(&self) -> Option<String> {
	// 	if self.is_singleton() {
	// 		let mut s = String::new();
	// 		self.build_singleton(&mut s);
	// 		Some(s)
	// 	} else {
	// 		None
	// 	}
	// }

	// fn build_singleton(&self, s: &mut String) {
	// 	match self {
	// 		Self::Any => unreachable!(),
	// 		Self::Set(charset) => s.push(charset.iter().next().unwrap().first().unwrap()),
	// 		Self::Sequence(seq) => {
	// 			for e in seq {
	// 				e.build_singleton(s)
	// 			}
	// 		}
	// 		Self::Repeat(e, _, _) => e.build_singleton(s),
	// 		Self::Union(items) => items[0].build_singleton(s),
	// 	}
	// }
}

/// Regular expression sequence disjunction.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Disjunction(Vec<Sequence>);

impl Disjunction {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Deref for Disjunction {
	type Target = [Sequence];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}

impl<'a> IntoIterator for &'a Disjunction {
	type IntoIter = std::slice::Iter<'a, Sequence>;
	type Item = &'a Sequence;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl IntoIterator for Disjunction {
	type IntoIter = std::vec::IntoIter<Sequence>;
	type Item = Sequence;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl From<Sequence> for Disjunction {
	fn from(value: Sequence) -> Self {
		Self(vec![value])
	}
}

/// Regular expression atom sequence.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence(Vec<Atom>);

impl Sequence {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn push(&mut self, atom: Atom) {
		self.0.push(atom)
	}

	pub fn into_disjunction(self) -> Disjunction {
		self.into()
	}
}

impl Deref for Sequence {
	type Target = [Atom];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}

impl FromIterator<Atom> for Sequence {
	fn from_iter<T: IntoIterator<Item = Atom>>(iter: T) -> Self {
		Self(Vec::from_iter(iter))
	}
}

impl<'a> IntoIterator for &'a Sequence {
	type IntoIter = std::slice::Iter<'a, Atom>;
	type Item = &'a Atom;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl IntoIterator for Sequence {
	type IntoIter = std::vec::IntoIter<Atom>;
	type Item = Atom;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Atom {
	/// Any character.
	///
	/// `.`
	Any,

	/// Single character.
	Char(char),

	/// Character set.
	///
	/// `[set]` or `[^set]`
	Set(Charset),

	/// Repetition.
	Repeat(Box<Self>, Repeat),

	/// Capture group.
	Group(Disjunction),
}

impl Atom {
	pub fn repeat(&mut self, r: Repeat) {
		replace_with_or_abort(self, |this| Self::Repeat(Box::new(this), r))
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Charset {
	negative: bool,
	classes: Classes,
	set: RangeSet<char>,
}

impl From<RangeSet<char>> for Charset {
	fn from(value: RangeSet<char>) -> Self {
		Self {
			negative: false,
			classes: Classes::none(),
			set: value,
		}
	}
}

macro_rules! classes {
	($($id:ident: $name:literal ($flag:ident: $flag_value:literal)),*) => {
		$(const $flag: u16 = $flag_value;)*

		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub enum Class {
			$($id),*
		}

		impl Class {
			pub fn from_name(name: &str) -> Option<Self> {
				match name {
					$($name => Some(Self::$id),)*
					_ => None
				}
			}

			pub fn name(&self) -> &'static str {
				match self {
					$(Self::$id => $name),*
				}
			}

			fn flag(&self) -> u16 {
				match self {
					$(Self::$id => $flag),*
				}
			}
		}

		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct Classes(u16);

		impl Classes {
			pub fn none() -> Self {
				Self(0)
			}

			pub fn all() -> Self {
				Self($($flag)|*)
			}

			pub fn contains(&self, c: Class) -> bool {
				self.0 & c.flag() != 0
			}

			pub fn insert(&mut self, c: Class) {
				self.0 |= c.flag()
			}

			pub fn iter(&self) -> ClassesIter {
				ClassesIter(self.0)
			}
		}

		pub struct ClassesIter(u16);

		impl Iterator for ClassesIter {
			type Item = Class;

			fn next(&mut self) -> Option<Class> {
				$(
					if self.0 & $flag != 0 {
						self.0 &= !$flag;
						return Some(Class::$id)
					}
				)*

				None
			}
		}
	};
}

impl IntoIterator for Classes {
	type IntoIter = ClassesIter;
	type Item = Class;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a Classes {
	type IntoIter = ClassesIter;
	type Item = Class;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

classes! {
	Upper:  "upper"  (CLASS_UPPER:  0b0000000000001),
	Lower:  "lower"  (CLASS_LOWER:  0b0000000000010),
	Alpha:  "alpha"  (CLASS_ALPHA:  0b0000000000100),
	Alnum:  "alnum"  (CLASS_ALNUM:  0b0000000001000),
	Digit:  "digit"  (CLASS_DIGIT:  0b0000000010000),
	Xdigit: "xdigit" (CLASS_XDIGIT: 0b0000000100000),
	Punct:  "punct"  (CLASS_PUNCT:  0b0000001000000),
	Blank:  "blank"  (CLASS_BLANK:  0b0000100000000),
	Space:  "space"  (CLASS_SPACE:  0b0001000000000),
	Cntrl:  "cntrl"  (CLASS_CNTRL:  0b0010000000000),
	Graph:  "graph"  (CLASS_GRAPH:  0b0100000000000),
	Print:  "print"  (CLASS_PRINT:  0b1000000000000)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Repeat {
	pub min: u32,
	pub max: Option<u32>,
}

#[cfg(test)]
mod tests {
	use iregex::automata::nfa::U32StateBuilder;

	use crate::Ast;

	#[test]
	fn test1() {
		let ast = Ast::parse("^#([^\n#][^\n]*)?$".chars()).unwrap();
		let exp = ast.build();
		let aut = exp.compile(U32StateBuilder::new()).unwrap();

		assert!(aut.matches_str("#").next().is_some());
		assert!(aut.matches_str("#a").next().is_some());
		assert!(aut.matches_str("##").next().is_none());

		// let aut = aut
		// 	.root
		// 	.unwrap()
		// 	.unwrap();

		// for &q in aut.states() {
		// 	eprintln!("#{q}:");
		// 	for (label, r) in aut.successors(&q) {
		// 		match label {
		// 			Some(set) => {
		// 				eprintln!("\t{set:?} => {r:?}");
		// 			}
		// 			None => {
		// 				eprintln!("\telse {r:?}")
		// 			}
		// 		}
		// 	}
		// }
	}
}
