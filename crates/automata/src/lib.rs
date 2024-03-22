//! This library provides an implementation of Nondeterministic Finite Automata
//! (NFA) and Deterministic Finite Automata (DFA) for Unicode scalar values
//! (the [`char`] type). It is used by the [`ere`] crate to represent compiled
//! regular expressions.
//!
//! [`ere`]: <https://github.com/timothee-haudebourg/ere-rs>
pub use btree_range_map::{AnyRange, RangeSet};

pub mod nfa;
pub use nfa::NFA;

pub mod dfa;
pub use dfa::DFA;

/// Computes the intersection of two character sets.
pub fn charset_intersection(a: &RangeSet<char>, b: &RangeSet<char>) -> RangeSet<char> {
	let mut result = a.clone();

	for r in b.gaps() {
		result.remove(r.cloned());
	}

	result
}
