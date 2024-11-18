use std::{hash::Hash, ops::Deref};

use iregex_automata::{
	nfa::{BuildNFA, StateBuilder, Tags},
	Class, Map, Token, NFA,
};

use crate::{Atom, Boundary, CaptureTag, Concatenation};

/// Regular expression sequence disjunction.
#[derive(Debug, Clone)]
pub struct Alternation<T = char, B = ()>(Vec<Concatenation<T, B>>);

impl<T, B> Default for Alternation<T, B> {
	fn default() -> Self {
		Self(Vec::new())
	}
}

impl<T, B> Alternation<T, B> {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<T, B> From<Concatenation<T, B>> for Alternation<T, B> {
	fn from(value: Concatenation<T, B>) -> Self {
		Self(vec![value])
	}
}

impl<T, B> From<Atom<T, B>> for Alternation<T, B> {
	fn from(value: Atom<T, B>) -> Self {
		Self(vec![value.into()])
	}
}

impl<T, B> Deref for Alternation<T, B> {
	type Target = [Concatenation<T, B>];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}

impl<'a, T, B> IntoIterator for &'a Alternation<T, B> {
	type IntoIter = std::slice::Iter<'a, Concatenation<T, B>>;
	type Item = &'a Concatenation<T, B>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl<T, B> IntoIterator for Alternation<T, B> {
	type IntoIter = std::vec::IntoIter<Concatenation<T, B>>;
	type Item = Concatenation<T, B>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<T, B> FromIterator<Concatenation<T, B>> for Alternation<T, B> {
	fn from_iter<I: IntoIterator<Item = Concatenation<T, B>>>(iter: I) -> Self {
		Self(Vec::from_iter(iter))
	}
}

impl<T, B, Q, C> BuildNFA<T, Q, C, CaptureTag> for Alternation<T, B>
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
		tags: &mut Tags<Q, CaptureTag>,
		class: &C,
	) -> Result<(Q, C::Map<Q>), S::Error> {
		match self.0.as_slice() {
			[] => {
				let a = state_builder.next_state(nfa, class.clone())?;
				Ok((a, Default::default()))
			}
			[concat] => concat.build_nfa_from(state_builder, nfa, tags, class),
			list => {
				let a = state_builder.next_state(nfa, class.clone())?;
				let mut output: C::Map<Q> = Default::default();

				for concat in list {
					let (concat_a, concat_b_map) =
						concat.build_nfa_from(state_builder, nfa, tags, class)?;
					nfa.add(a, None, concat_a);

					for (b_class, concat_b) in concat_b_map.into_entries() {
						let b = *output.get_or_try_insert_with(&b_class, || {
							state_builder.next_state(nfa, b_class.clone())
						})?;

						nfa.add(concat_b, None, b);
					}
				}

				Ok((a, output))
			}
		}
	}
}
