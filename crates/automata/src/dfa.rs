use std::{
	collections::{hash_map::Entry, BTreeMap, BTreeSet, HashMap, HashSet},
	hash::Hash,
};

use btree_range_map::AnyRange;

/// Deterministic finite automaton.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DFA<Q, L = AnyRange<char>> {
	initial_state: Q,
	final_states: BTreeSet<Q>,
	transitions: DetTransitions<Q, L>,
}

impl<Q, L> DFA<Q, L> {
	/// Creates a new empty deterministic finite automaton.
	pub fn new(initial_state: Q) -> Self {
		Self {
			initial_state,
			final_states: BTreeSet::new(),
			transitions: DetTransitions(BTreeMap::new()),
		}
	}

	/// Creates a new DFA from its internal representation.
	pub fn from_parts(
		initial_state: Q,
		final_states: BTreeSet<Q>,
		transitions: DetTransitions<Q, L>,
	) -> Self {
		Self {
			initial_state,
			final_states,
			transitions,
		}
	}

	/// Returns the initial state of the automaton.
	pub fn initial_state(&self) -> &Q {
		&self.initial_state
	}

	/// Returns the final states of the automaton.
	pub fn final_states(&self) -> &BTreeSet<Q> {
		&self.final_states
	}

	/// Returns the transitions of the automaton.
	pub fn transitions(&self) -> &BTreeMap<Q, BTreeMap<L, Q>> {
		&self.transitions.0
	}

	/// Returns an iterator over all the states reachable from the given
	/// starting state `q`.
	pub fn reachable_states_from<'a>(&'a self, q: &'a Q) -> ReachableStates<'a, Q, L> {
		ReachableStates::new(self, q)
	}
}

impl<Q: Ord, L: Ord> DFA<Q, L> {
	pub fn is_initial_state(&self, q: &Q) -> bool {
		self.initial_state == *q
	}

	pub fn is_final_state(&self, q: &Q) -> bool {
		self.final_states.contains(q)
	}

	pub fn add_final_state(&mut self, q: Q) -> bool {
		self.final_states.insert(q)
	}

	pub fn declare_state(&mut self, q: Q) {
		self.transitions.0.entry(q).or_default();
	}

