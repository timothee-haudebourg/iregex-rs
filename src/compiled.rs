use std::{ops::Range, str::Chars};

use iregex_automata::{Automaton, Class, Map, MapSource, RangeSet, Token, DFA, NFA};

/// Compound automaton.
pub struct CompoundAutomaton<A = NFA<u32, char>, C: MapSource = ()> {
	pub prefix: A,
	pub root: C::Map<A>,
	pub suffix: C::Map<A>,
}

impl<A, C: MapSource> CompoundAutomaton<A, C> {
	pub fn matches_str<'a>(&self, haystack: &'a str) -> Matches<A, C, Chars<'a>>
	where
		A: Automaton<char>,
		C: Default + Class,
	{
		self.matches(haystack.chars())
	}

	pub fn matches<H>(&self, haystack: H) -> Matches<A, C, H>
	where
		H: Clone + Iterator,
		H::Item: Clone,
		A: Automaton<H::Item>,
		C: Default + Class<H::Item>,
	{
		Matches {
			regex: self,
			prefix_state: self.prefix.initial_state(),
			haystack,
			class: C::default(),
			position: 0,
			min: 0,
		}
	}
}

impl<T, Q, C: MapSource> CompoundAutomaton<NFA<Q, T>, C> {
	pub fn determinize(&self) -> CompoundAutomaton<DFA<Q, RangeSet<T>>, C> {
		todo!()
	}
}

pub struct Matches<'a, A: Automaton<H::Item>, C: MapSource, H: Iterator> {
	regex: &'a CompoundAutomaton<A, C>,
	prefix_state: Option<A::State<'a>>,
	haystack: H,
	class: C,
	position: usize,
	min: usize,
}

impl<'a, A: Automaton<H::Item>, C: Clone + Class<H::Item>, H: Clone + Iterator> Matches<'a, A, C, H>
where
	H::Item: Token,
{
	fn next_from_position(&self, mut haystack: H, class: &C) -> Option<usize> {
		let root = self.regex.root.get(class)?;
		let mut root_state = root.initial_state()?;
		let mut end = self.position;
		let mut candidate = None;

		let mut class = class.clone();

		loop {
			if root.is_final_state(&root_state) && self.check_suffix(haystack.clone(), &class) {
				candidate = Some(end)
			}

			match haystack.next() {
				Some(token) => {
					end += token.len();
					class = class.next_class(&token);
					match root.next_state(root_state, token) {
						Some(next_state) => root_state = next_state,
						None => break,
					}
				}
				None => break,
			}
		}

		candidate
	}

	fn check_suffix(&self, haystack: H, class: &C) -> bool {
		let Some(suffix) = self.regex.suffix.get(class) else {
			return false;
		};

		match suffix.initial_state() {
			Some(mut suffix_state) => {
				for token in haystack {
					match suffix.next_state(suffix_state, token) {
						Some(next_state) => suffix_state = next_state,
						None => return false,
					}
				}

				suffix.is_final_state(&suffix_state)
			}
			None => false,
		}
	}
}

impl<'a, A: Automaton<H::Item>, C: Clone + Class<H::Item>, H: Clone + Iterator> Iterator
	for Matches<'a, A, C, H>
where
	H::Item: Token,
{
	type Item = Range<usize>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.prefix_state.take() {
				Some(prefix_state) => {
					if self.position >= self.min && self.regex.prefix.is_final_state(&prefix_state)
					{
						if let Some(end) =
							self.next_from_position(self.haystack.clone(), &self.class)
						{
							self.min = end.max(self.position + 1);
							self.prefix_state = Some(prefix_state);
							break Some(self.position..end);
						}
					}

					match self.haystack.next() {
						Some(token) => {
							self.class = self.class.next_class(&token);
							self.position += token.len();
							self.prefix_state = self.regex.prefix.next_state(prefix_state, token);
						}
						None => break None,
					}
				}
				None => break None,
			}
		}
	}
}
