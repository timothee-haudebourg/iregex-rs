use std::ops::Deref;

use iregex_automata::{
	nfa::{BuildNFA, StateBuilder},
	NFA,
};

use super::Atom;

/// Regular expression atom sequence.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Concatenation(Vec<Atom>);

impl Concatenation {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn push(&mut self, atom: Atom) {
		self.0.push(atom)
	}
}

impl From<Atom> for Concatenation {
	fn from(value: Atom) -> Self {
		Self(vec![value.into()])
	}
}

impl Deref for Concatenation {
	type Target = [Atom];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}

impl<'a> IntoIterator for &'a Concatenation {
	type IntoIter = std::slice::Iter<'a, Atom>;
	type Item = &'a Atom;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl IntoIterator for Concatenation {
	type IntoIter = std::vec::IntoIter<Atom>;
	type Item = Atom;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl FromIterator<Atom> for Concatenation {
	fn from_iter<T: IntoIterator<Item = Atom>>(iter: T) -> Self {
		Self(Vec::from_iter(iter))
	}
}

impl<Q: Copy + Ord> BuildNFA<Q> for Concatenation {
	fn build_nfa_from<S: StateBuilder<Q>>(
		&self,
		state_builder: &mut S,
		nfa: &mut NFA<Q>,
	) -> Result<(Q, Q), S::Error> {
		match self.0.as_slice() {
			[] => {
				let a = state_builder.next_state(nfa)?;
				Ok((a, a))
			}
			[atom] => atom.build_nfa_from(state_builder, nfa),
			list => {
				let a = state_builder.next_state(nfa)?;
				let mut b = a;

				for atom in list {
					let (atom_a, atom_b) = atom.build_nfa_from(state_builder, nfa)?;
					nfa.add(b, None, atom_a);
					b = atom_b
				}

				Ok((a, b))
			}
		}
	}
}
