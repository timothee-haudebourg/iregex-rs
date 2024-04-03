use std::ops::Deref;

use iregex_automata::{
	nfa::{BuildNFA, StateBuilder},
	NFA,
};

use crate::{Atom, Concatenation};

/// Regular expression sequence disjunction.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Alternation(Vec<Concatenation>);

impl Alternation {
	pub fn new() -> Self {
		Self::default()
	}
}

impl From<Concatenation> for Alternation {
	fn from(value: Concatenation) -> Self {
		Self(vec![value])
	}
}

impl From<Atom> for Alternation {
	fn from(value: Atom) -> Self {
		Self(vec![value.into()])
	}
}

impl Deref for Alternation {
	type Target = [Concatenation];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}

impl<'a> IntoIterator for &'a Alternation {
	type IntoIter = std::slice::Iter<'a, Concatenation>;
	type Item = &'a Concatenation;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl IntoIterator for Alternation {
	type IntoIter = std::vec::IntoIter<Concatenation>;
	type Item = Concatenation;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl FromIterator<Concatenation> for Alternation {
	fn from_iter<T: IntoIterator<Item = Concatenation>>(iter: T) -> Self {
		Self(Vec::from_iter(iter))
	}
}

impl<Q: Copy + Ord> BuildNFA<Q> for Alternation {
	fn build_nfa_from<S: StateBuilder<Q>>(
		&self,
		state_builder: &mut S,
		nfa: &mut NFA<Q>,
	) -> Result<(Q, Q), S::Error> {
		match self.0.as_slice() {
			[] => {
				let a = state_builder.next_state(nfa)?;
				let b = state_builder.next_state(nfa)?;
				Ok((a, b))
			}
			[concat] => concat.build_nfa_from(state_builder, nfa),
			list => {
				let a = state_builder.next_state(nfa)?;
				let b = state_builder.next_state(nfa)?;

				for concat in list {
					let (concat_a, concat_b) = concat.build_nfa_from(state_builder, nfa)?;
					nfa.add(a, None, concat_a);
					nfa.add(concat_b, None, b);
				}

				Ok((a, b))
			}
		}
	}
}
