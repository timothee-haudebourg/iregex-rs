use std::hash::Hash;

mod boundary;
pub use boundary::*;
mod atom;
pub use atom::*;
mod concatenation;
pub use concatenation::*;
mod alternation;
pub use alternation::*;
mod affix;
pub use affix::*;
use iregex_automata::{
	nfa::{BuildNFA, StateBuilder, TaggedNFA, Tags},
	Class, Map, MapSource, Token, NFA,
};

use crate::CompoundAutomaton;

/// Intermediate Regular Expression.
#[derive(Debug)]
pub struct IRegEx<T = char, B = ()> {
	pub root: Alternation<T, B>,
	pub prefix: Affix<T, B>,
	pub suffix: Affix<T, B>,
}

impl<T, B> IRegEx<T, B> {
	pub fn anchored(root: Alternation<T, B>) -> Self {
		Self {
			root,
			prefix: Affix::Anchor,
			suffix: Affix::Anchor,
		}
	}

	pub fn unanchored(root: Alternation<T, B>) -> Self {
		Self {
			root,
			prefix: Affix::Any,
			suffix: Affix::Any,
		}
	}

	pub fn into_anchored(self) -> Result<Alternation<T, B>, Self> {
		if self.prefix.is_anchor() && self.suffix.is_anchor() {
			Ok(self.root)
		} else {
			Err(self)
		}
	}

	pub fn into_unanchored(self) -> Result<Alternation<T, B>, Self> {
		if self.prefix.is_any() && self.suffix.is_any() {
			Ok(self.root)
		} else {
			Err(self)
		}
	}

	/// Compiles the regular expression.
	pub fn compile<Q, S>(&self, mut state_builder: S) -> Result<CompiledRegEx<T, B, Q>, S::Error>
	where
		T: Token,
		B: Boundary<T>,
		B::Class: Default + Clone + Eq + Hash,
		Q: Copy + Ord,
		S: StateBuilder<T, Q, B::Class>,
	{
		let prefix = self
			.prefix
			.build_nfa(&mut state_builder, Default::default())?;

		let mut root: <B::Class as MapSource>::Map<TaggedNFA<Q, T, CaptureTag>> =
			Default::default();
		for q in prefix.final_states() {
			let q_class = state_builder.class_of(q).unwrap().clone();
			root.get_or_try_insert_with(&q_class, || {
				self.root.build_nfa(&mut state_builder, q_class.clone())
			})?;
		}

		let mut suffix: <B::Class as MapSource>::Map<TaggedNFA<Q, T, CaptureTag>> =
			Default::default();
		for (_, aut) in root.iter() {
			for q in aut.final_states() {
				let q_class = state_builder.class_of(q).unwrap().clone();
				suffix.get_or_try_insert_with(&q_class, || {
					self.suffix.build_nfa(&mut state_builder, q_class.clone())
				})?;
			}
		}

		Ok(CompoundAutomaton {
			root,
			prefix,
			suffix,
		})
	}
}

pub type CompiledRegEx<T, B, Q> =
	CompoundAutomaton<TaggedNFA<Q, T, CaptureTag>, <B as Boundary<T>>::Class>;

/// Capture group identifier.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CaptureGroupId(pub u32);

pub enum CaptureTag {
	Begin(CaptureGroupId),
	End(CaptureGroupId),
}

/// Repetition.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Repeat {
	pub min: u32,
	pub max: Option<u32>,
}

impl Repeat {
	pub const ONCE: Self = Self {
		min: 1,
		max: Some(1),
	};
	pub const STAR: Self = Self { min: 0, max: None };

	pub fn is_zero(&self) -> bool {
		match self.max {
			Some(max) => max < self.min,
			None => false,
		}
	}

	pub fn is_one(&self) -> bool {
		self.min == 1 && self.max == Some(1)
	}

	pub fn split_last(&self) -> Option<Self> {
		match self.max {
			Some(0) | Some(1) => None,
			_ => Some(Self {
				min: if self.min == 0 { 0 } else { self.min - 1 },
				max: self.max.map(|max| if max == 0 { 0 } else { max - 1 }),
			}),
		}
	}

