//! This library provides an implementation of the *POSIX Extended Regular
//! Expression* (ERE) class of regular expressions, and nothing more.
//! It also aims at providing easy tools to inspect finite automata built from
//! regular expressions, or manually.
//!
//! If you are looking for more advanced regular expression, please use the
//! [`regex`] library.
//!
//! [`regex`]: <https://github.com/rust-lang/regex>
use std::ops::Deref;

use regir_automata::RangeSet;
 
pub use regir_automata as automata;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Atom {
	/// Token.
	Token(RangeSet<char>),

	/// Repetition.
	Repeat(Disjunction, Repeat),

	/// Capture group.
	Capture(CaptureGroupId, Disjunction),
}

/// Capture group identifier.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CaptureGroupId(pub u32);

/// Repetition.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Repeat {
	pub min: u32,
	pub max: u32,
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
}

impl Deref for Sequence {
	type Target = [Atom];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
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