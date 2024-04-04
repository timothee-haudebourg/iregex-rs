use btree_range_map::{AnyRange, RangeMap, RangeSet};
use std::{
	collections::{BTreeMap, BTreeSet, HashSet},
	hash::Hash,
};

use crate::{dfa::DetTransitions, Automaton, Class, Map, Token, DFA};

use super::token_set_intersection;

#[derive(Debug)]
pub struct TooManyStates;

/// State builder.
pub trait StateBuilder<T, Q, C = ()> {
	type Error;

	fn next_state(&mut self, nfa: &mut NFA<Q, T>, class: C) -> Result<Q, Self::Error>;

	fn class_of(&self, q: &Q) -> Option<&C>;
}

impl<'a, T, Q, C, S: StateBuilder<T, Q, C>> StateBuilder<T, Q, C> for &'a mut S {
	type Error = S::Error;

	fn next_state(&mut self, nfa: &mut NFA<Q, T>, class: C) -> Result<Q, Self::Error> {
		S::next_state(*self, nfa, class)
	}

	fn class_of(&self, q: &Q) -> Option<&C> {
		S::class_of(*self, q)
	}
}

pub struct U32StateBuilder<C> {
	states: Vec<C>,
	limit: u32,
}

impl<C> Default for U32StateBuilder<C> {
	fn default() -> Self {
		U32StateBuilder {
			states: Vec::new(),
			limit: u32::MAX,
		}
	}
}

impl<T, C> StateBuilder<T, u32, C> for U32StateBuilder<C> {
	type Error = TooManyStates;

	fn next_state(&mut self, nfa: &mut NFA<u32, T>, class: C) -> Result<u32, Self::Error> {
		let q = self.states.len() as u32;
		self.states.push(class);
		if self.states.len() as u32 > self.limit {
			Err(TooManyStates)
		} else {
			nfa.add_state(q);
			Ok(q)
		}
	}

	fn class_of(&self, q: &u32) -> Option<&C> {
		self.states.get(*q as usize)
	}
}

pub trait BuildNFA<T = char, Q = u32, C = ()>
where
	T: Clone,
	Q: Ord,
	C: Class<T>,
{
	fn build_nfa<S: StateBuilder<T, Q, C>>(
		&self,
		mut state_builder: S,
		class: C,
	) -> Result<NFA<Q, T>, S::Error> {
		let mut nfa = NFA::new();
		let (a, bs) = self.build_nfa_from(&mut state_builder, &mut nfa, &class)?;
		nfa.add_initial_state(a);
		for (_, b) in bs.into_entries() {
			nfa.add_final_state(b);
		}

		Ok(nfa)
	}

	fn build_nfa_from<S: StateBuilder<T, Q, C>>(
		&self,
		state_builder: &mut S,
		nfa: &mut NFA<Q, T>,
		class: &C,
	) -> Result<(Q, C::Map<Q>), S::Error>;
}

/// Nondeterministic state transitions.
pub type Transitions<T, Q> = BTreeMap<Option<RangeSet<T>>, BTreeSet<Q>>;

/// Nondeterministic finite automaton.
#[derive(Debug)]
pub struct NFA<Q = u32, T = char> {
	transitions: BTreeMap<Q, Transitions<T, Q>>,
	initial_states: BTreeSet<Q>,
	final_states: BTreeSet<Q>,
}

impl<T, Q> Default for NFA<Q, T> {
	fn default() -> Self {
		Self {
			transitions: BTreeMap::new(),
			initial_states: BTreeSet::new(),
			final_states: BTreeSet::new(),
		}
	}
}

impl<T, Q> NFA<Q, T> {
	/// Create a new empty nondeterministic finite automaton.
	pub fn new() -> Self {
		Self::default()
	}

	/// Returns the set of initial states.
	pub fn initial_states(&self) -> &BTreeSet<Q> {
		&self.initial_states
	}

	/// Returns the set of final states.
	pub fn final_states(&self) -> &BTreeSet<Q> {
		&self.final_states
	}

	/// Returns the set of final states.
	pub fn states(&self) -> impl Iterator<Item = &Q> {
		self.transitions.keys()
	}

