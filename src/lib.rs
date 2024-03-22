pub mod regexp;
pub mod automaton;

use btree_range_map::RangeSet;
pub use regexp::RegExp;
pub use automaton::{Automaton, DetAutomaton};

/// Computes the intersection of two character sets.
pub fn charset_intersection(a: &RangeSet<char>, b: &RangeSet<char>) -> RangeSet<char> {
	let mut result = a.clone();

	for r in b.gaps() {
		result.remove(r.cloned());
	}

	result
}