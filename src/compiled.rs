use std::{ops::Range, str::Chars};

use iregex_automata::{Automaton, NFA};

use crate::Token;

/// Compiled Regular Expression.
pub struct CompiledRegEx<A = NFA<u32>> {
	pub root: A,
	pub prefix: A,
	pub suffix: A,
}

impl<A> CompiledRegEx<A> {
	pub fn matches_str<'a>(&self, haystack: &'a str) -> Matches<A, Chars<'a>>
	where
		A: Automaton<char>,
	{
		self.matches(haystack.chars())
	}

	pub fn matches<H>(&self, haystack: H) -> Matches<A, H>
	where
		H: Clone + Iterator,
		A: Automaton<H::Item>,
	{
		Matches {
			regex: self,
			prefix_state: self.prefix.initial_state(),
			haystack,
			position: 0,
			min: 0,
		}
	}
}

pub struct Matches<'a, A: Automaton<H::Item>, H: Iterator> {
	regex: &'a CompiledRegEx<A>,
	prefix_state: Option<A::State<'a>>,
	haystack: H,
	position: usize,
	min: usize,
}

impl<'a, A: Automaton<H::Item>, H: Clone + Iterator> Matches<'a, A, H>
where
	H::Item: Token,
{
	fn next_from_position(&self, mut haystack: H) -> Option<usize> {
		let mut root_state = self.regex.root.initial_state()?;
		let mut end = self.position;
		let mut candidate = None;

		loop {
			if self.regex.root.is_final_state(&root_state) {
				if self.check_suffix(haystack.clone()) {
					candidate = Some(end)
				}
			}

			match haystack.next() {
				Some(token) => {
					end += token.len();
					match self.regex.root.next_state(root_state, token) {
						Some(next_state) => root_state = next_state,
						None => break,
					}
				}
				None => break,
			}
		}

		candidate
	}

	fn check_suffix(&self, haystack: H) -> bool {
		match self.regex.suffix.initial_state() {
			Some(mut suffix_state) => {
				for token in haystack {
					match self.regex.suffix.next_state(suffix_state, token) {
						Some(next_state) => suffix_state = next_state,
						None => return false,
					}
				}

				self.regex.suffix.is_final_state(&suffix_state)
			}
			None => false,
		}
	}
}

impl<'a, A: Automaton<H::Item>, H: Clone + Iterator> Iterator for Matches<'a, A, H>
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
						if let Some(end) = self.next_from_position(self.haystack.clone()) {
							self.min = end.max(self.position + 1);
							self.prefix_state = Some(prefix_state);
							break Some(self.position..end);
						}
					}

					match self.haystack.next() {
						Some(token) => {
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