	/// Returns an iterator over the transitions.
	pub fn transitions(&self) -> std::collections::btree_map::Iter<Q, Transitions<T, Q>> {
		self.transitions.iter()
	}
}

impl<T, Q: Ord> NFA<Q, T> {
	/// Get the successors of the given state.
	pub fn successors(&self, q: &Q) -> Successors<T, Q> {
		Successors::new(self.transitions.get(q))
	}

	/// Adds the given state into the automaton, even if it is not the source
	/// or destination of any transition.
	pub fn add_state(&mut self, q: Q) {
		self.transitions.entry(q).or_default();
	}

	/// Checks if the given state is an initial state.
	pub fn is_initial_state(&self, q: &Q) -> bool {
		self.initial_states.contains(q)
	}

	/// Sets the given state as an initial state.
	pub fn add_initial_state(&mut self, q: Q) -> bool {
		self.initial_states.insert(q)
	}

	/// Checks if the given state is a final state.
	pub fn is_final_state(&self, q: &Q) -> bool {
		self.final_states.contains(q)
	}

	/// Adds a final state to the automaton.
	pub fn add_final_state(&mut self, q: Q) -> bool {
		self.final_states.insert(q)
	}
}

impl<T: Token, Q: Ord> NFA<Q, T> {
	/// Adds the given transition to the automaton.
	pub fn add(&mut self, source: Q, label: Option<RangeSet<T>>, target: Q)
	where
		Q: Clone,
	{
		self.add_state(target.clone());
		self.transitions
			.entry(source)
			.or_default()
			.entry(label)
			.or_default()
			.insert(target);
	}

	/// Checks if this automaton can recognize the empty string.
	pub fn recognizes_empty(&self) -> bool {
		let mut stack: Vec<_> = self.initial_states.iter().collect();
		let mut visited = BTreeSet::new();

		while let Some(q) = stack.pop() {
			if visited.insert(q) {
				if self.is_final_state(q) {
					return true;
				}

				if let Some(transitions) = self.transitions.get(q) {
					if let Some(successors) = transitions.get(&None) {
						stack.extend(successors)
					}
				}
			}
		}

		false
	}

	/// Checks if this automaton recognizes exactly one string.
	pub fn is_singleton(&self) -> bool {
		if self.initial_states.len() > 1 {
			return false;
		}

		if let Some(mut q) = self.initial_states.first() {
			while let Some(q_transitions) = self.transitions.get(q) {
				if q_transitions.len() > 1 {
					return false;
				}

				match q_transitions.first_key_value() {
					Some((label, r)) => {
						if r.len() > 1 {
							return false;
						}

						match r.first() {
							Some(r) => match label {
								Some(range) if T::is_one(range.len()) => q = r,
								_ => return false,
							},
							None => break,
						}
					}
					None => break,
				}
			}
		}

		true
	}

	/// Returns the string recognized by this automaton if it is a singleton
	/// automaton (it recognizes exactly one string).
	///
	/// Returns `None` if this automaton recognizes no string, or more than one
	/// string.
	pub fn to_singleton(&self) -> Option<Vec<T>> {
		if self.initial_states.len() > 1 {
			return None;
		}

		let mut result = Vec::new();

		if let Some(mut q) = self.initial_states.first() {
			while let Some(q_transitions) = self.transitions.get(q) {
				if q_transitions.len() > 1 {
					return None;
				}

				match q_transitions.first_key_value() {
					Some((label, r)) => {
						if r.len() > 1 {
							return None;
						}

						match r.first() {
							Some(r) => match label {
								Some(range) if T::is_one(range.len()) => {
									let c = range.iter().next().unwrap().first().unwrap();
									result.push(c);
									q = r
								}
								_ => return None,
							},
							None => break,
						}
					}
					None => break,
				}
			}
		}

		Some(result)
	}

	fn modulo_epsilon_state<'a>(&'a self, qs: impl IntoIterator<Item = &'a Q>) -> BTreeSet<&'a Q> {
		let mut states = BTreeSet::new();
		let mut stack: Vec<_> = qs.into_iter().collect();

		while let Some(q) = stack.pop() {
			if states.insert(q) {
				// add states reachable trough epsilon-transitions.
				if let Some(transitions) = self.transitions.get(q) {
					if let Some(epsilon_qs) = transitions.get(&None) {
						for t in epsilon_qs {
							stack.push(t)
						}
					}
				}
			}
		}

		states
	}

