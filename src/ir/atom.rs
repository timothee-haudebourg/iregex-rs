use std::hash::Hash;

use iregex_automata::{
	nfa::{BuildNFA, StateBuilder},
	Class, Map, RangeSet, Token, NFA,
};

use crate::Boundary;

use super::{Alternation, CaptureGroupId, Repeat};

#[derive(Debug, Clone)]
pub enum Atom<T, B> {
	/// Boundary.
	Boundary(B),

	/// Token.
	Token(RangeSet<T>),

	/// Repetition.
	Repeat(Alternation<T, B>, Repeat),

	/// Capture group.
	Capture(CaptureGroupId, Alternation<T, B>),
}

impl<T, B> Atom<T, B> {
	pub fn alternation(alt: Alternation<T, B>) -> Self {
		Self::Repeat(alt, Repeat::ONCE)
	}

	pub fn star(inner: Alternation<T, B>) -> Self {
		Self::Repeat(inner, Repeat::STAR)
	}
}

impl<T, B, Q, C> BuildNFA<T, Q, C> for Atom<T, B>
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
		class: &B::Class,
	) -> Result<(Q, C::Map<Q>), S::Error> {
		match self {
			Self::Boundary(boundary) => {
				let a = state_builder.next_state(nfa, class.clone())?;
				let mut output: C::Map<Q> = Default::default();
				if let Some(b_class) = boundary.apply(class) {
					let b = state_builder.next_state(nfa, b_class.clone())?;
					output.set(b_class, b);
				}
				Ok((a, output))
			}
			Self::Token(set) => {
				let a = state_builder.next_state(nfa, class.clone())?;
				let mut output: C::Map<Q> = Default::default();
				for (b_class, set) in class.classify(set).into_entries() {
					let b = state_builder.next_state(nfa, b_class.clone())?;
					nfa.add(a, Some(set.into_owned()), b);
					output.set(b_class, b);
				}

				Ok((a, output))
			}
			Self::Repeat(alt, r) => r.build_nfa_for(alt, state_builder, nfa, class),
			Self::Capture(_, alt) => alt.build_nfa_from(state_builder, nfa, class),
		}
	}
}