	pub fn build_nfa_for<T, Q, C, G, S>(
		self,
		value: &impl BuildNFA<T, Q, C, G>,
		state_builder: &mut S,
		nfa: &mut NFA<Q, T>,
		tags: &mut Tags<Q, G>,
		class: &C,
	) -> Result<(Q, C::Map<Q>), S::Error>
	where
		T: Token,
		Q: Copy + Ord,
		C: Clone + Eq + Hash + Class<T>,
		S: StateBuilder<T, Q, C>,
	{
		if self.is_zero() {
			let a = state_builder.next_state(nfa, class.clone())?;
			Ok((a, Map::singleton(class.clone(), a)))
		} else if self.is_one() {
			value.build_nfa_from(state_builder, nfa, tags, class)
		} else if self.min > 0 {
			let (a, bs) = value.build_nfa_from(state_builder, nfa, tags, class)?;

			let mut output = ClassConcatenation::default();

			for (b_class, b) in bs.into_entries() {
				let (c, ds) = Self {
					min: self.min - 1,
					max: self.max.map(|max| max - 1),
				}
				.build_nfa_for(value, state_builder, nfa, tags, &b_class)?;
				nfa.add(b, None, c);

				for (_, d) in ds.into_entries() {
					output.insert(state_builder, nfa, d)?;
				}
			}

			Ok((a, output.into_map()))
		} else {
			match self.max {
				Some(max) => {
					// initial state.
					let a = state_builder.next_state(nfa, class.clone())?;

					// intermediate state.
					let (b, b_output) = value.build_nfa_from(state_builder, nfa, tags, class)?;

					// final state.
					let f = state_builder.next_state(nfa, class.clone())?;

					// we can go from initial to intermediate.
					nfa.add(a, None, b);

					// we can go directly from initial to final.
					nfa.add(a, None, f);

					let mut output = ClassAlternation::singleton(class.clone(), f);

					for (c_class, c) in b_output.into_entries() {
						if max > 0 {
							let (d, d_output) = Self {
								min: 0,
								max: Some(max - 1),
							}
							.build_nfa_for(value, state_builder, nfa, tags, &c_class)?;

							nfa.add(c, None, d);

							for (e_class, e) in d_output.into_entries() {
								// connect to final.
								let f = output.insert(state_builder, nfa, e_class)?;
								nfa.add(e, None, f);
							}
						} else {
							// connect to final.
							let f = output.insert(state_builder, nfa, c_class)?;
							nfa.add(c, None, f);
						}
					}

					Ok((a, output.into_map()))
				}
				None => {
					let mut map: C::Map<Q> = Default::default();
					let q = kleene_star_closure(&mut map, value, state_builder, nfa, tags, class)?;
					Ok((q, map))
				}
			}
		}
	}
}

fn kleene_star_closure<T, Q, C, G, S: StateBuilder<T, Q, C>>(
	map: &mut C::Map<Q>,
	value: &impl BuildNFA<T, Q, C, G>,
	state_builder: &mut S,
	nfa: &mut NFA<Q, T>,
	tags: &mut Tags<Q, G>,
	class: &C,
) -> Result<Q, S::Error>
where
	T: Token,
	Q: Copy + Ord,
	C: Clone + Eq + Hash + Class<T>,
{
	match map.get(class) {
		Some(q) => Ok(*q),
		None => {
			let q = state_builder.next_state(nfa, class.clone())?;
			map.set(class.clone(), q);
			nfa.add(q, None, q);

			let (a, bs) = value.build_nfa_from(state_builder, nfa, tags, class)?;
			nfa.add(q, None, a);

			for (b_class, b) in bs.into_entries() {
				let target = kleene_star_closure(map, value, state_builder, nfa, tags, &b_class)?;
				nfa.add(b, None, target);
			}

			Ok(q)
		}
	}
}

pub struct ClassAlternation<Q, C: MapSource>(C::Map<Q>);

impl<Q, C: MapSource> Default for ClassAlternation<Q, C> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<Q, C: MapSource> ClassAlternation<Q, C> {
	pub fn into_map(self) -> C::Map<Q> {
		self.0
	}
}

impl<Q, C: MapSource> ClassAlternation<Q, C>
where
	Q: Copy + Ord,
	C: Clone + Eq + Hash,
{
	pub fn singleton(class: C, q: Q) -> Self {
		Self([(class, q)].into_iter().collect())
	}

	pub fn insert<T, S: StateBuilder<T, Q, C>>(
		&mut self,
		state_builder: &mut S,
		nfa: &mut NFA<Q, T>,
		class: C,
	) -> Result<Q, S::Error> {
		match self.0.get(&class) {
			Some(b) => Ok(*b),
			None => {
				let b = state_builder.next_state(nfa, class.clone())?;
				self.0.set(class, b);
				Ok(b)
			}
		}
	}
}

pub struct ClassConcatenation<Q, C: MapSource>(C::Map<(Q, bool)>);

impl<Q, C: MapSource> Default for ClassConcatenation<Q, C> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<Q, C: MapSource> ClassConcatenation<Q, C> {
	pub fn into_map(self) -> C::Map<Q> {
		self.0.into_entries().map(|(c, (q, _))| (c, q)).collect()
	}
}

impl<Q, C: MapSource> ClassConcatenation<Q, C>
where
	Q: Copy + Ord,
	C: Clone + Eq + Hash,
{
	pub fn singleton(q: Q, class: C) -> Self {
		Self([(class, (q, false))].into_iter().collect())
	}

	pub fn insert<T: Token, S: StateBuilder<T, Q, C>>(
		&mut self,
		state_builder: &mut S,
		nfa: &mut NFA<Q, T>,
		q: Q,
	) -> Result<(), S::Error> {
		let class = state_builder.class_of(&q).unwrap().clone();
		let (r, merging) = self.0.get_mut_or_insert_with(&class, || (q, false));

		if *r != q {
			if *merging {
				nfa.add(q, None, *r);
			} else {
				let d = state_builder.next_state(nfa, class)?;
				nfa.add(q, None, d);
				nfa.add(*r, None, d);
				*r = d;
				*merging = true;
			}
		}

		Ok(())
	}
}
