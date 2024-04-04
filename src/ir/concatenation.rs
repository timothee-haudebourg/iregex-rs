use iregex_automata::{
	nfa::{BuildNFA, StateBuilder},
	Class, Map, Token, NFA,
};
use std::{hash::Hash, ops::Deref};

use crate::Boundary;

use super::Atom;

/// Regular expression atom sequence.
#[derive(Debug, Clone)]
pub struct Concatenation<T = char, B = ()>(Vec<Atom<T, B>>);

impl<T, B> Default for Concatenation<T, B> {
	fn default() -> Self {
		Self(Vec::new())
	}
}

impl<T, B> Concatenation<T, B> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn push(&mut self, atom: Atom<T, B>) {
		self.0.push(atom)
	}
}

impl<T, B> From<Atom<T, B>> for Concatenation<T, B> {
	fn from(value: Atom<T, B>) -> Self {
		Self(vec![value])
	}
}

impl<T, B> Deref for Concatenation<T, B> {
	type Target = [Atom<T, B>];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}

impl<'a, T, B> IntoIterator for &'a Concatenation<T, B> {
	type IntoIter = std::slice::Iter<'a, Atom<T, B>>;
	type Item = &'a Atom<T, B>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl<T, B> IntoIterator for Concatenation<T, B> {
	type IntoIter = std::vec::IntoIter<Atom<T, B>>;
	type Item = Atom<T, B>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<T, B> FromIterator<Atom<T, B>> for Concatenation<T, B> {
	fn from_iter<I: IntoIterator<Item = Atom<T, B>>>(iter: I) -> Self {
		Self(Vec::from_iter(iter))
	}
}

impl<T, B, Q, C> BuildNFA<T, Q, C> for Concatenation<T, B>
where
	T: Token,
	B: Boundary<T, Class = C>,
	Q: Copy + Ord,
	C: Clone + Eq + Hash + Class<T>,
{
	fn build_nfa_from<S: StateBuilder<T, Q, C>>(
		&self,
		state_builder: &mut S,
		nfa: &mut NFA<Q, T>,
		class: &C,
	) -> Result<(Q, C::Map<Q>), S::Error> {
		match self.0.as_slice() {
			[] => {
				let a = state_builder.next_state(nfa, class.clone())?;
				Ok((a, Map::singleton(class.clone(), a)))
			}
			[atom] => atom.build_nfa_from(state_builder, nfa, class),
			list => {
				let a = state_builder.next_state(nfa, class.clone())?;

				let mut map: C::Map<(Q, bool)> = Map::singleton(class.clone(), (a, false));

				for atom in list {
					for (class, (b, _)) in std::mem::take(&mut map).into_entries() {
						let (atom_a, atom_b_map) =
							atom.build_nfa_from(state_builder, nfa, &class)?;
						nfa.add(b, None, atom_a);
						for (b_class, atom_b) in atom_b_map.into_entries() {
							let (c, merging) =
								map.get_mut_or_insert_with(&b_class, || (atom_b, false));

							if *c != atom_b {
								if *merging {
									nfa.add(atom_b, None, *c);
								} else {
									let d = state_builder.next_state(nfa, b_class)?;
									nfa.add(atom_b, None, d);
									nfa.add(*c, None, d);
									*c = d;
									*merging = true;
								}
							}
						}
					}
				}

				Ok((a, map.into_entries().map(|(c, (q, _))| (c, q)).collect()))
			}
		}
	}
}
