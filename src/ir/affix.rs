use std::hash::Hash;

use iregex_automata::{
	nfa::{BuildNFA, StateBuilder, Tags},
	Class, Token, NFA,
};

use crate::{Alternation, Atom, Boundary, CaptureTag, Concatenation};

#[derive(Debug)]
pub enum Affix<T, B> {
	Any,
	Anchor,
	Alternation(Alternation<T, B>),
}

impl<T, B> Affix<T, B> {
	pub fn is_any(&self) -> bool {
		matches!(self, Self::Any)
	}

	pub fn is_anchor(&self) -> bool {
		matches!(self, Self::Anchor)
	}
}

impl<T, B, Q, C> BuildNFA<T, Q, C, CaptureTag> for Affix<T, B>
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
		match self {
			Self::Any => Alternation::from(Atom::<_, B>::star(Atom::Token(T::all()).into()))
				.build_nfa_from(state_builder, nfa, tags, class),
			Self::Anchor => Alternation::from(Concatenation::<_, B>::new()).build_nfa_from(
				state_builder,
				nfa,
				tags,
				class,
			),
			Self::Alternation(alt) => alt.build_nfa_from(state_builder, nfa, tags, class),
		}
	}
}
