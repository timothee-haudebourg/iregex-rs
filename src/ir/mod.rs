mod atom;
pub use atom::*;
mod concatenation;
pub use concatenation::*;
mod alternation;
pub use alternation::*;
mod affix;
pub use affix::*;
use iregex_automata::{
	nfa::{BuildNFA, StateBuilder},
	NFA,
};

use crate::CompiledRegEx;

/// Intermediate Regular Expression.
pub struct IRegEx {
	pub root: Alternation,
	pub prefix: Affix,
	pub suffix: Affix,
}

impl IRegEx {
	pub fn anchored(root: Alternation) -> Self {
		Self {
			root,
			prefix: Affix::Anchor,
			suffix: Affix::Anchor,
		}
	}

	pub fn unanchored(root: Alternation) -> Self {
		Self {
			root,
			prefix: Affix::Any,
			suffix: Affix::Any,
		}
	}

	/// Compiles the regular expression.
	pub fn compile<Q, S>(&self, mut state_builder: S) -> Result<CompiledRegEx<NFA<Q>>, S::Error>
	where
		Q: Copy + Ord,
		S: StateBuilder<Q>,
	{
		Ok(CompiledRegEx {
			root: self.root.build_nfa(&mut state_builder)?,
			prefix: self.prefix.build_nfa(&mut state_builder)?,
			suffix: self.suffix.build_nfa(&mut state_builder)?,
		})
	}
}

/// Capture group identifier.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CaptureGroupId(pub u32);

/// Repetition.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Repeat {
	pub min: u32,
	pub max: Option<u32>,
}

impl Repeat {
	const STAR: Self = Self { min: 0, max: None };

	pub fn is_zero(&self) -> bool {
		match self.max {
			Some(max) => max <= self.min,
			None => false,
		}
	}

	pub fn is_one(&self) -> bool {
		self.min == 1 && self.max == Some(1)
	}

	pub fn build_nfa_for<Q: Copy + Ord, S: StateBuilder<Q>>(
		self,
		value: &impl BuildNFA<Q>,
		state_builder: &mut S,
		nfa: &mut NFA<Q>,
	) -> Result<(Q, Q), S::Error> {
		if self.is_zero() {
			let a = state_builder.next_state(nfa)?;
			Ok((a, a))
		} else if self.is_one() {
			value.build_nfa_from(state_builder, nfa)
		} else {
			if self.min > 0 {
				let (a, b) = value.build_nfa_from(state_builder, nfa)?;
				let (c, d) = Self {
					min: self.min - 1,
					max: self.max.map(|max| max - 1),
				}
				.build_nfa_for(value, state_builder, nfa)?;
				nfa.add(b, None, c);
				Ok((a, d))
			} else {
				match self.max {
					Some(max) => {
						let a = state_builder.next_state(nfa)?;
						let b = state_builder.next_state(nfa)?;
						let (c, d) = Self {
							min: 0,
							max: Some(max - 1),
						}
						.build_nfa_for(value, state_builder, nfa)?;
						nfa.add(a, None, b);
						nfa.add(a, None, c);
						nfa.add(d, None, b);
						Ok((a, b))
					}
					None => {
						let q = state_builder.next_state(nfa)?;
						let (a, b) = value.build_nfa_from(state_builder, nfa)?;
						nfa.add(q, None, q);
						nfa.add(q, None, a);
						nfa.add(b, None, q);
						Ok((q, q))
					}
				}
			}
		}
	}
}
