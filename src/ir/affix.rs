use std::hash::Hash;

use iregex_automata::{
	nfa::{BuildNFA, StateBuilder},
	Class, Token, NFA,
};

use crate::{Alternation, Atom, Boundary, Concatenation};

pub enum Affix<T, B> {
	Any,
	Anchor,
	Alternation(Alternation<T, B>),
}

impl<T, B, Q, C> BuildNFA<T, Q, C> for Affix<T, B>
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
		match self {
			Self::Any => Alternation::from(Atom::<_, B>::star(Atom::Token(T::all()).into()))
				.build_nfa_from(state_builder, nfa, class),
			Self::Anchor => Alternation::from(Concatenation::<_, B>::new()).build_nfa_from(
				state_builder,
				nfa,
				class,
			),
			Self::Alternation(alt) => alt.build_nfa_from(state_builder, nfa, class),
		}
	}
}