	fn determinize_transitions_for(
		&self,
		states: &BTreeSet<&Q>,
	) -> BTreeMap<AnyRange<T>, BTreeSet<&Q>> {
		let mut map = RangeMap::new();

		for q in states {
			if let Some(transitions) = self.transitions.get(q) {
				for (label, targets) in transitions {
					if let Some(label) = label {
						for range in label.iter() {
							debug_assert!(!range.is_empty());

							map.update(
								*range,
								|current_target_states_opt: Option<&BTreeSet<&Q>>| {
									let mut current_target_states = match current_target_states_opt
									{
										Some(current_target_states) => {
											current_target_states.clone()
										}
										None => BTreeSet::new(),
									};

									for q in targets {
										current_target_states
											.extend(self.modulo_epsilon_state(Some(q)));
									}

									Some(current_target_states)
								},
							);

							assert!(map.get(range.first().unwrap()).is_some());
						}
					}
				}
			}
		}

		let mut simplified_map = BTreeMap::new();

		for (range, set) in map {
			debug_assert!(!range.is_empty());
			simplified_map.insert(range, set);
		}

		simplified_map
	}

	/// Turns this NFA into a DFA.
	pub fn determinize<'a, R>(
		&'a self,
		mut f: impl FnMut(&BTreeSet<&'a Q>) -> R,
	) -> DFA<R, AnyRange<T>>
	where
		R: Clone + Ord + Hash,
	{
		let mut transitions = BTreeMap::new();

		// create the initial deterministic state.
		let initial_state = self.modulo_epsilon_state(&self.initial_states);
		let mut final_states = BTreeSet::new();

		let mut visited_states = HashSet::new();
		let mut stack = vec![initial_state.clone()];
		while let Some(det_q) = stack.pop() {
			let r = f(&det_q);
			if visited_states.insert(r.clone()) {
				if det_q.iter().any(|q| self.final_states.contains(q)) {
					final_states.insert(r.clone());
				}

				let map = self.determinize_transitions_for(&det_q);

				let mut r_map = BTreeMap::new();
				for (label, next_det_q) in map {
					r_map.insert(label, f(&next_det_q));
					stack.push(next_det_q)
				}

				transitions.insert(r, r_map);
			}
		}

		DFA::from_parts(
			f(&initial_state),
			final_states,
			DetTransitions::from(transitions),
		)
	}

	/// Adds the given `other` automaton to `self`, mapping the other automaton
	/// states in the process.
	pub fn mapped_union<R>(&mut self, other: NFA<R, T>, f: impl Fn(R) -> Q) {
		for (q, transitions) in other.transitions {
			let this_transitions = self.transitions.entry(f(q)).or_default();
			for (label, targets) in transitions {
				this_transitions
					.entry(label)
					.or_default()
					.extend(targets.into_iter().map(&f));
			}
		}

		self.initial_states
			.extend(other.initial_states.into_iter().map(&f));
		self.final_states
			.extend(other.final_states.into_iter().map(f));
	}

	/// Adds the given `other` automaton to `self`.
	pub fn union<R>(&mut self, other: NFA<Q, T>) {
		self.mapped_union(other, |q| q)
	}