	pub fn transitions_from(&self, q: &Q) -> impl '_ + Iterator<Item = (&'_ L, &'_ Q)> {
		self.transitions.0.get(q).into_iter().flatten()
	}

	pub fn successors(&self, q: &Q) -> DetSuccessors<Q, L> {
		DetSuccessors::new(self.transitions.0.get(q))
	}

	pub fn add(&mut self, source: Q, label: L, target: Q) {
		self.transitions
			.0
			.entry(source)
			.or_default()
			.insert(label, target);
	}

	pub fn select_states<F>(&self, f: F) -> BTreeSet<&Q>
	where
		Q: Hash + Eq,
		F: Fn(&Q) -> bool,
	{
		let mut set = BTreeSet::new();
		let mut visited = HashSet::new();
		self.select_states_from(&self.initial_state, &f, &mut visited, &mut set);
		set
	}

	pub fn states(&self) -> BTreeSet<&Q>
	where
		Q: Hash + Eq,
	{
		self.select_states(|_| true)
	}

	fn select_states_from<'a, F>(
		&'a self,
		q: &'a Q,
		f: &F,
		visited: &mut HashSet<&'a Q>,
		set: &mut BTreeSet<&'a Q>,
	) where
		Q: Hash + Eq,
		F: Fn(&Q) -> bool,
	{
		if visited.insert(q) {
			if f(q) {
				set.insert(q);
			}

			for (_, r) in self.successors(q) {
				self.select_states_from(r, f, visited, set)
			}
		}
	}

	/// Creates a partition of the automaton's states.
	pub fn partition<P, F>(&self, f: F) -> HashMap<P, BTreeSet<&Q>>
	where
		Q: Ord + Hash + Eq,
		P: Hash + Eq,
		F: Fn(&Q) -> P,
	{
		unsafe {
			self.try_partition::<P, _, std::convert::Infallible>(|q| Ok(f(q)))
				.unwrap_unchecked() // safe because infallible.
		}
	}

	/// Creates a partition of the automaton's states.
	pub fn try_partition<P, F, E>(&self, f: F) -> Result<HashMap<P, BTreeSet<&Q>>, E>
	where
		Q: Ord + Hash + Eq,
		P: Hash + Eq,
		F: Fn(&Q) -> Result<P, E>,
	{
		let mut partition = HashMap::new();
		let mut visited = HashSet::new();
		self.try_partition_from(&self.initial_state, &f, &mut visited, &mut partition)?;
		Ok(partition)
	}

	fn try_partition_from<'a, P, F, E>(
		&'a self,
		q: &'a Q,
		f: &F,
		visited: &mut HashSet<&'a Q>,
		partition: &mut HashMap<P, BTreeSet<&'a Q>>,
	) -> Result<(), E>
	where
		Q: Ord + Hash + Eq,
		P: Hash + Eq,
		F: Fn(&Q) -> Result<P, E>,
	{
		if visited.insert(q) {
			let p = f(q)?;

			partition.entry(p).or_default().insert(q);

			for (_, r) in self.successors(q) {
				self.try_partition_from(r, f, visited, partition)?;
			}
		}

		Ok(())
	}

	/// Minimizes the automaton.
	// Hopcroft's algorithm.
	// https://en.wikipedia.org/wiki/DFA_minimization
	pub fn minimize<'a, P>(&'a self, partition: P) -> DFA<BTreeSet<&Q>, &L>
	where
		Q: Hash,
		L: Hash,
		P: Iterator<Item = BTreeSet<&'a Q>>,
	{
		let mut partition: BTreeSet<_> = partition.collect();

		let mut working = partition.clone();

		while let Some(a) = working.pop_first() {
			let mut sources_by_label: HashMap<&L, BTreeSet<&Q>> = HashMap::new();

			for (source, targets) in &self.transitions.0 {
				for (label, target) in targets {
					if a.contains(target) {
						if sources_by_label.contains_key(label) {
							let sources = sources_by_label.get_mut(label).unwrap();
							sources.insert(source);
						} else {
							let mut sources = BTreeSet::new();
							sources.insert(source);
							sources_by_label.insert(label, sources);
						}
					}
				}
			}

			for sources in sources_by_label.values() {
				for y in partition.clone() {
					if y.intersection(sources).next().is_some()
						&& y.difference(sources).next().is_some()
					{
						let intersection: BTreeSet<&Q> = y.intersection(sources).cloned().collect();
						let difference: BTreeSet<&Q> = y.difference(sources).cloned().collect();

						if working.contains(&y) {
							working.remove(&y);
							working.insert(intersection.clone());
							working.insert(difference.clone());
						} else if intersection.len() <= difference.len() {
							working.insert(intersection.clone());
						} else {
							working.insert(difference.clone());
						}

						partition.remove(&y);
						partition.insert(intersection);
						partition.insert(difference);
					}
				}
			}
		}

		let mut map = HashMap::new();
		for member in partition {
			for q in &member {
				map.insert(*q, member.clone());
			}
		}

		let mut result = DFA::new(map[&self.initial_state].clone());
		for (source, transitions) in &self.transitions.0 {
			for (range, target) in transitions {
				result.add(map[source].clone(), range, map[target].clone());
			}
		}

		result
	}

	pub fn map<P, M>(&self, mut f: impl FnMut(&Q) -> P, mut g: impl FnMut(&L) -> M) -> DFA<P, M>
	where
		Q: Hash,
		L: Hash,
		P: Clone + Ord + Hash,
		M: Clone + Ord + Hash,
	{
		let mut map = HashMap::new();
		let mapped_initial_state = f(&self.initial_state);
		map.insert(&self.initial_state, mapped_initial_state.clone());

		let mut label_map = HashMap::new();

		let mut result = DFA::new(mapped_initial_state);
		for (source, transitions) in &self.transitions.0 {
			for (range, target) in transitions {
				let source = map.entry(source).or_insert_with(|| f(source)).clone();
				let target = map.entry(target).or_insert_with(|| f(target)).clone();
				let range = label_map.entry(range).or_insert_with(|| g(range)).clone();
				result.add(source, range, target);
			}
		}

		for q in &self.final_states {
			let q = map.entry(q).or_insert_with(|| f(q)).clone();
			result.add_final_state(q);
		}

		result
	}

	pub fn try_map<P, M, E>(
		&self,
		mut f: impl FnMut(&Q) -> Result<P, E>,
		mut g: impl FnMut(&L) -> Result<M, E>,
	) -> Result<DFA<P, M>, E>
	where
		Q: Hash,
		L: Hash,
		P: Clone + Ord + Hash,
		M: Clone + Ord + Hash,
	{
		let mut map = HashMap::new();
		let mapped_initial_state = f(&self.initial_state)?;
		map.insert(&self.initial_state, mapped_initial_state.clone());

		let mut label_map: HashMap<&L, M> = HashMap::new();

		let mut result = DFA::new(mapped_initial_state);
		for (source, transitions) in &self.transitions.0 {
			for (label, target) in transitions {
				let source = match map.entry(source) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => entry.insert(f(source)?).clone(),
				};

				let target = match map.entry(target) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => entry.insert(f(target)?).clone(),
				};

				let label = match label_map.entry(label) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => entry.insert(g(label)?).clone(),
				};

				result.add(source, label, target);
			}
		}

		Ok(result)
	}

	pub fn product<'a, 'b, R, S, M, N>(
		&'a self,
		other: &'b DFA<R, M>,
		mut f: impl FnMut(&'a Q, &'b R) -> S,
		mut g: impl FnMut(&'a L, &'b M) -> Option<N>,
	) -> DFA<S, N>
	where
		R: Ord,
		S: Clone + Ord + Hash,
		M: Ord,
		N: Ord,
	{
		let mut stack = Vec::new();
		let initial_state = f(&self.initial_state, &other.initial_state);
		stack.push((
			initial_state.clone(),
			&self.initial_state,
			&other.initial_state,
		));
		let mut result = DFA::new(initial_state);

		let mut visited = HashSet::new();
		while let Some((q, a, b)) = stack.pop() {
			if visited.insert(q.clone()) {
				if self.is_final_state(a) && other.is_final_state(b) {
					result.add_final_state(q.clone());
				}

				let transitions = result.transitions.0.entry(q).or_default();

				for (a_label, sa) in self.successors(a) {
					for (b_label, sb) in other.successors(b) {
						if let Some(label) = g(a_label, b_label) {
							let s = f(sa, sb);
							stack.push((s.clone(), sa, sb));
							transitions.insert(label, s);
						}
					}
				}
			}
		}

		result
	}

	/// Returns the single transition that follows the state `q`.
	///
	/// Returns `None` if the state has no transitions, or multiple transitions.
	fn single_transition_of(&self, q: &Q) -> Option<(&L, &Q)> {
		let mut transitions = self.transitions().get(q)?.iter();
		let first = transitions.next()?;

		match transitions.next() {
			Some(_) => None,
			None => Some(first),
		}
	}

	/// Compress the transitions of a the automaton.
	///
	/// # Example
	///
	/// ```
	/// # use ere_automata::DFA;
	/// # let dfa = DFA::new(0);
	/// let _: DFA<_, String> = dfa.compress(|s: &mut String, c: &char| s.push(*c));
	/// ```
	pub fn compress<M>(&self, append: impl Fn(&mut M, &L)) -> DFA<Q, M>
	where
		Q: Clone,
		M: Default + Ord + Clone,
	{
		let mut transitions = BTreeMap::new();
		let mut stack = vec![&self.initial_state];

		while let Some(q) = stack.pop() {
			if !transitions.contains_key(q) {
				let mut q_transitions = BTreeMap::new();

				for (label, mut r) in self.transitions.0.get(q).into_iter().flatten() {
					let mut compact_label = M::default();
					append(&mut compact_label, label);

					while let Some((label, s)) = self.single_transition_of(r) {
						if self.is_final_state(r) {
							q_transitions.insert(compact_label.clone(), r.clone());
						}

						append(&mut compact_label, label);
						r = s;
					}

					q_transitions.insert(compact_label, r.clone());
				}

				transitions.insert(q.clone(), q_transitions);
			}
		}

		DFA::from_parts(
			self.initial_state.clone(),
			self.final_states.clone(),
			transitions.into(),
		)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetTransitions<Q, L>(BTreeMap<Q, BTreeMap<L, Q>>);

impl<Q, L> DetTransitions<Q, L> {
	pub fn len(&self) -> usize {
		self.0.values().fold(0, |x, map| x + map.len())
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl<Q, L> From<BTreeMap<Q, BTreeMap<L, Q>>> for DetTransitions<Q, L> {
	fn from(value: BTreeMap<Q, BTreeMap<L, Q>>) -> Self {
		Self(value)
	}
}

pub struct DetSuccessors<'a, Q, L> {
	inner: Option<std::collections::btree_map::Iter<'a, L, Q>>,
}

impl<'a, Q, L> DetSuccessors<'a, Q, L> {
	pub fn new(map: Option<&'a BTreeMap<L, Q>>) -> Self {
		Self {
			inner: map.map(|map| map.iter()),
		}
	}
}

impl<'a, Q, L> Iterator for DetSuccessors<'a, Q, L> {
	type Item = (&'a L, &'a Q);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.as_mut().and_then(|inner| inner.next())
	}
}

pub struct ReachableStates<'a, Q, L = AnyRange<char>> {
	aut: &'a DFA<Q, L>,
	visited: HashSet<&'a Q>,
	stack: Vec<&'a Q>,
}

impl<'a, Q, L> ReachableStates<'a, Q, L> {
	fn new(aut: &'a DFA<Q, L>, q: &'a Q) -> Self {
		Self {
			aut,
			visited: HashSet::new(),
			stack: vec![q],
		}
	}
}

impl<'a, Q, L> Iterator for ReachableStates<'a, Q, L>
where
	Q: Ord + Eq + Hash,
{
	type Item = &'a Q;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.stack.pop() {
				Some(q) => {
					if self.visited.insert(q) {
						if let Some(q_transitions) = self.aut.transitions.0.get(q) {
							for target in q_transitions.values() {
								self.stack.push(target)
							}
						}

						break Some(q);
					}
				}
				None => break None,
			}
		}
	}
}
