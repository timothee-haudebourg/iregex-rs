use iregex_automata::{
	any_char,
	nfa::{BuildNFA, StateBuilder},
	NFA,
};

use crate::Alternation;

pub enum Affix {
	Any,
	Anchor,
	Alternation(Alternation),
}

impl<Q: Copy + Ord> BuildNFA<Q> for Affix {
	fn build_nfa_from<S: StateBuilder<Q>>(
		&self,
		state_builder: &mut S,
		nfa: &mut NFA<Q>,
	) -> Result<(Q, Q), S::Error> {
		match self {
			Self::Any => {
				let q = state_builder.next_state(nfa)?;
				nfa.add(q, Some(any_char()), q);
				Ok((q, q))
			}
			Self::Anchor => {
				let q = state_builder.next_state(nfa)?;
				Ok((q, q))
			}
			Self::Alternation(alt) => alt.build_nfa_from(state_builder, nfa),
		}
	}
}
