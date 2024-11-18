//! This library provides an implementation of Nondeterministic Finite Automata
//! (NFA) and Deterministic Finite Automata (DFA) for Unicode scalar values
//! (the [`char`] type). It is used by the [`ere`] crate to represent compiled
//! regular expressions.
//!
//! [`ere`]: <https://github.com/timothee-haudebourg/ere-rs>
use btree_range_map::RangePartialOrd;
pub use btree_range_map::{AnyRange, RangeSet};

pub mod nfa;
use mown::Mown;
pub use nfa::NFA;

pub mod dfa;
pub use dfa::DFA;
use range_traits::{Bounded, Measure, PartialEnum};

#[cfg(feature = "dot")]
pub mod dot;

pub fn any_char() -> RangeSet<char> {
	let mut set = RangeSet::new();
	set.insert('\u{0}'..='\u{d7ff}');
	set.insert('\u{e000}'..='\u{10ffff}');
	set
}

/// Computes the intersection of two character sets.
pub fn token_set_intersection<T>(a: &RangeSet<T>, b: &RangeSet<T>) -> RangeSet<T>
where
	T: Clone + Measure + PartialEnum,
{
	let mut result = a.clone();

	for r in b.gaps() {
		result.remove(r.cloned());
	}

	result
}

pub trait MapSource: Sized {
	type Map<U>: Map<Self, U>;
}

#[allow(clippy::len_without_is_empty)]
pub trait Token: Copy + Ord + Measure + PartialEnum + RangePartialOrd + Bounded {
	fn all() -> RangeSet<Self>;

	/// Returns the (byte) length of the token.
	fn len(&self) -> usize;

	fn is_one(len: Self::Len) -> bool;
}

impl Token for u8 {
	fn all() -> RangeSet<Self> {
		let mut set = RangeSet::new();
		set.insert(u8::MIN..=u8::MAX);
		set
	}

	fn len(&self) -> usize {
		1
	}

	fn is_one(len: Self::Len) -> bool {
		len == 1
	}
}

impl Token for char {
	fn all() -> RangeSet<Self> {
		any_char()
	}

	fn len(&self) -> usize {
		self.len_utf8()
	}

	fn is_one(len: Self::Len) -> bool {
		len == 1
	}
}

/// Token class.
pub trait Class<T = char>: MapSource {
	/// Classify the given token set.
	///
	/// The output is a partition of the input set, where each member of the
	/// partition is associated to a different class.
	fn classify<'a>(&self, set: &'a RangeSet<T>) -> Self::Map<Mown<'a, RangeSet<T>>>;

	fn next_class(&self, token: &T) -> Self;
}

pub trait Map<C, T>: Default + FromIterator<(C, T)> {
	type Iter<'a>: Iterator<Item = (&'a C, &'a T)>
	where
		C: 'a,
		T: 'a,
		Self: 'a;
	type IntoEntries: Iterator<Item = (C, T)>;

	fn singleton(class: C, value: T) -> Self {
		let mut result = Self::default();
		result.set(class, value);
		result
	}

	fn get(&self, class: &C) -> Option<&T>;

	fn get_mut(&mut self, class: &C) -> Option<&mut T>;

	fn contains(&self, class: &C) -> bool {
		self.get(class).is_some()
	}

	fn set(&mut self, class: C, value: T);

	fn get_mut_or_insert_with(&mut self, class: &C, f: impl FnOnce() -> T) -> &mut T
	where
		C: Clone,
	{
		if !self.contains(class) {
			let t = f();
			self.set(class.clone(), t);
		}

		self.get_mut(class).unwrap()
	}

	fn get_or_try_insert_with<E>(
		&mut self,
		class: &C,
		f: impl FnOnce() -> Result<T, E>,
	) -> Result<&T, E>
	where
		C: Clone,
	{
		if !self.contains(class) {
			let t = f()?;
			self.set(class.clone(), t);
		}

		Ok(self.get(class).unwrap())
	}

	fn iter(&self) -> Self::Iter<'_>;

	fn into_entries(self) -> Self::IntoEntries;
}

impl MapSource for () {
	type Map<U> = Unmapped<U>;
}

impl<T> Class<T> for () {
	fn classify<'a>(&self, set: &'a RangeSet<T>) -> Self::Map<Mown<'a, RangeSet<T>>> {
		Unmapped(Some(Mown::Borrowed(set)))
	}

	fn next_class(&self, _token: &T) -> Self {}
}

pub struct Unmapped<T>(Option<T>);

impl<T> Unmapped<T> {
	pub fn unwrap(self) -> Option<T> {
		self.0
	}
}

impl<T> Default for Unmapped<T> {
	fn default() -> Self {
		Self(None)
	}
}

impl<T> Map<(), T> for Unmapped<T> {
	type Iter<'a> = OptionClassIter<'a, T> where T: 'a;
	type IntoEntries = OptionClassIntoIter<T>;

	fn get(&self, _class: &()) -> Option<&T> {
		self.0.as_ref()
	}

	fn get_mut(&mut self, _class: &()) -> Option<&mut T> {
		self.0.as_mut()
	}

	fn set(&mut self, _class: (), value: T) {
		self.0 = Some(value)
	}

	fn iter(&self) -> Self::Iter<'_> {
		OptionClassIter(self.0.as_ref())
	}

	fn into_entries(self) -> Self::IntoEntries {
		OptionClassIntoIter(self.0)
	}
}

impl<T> FromIterator<((), T)> for Unmapped<T> {
	fn from_iter<I: IntoIterator<Item = ((), T)>>(iter: I) -> Self {
		let mut result = Self::default();
		for ((), t) in iter {
			result.set((), t);
		}
		result
	}
}

pub struct OptionClassIter<'a, T>(Option<&'a T>);

impl<'a, T> Iterator for OptionClassIter<'a, T> {
	type Item = (&'a (), &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.take().map(|t| (&(), t))
	}
}

pub struct OptionClassIntoIter<T>(Option<T>);

impl<T> Iterator for OptionClassIntoIter<T> {
	type Item = ((), T);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.take().map(|t| ((), t))
	}
}

/// Deterministic or non-deterministic automaton.
pub trait Automaton<T> {
	type State<'a>
	where
		Self: 'a;

	fn initial_state(&self) -> Option<Self::State<'_>>;

	fn next_state<'a>(
		&'a self,
		current_state: Self::State<'a>,
		token: T,
	) -> Option<Self::State<'_>>;

	fn is_final_state<'a>(&'a self, state: &Self::State<'a>) -> bool;

	fn contains(&self, tokens: impl IntoIterator<Item = T>) -> bool {
		match self.initial_state() {
			Some(mut q) => {
				for token in tokens {
					match self.next_state(q, token) {
						Some(r) => q = r,
						None => return false,
					}
				}

				self.is_final_state(&q)
			}
			None => false,
		}
	}
}

/// Deterministic or non-deterministic automaton.
pub trait TaggedAutomaton<T, G>: Automaton<T> {
	fn get_tag(&self, state: &G) -> Option<usize>;
}
