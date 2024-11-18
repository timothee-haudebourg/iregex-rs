use std::{
	collections::{BTreeMap, BTreeSet},
	hash::Hash,
	ops::Deref,
};

use crate::{Automaton, Token, NFA};

use super::VisitingState;

/// NFA tags.
pub struct Tags<Q, T>(BTreeMap<(Q, Q), BTreeSet<T>>);

impl<Q, T> Default for Tags<Q, T> {
	fn default() -> Self {
		Self(BTreeMap::new())
	}
}

impl<Q, T> Tags<Q, T> {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<Q: Ord, T: Ord> Tags<Q, T> {
	pub fn insert(&mut self, source: Q, tag: T, target: Q) -> bool {
		self.0.entry((source, target)).or_default().insert(tag)
	}

	pub fn get(&self, source: Q, target: Q) -> impl Iterator<Item = &T> {
		self.0.get(&(source, target)).into_iter().flatten()
	}
}

pub struct TaggedNFA<Q, T, G> {
	pub untagged: NFA<Q, T>,
	pub tags: Tags<Q, G>,
}

impl<Q, T, G> TaggedNFA<Q, T, G> {
	pub fn new(untagged: NFA<Q, T>, tags: Tags<Q, G>) -> Self {
		Self { untagged, tags }
	}

	pub fn into_untagged(self) -> NFA<Q, T> {
		self.untagged
	}
}

impl<Q, T, G> Deref for TaggedNFA<Q, T, G> {
	type Target = NFA<Q, T>;

	fn deref(&self) -> &Self::Target {
		&self.untagged
	}
}

impl<Q: Ord + Hash, T: Token, G> Automaton<T> for TaggedNFA<Q, T, G> {
	type State<'a> = VisitingState<'a, Q>
		where
			Self: 'a;

	fn initial_state(&self) -> Option<Self::State<'_>> {
		Automaton::initial_state(&self.untagged)
	}

	fn next_state<'a>(
		&'a self,
		current_state: Self::State<'a>,
		token: T,
	) -> Option<Self::State<'_>> {
		Automaton::next_state(&self.untagged, current_state, token)
	}

	fn is_final_state<'a>(&'a self, state: &Self::State<'a>) -> bool {
		Automaton::is_final_state(&self.untagged, state)
	}
}
