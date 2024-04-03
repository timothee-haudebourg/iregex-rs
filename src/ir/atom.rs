use iregex_automata::{
	nfa::{BuildNFA, StateBuilder},
	RangeSet, NFA,
};

use super::{Alternation, CaptureGroupId, Repeat};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Atom {
	/// Token.
	Token(RangeSet<char>),

	/// Repetition.
	Repeat(Alternation, Repeat),

	/// Capture group.
	Capture(CaptureGroupId, Alternation),
}

impl Atom {
	pub fn star(inner: Alternation) -> Self {
		Self::Repeat(inner, Repeat::STAR)
	}
}

impl<Q: Copy + Ord> BuildNFA<Q> for Atom {
	fn build_nfa_from<S: StateBuilder<Q>>(
		&self,
		state_builder: &mut S,
		nfa: &mut NFA<Q>,
	) -> Result<(Q, Q), S::Error> {
		match self {
			Self::Token(set) => {
				let a = state_builder.next_state(nfa)?;
				let b = state_builder.next_state(nfa)?;
				nfa.add(a, Some(set.clone()), b);
				Ok((a, b))
			}
			Self::Repeat(alt, r) => r.build_nfa_for(alt, state_builder, nfa),
			Self::Capture(_, alt) => alt.build_nfa_from(state_builder, nfa),
		}
	}
}