	/// Computes the product between `self` and `other`.
	///
	/// The input function `f` computes the product between two states.
	pub fn product<'a, 'b, R, S>(
		&'a self,
		other: &'b NFA<R, T>,
		mut f: impl FnMut(&'a Q, &'b R) -> S,
	) -> NFA<S, T>
	where
		R: Ord,
		S: Clone + Ord + Hash,
	{
		let mut result = NFA::new();

		let mut stack = Vec::with_capacity(self.initial_states.len() * other.initial_states.len());
		for a in &self.initial_states {
			for b in &other.initial_states {
				let q = f(a, b);
				stack.push((q.clone(), a, b));
				result.add_initial_state(q);
			}
		}

		let mut visited = HashSet::new();
		while let Some((q, a, b)) = stack.pop() {
			if visited.insert(q.clone()) {
				if self.is_final_state(a) && other.is_final_state(b) {
					result.add_final_state(q.clone());
				}

				let transitions = result.transitions.entry(q).or_default();

				for (a_label, a_successors) in self.successors(a) {
					match a_label {
						Some(a_label) => {
							for (b_label, b_successors) in other.successors(b) {
								if let Some(b_label) = b_label {
									let label = token_set_intersection(a_label, b_label);
									if !label.is_empty() {
										let successors =
											transitions.entry(Some(label)).or_default();

										for sa in a_successors {
											for sb in b_successors {
												let s = f(sa, sb);
												stack.push((s.clone(), sa, sb));
												successors.insert(s);
											}
										}
									}
								}
							}
						}
						None => {
							if let Some(b_successors) =
								other.transitions.get(b).and_then(|s| s.get(&None))
							{
								let successors = transitions.entry(None).or_default();

								for sa in a_successors {
									for sb in b_successors {
										let s = f(sa, sb);
										stack.push((s.clone(), sa, sb));
										successors.insert(s);
									}
								}
							}
						}
					}
				}
			}
		}

		result
	}
}

/// Iterator over the successors of a given state in a [`NFA`].
pub struct Successors<'a, T, Q> {
	inner: Option<std::collections::btree_map::Iter<'a, Option<RangeSet<T>>, BTreeSet<Q>>>,
}

impl<'a, T, Q> Successors<'a, T, Q> {
	pub fn new(map: Option<&'a BTreeMap<Option<RangeSet<T>>, BTreeSet<Q>>>) -> Self {
		Self {
			inner: map.map(|map| map.iter()),
		}
	}
}

impl<'a, T, Q> Iterator for Successors<'a, T, Q> {
	type Item = (&'a Option<RangeSet<T>>, &'a BTreeSet<Q>);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.as_mut().and_then(|inner| inner.next())
	}
}

impl<T: Token, Q: Ord + Hash> Automaton<T> for NFA<Q, T> {
	type State<'a> = VisitingState<'a, Q> where Self: 'a;

	fn initial_state(&self) -> Option<Self::State<'_>> {
		let mut stack = Vec::new();
		let mut states = HashSet::new();

		for r in &self.initial_states {
			states.insert(r);
			stack.push(r);
		}

		// epsilon-closure.
		while let Some(q) = stack.pop() {
			if let Some(q_transitions) = self.transitions.get(q) {
				if let Some(targets) = q_transitions.get(&None) {
					for r in targets {
						if states.insert(r) {
							stack.push(r);
						}
					}
				}
			}
		}

		if states.is_empty() {
			None
		} else {
			Some(VisitingState {
				states,
				next_states: HashSet::new(),
				stack,
			})
		}
	}

	fn next_state<'a>(
		&'a self,
		VisitingState {
			mut states,
			mut next_states,
			mut stack,
		}: Self::State<'a>,
		token: T,
	) -> Option<Self::State<'_>> {
		for &q in &states {
			if let Some(q_transitions) = self.transitions.get(q) {
				for (label, targets) in q_transitions {
					if let Some(label) = label {
						if label.contains(token) {
							for r in targets {
								if next_states.insert(r) {
									stack.push(r);
								}
							}
						}
					}
				}
			}
		}

		// epsilon-closure.
		while let Some(q) = stack.pop() {
			if let Some(q_transitions) = self.transitions.get(q) {
				if let Some(targets) = q_transitions.get(&None) {
					for r in targets {
						if next_states.insert(r) {
							stack.push(r);
						}
					}
				}
			}
		}

		if next_states.is_empty() {
			None
		} else {
			states.clear();
			eprintln!("next is {} len", next_states.len());
			Some(VisitingState {
				states: next_states,
				next_states: states,
				stack,
			})
		}
	}

	fn is_final_state<'a>(&'a self, VisitingState { states, .. }: &Self::State<'a>) -> bool {
		for &q in states {
			if self.final_states.contains(q) {
				return true;
			}
		}

		false
	}
}

pub struct VisitingState<'a, Q> {
	states: HashSet<&'a Q>,
	next_states: HashSet<&'a Q>,
	stack: Vec<&'a Q>,
}
