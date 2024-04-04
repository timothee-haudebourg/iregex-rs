use std::{fs, ops::Range};

use iregex::{Alternation, Atom, CompoundAutomaton, Concatenation, IRegEx};
use iregex_automata::{any_char, dot::DotDisplay, nfa::U32StateBuilder, Map, RangeSet, NFA};

#[test]
fn no_matches_anchored() {
	let vectors = [(
		Atom::<_, ()>::star(Atom::Token(['a', 'b', 'c'].into_iter().collect()).into()).into(),
		"abcd",
	)];

	for (root, haystack) in vectors {
		let ire = IRegEx::anchored(root);
		let aut = ire.compile(U32StateBuilder::default()).unwrap();
		let mut matches = aut.matches(haystack.chars());
		assert_eq!(matches.next(), None);
	}
}

#[test]
fn single_match_anchored() {
	let vectors = [
		(Concatenation::<_, ()>::new().into(), ""),
		(Atom::Token(any_char()).into(), "a"),
		(Atom::star(Atom::Token(any_char()).into()).into(), "abcd"),
	];

	for (root, haystack) in vectors {
		let ire = IRegEx::anchored(root);
		let aut = ire.compile(U32StateBuilder::default()).unwrap();
		let mut matches = aut.matches(haystack.chars());
		assert_eq!(matches.next(), Some(0..haystack.len()));
		assert_eq!(matches.next(), None);
	}
}

#[test]
fn single_match_unanchored() {
	let b: RangeSet<char> = ['b'].into_iter().collect();

	let vectors = [
		(Atom::Token(b.clone()).into(), "aba", 1..2),
		(
			[Atom::Token(b.clone()).into(), Atom::Token(b).into()]
				.into_iter()
				.collect::<Concatenation>()
				.into(),
			"abba",
			1..3,
		),
	];

	for (root, haystack, expected) in vectors {
		let ire = IRegEx::unanchored(root);
		let aut = ire.compile(U32StateBuilder::default()).unwrap();
		let mut matches = aut.matches(haystack.chars());
		assert_eq!(matches.next(), Some(expected));
		assert_eq!(matches.next(), None);
	}
}

#[test]
fn many_matches_unanchored() {
	let a = Atom::Token(['a'].into_iter().collect());
	let b = Atom::Token(['b'].into_iter().collect());

	let vectors: [(Alternation, &str, &[Range<usize>]); 3] = [
		(
			Concatenation::new().into(),
			"aaa",
			&[0..0, 1..1, 2..2, 3..3],
		),
		(Atom::Token(any_char()).into(), "aaa", &[0..1, 1..2, 2..3]),
		(
			[Concatenation::from(a), Concatenation::from(b)]
				.into_iter()
				.collect(),
			"abab",
			&[0..1, 1..2, 2..3, 3..4],
		),
	];

	for (i, (root, haystack, expected)) in vectors.into_iter().enumerate() {
		let ire = IRegEx::unanchored(root);
		let aut = ire.compile(U32StateBuilder::default()).unwrap();
		let matches: Vec<_> = aut.matches(haystack.chars()).collect();

		if matches != expected {
			write_compound_automaton(format!("many_matches_unanchored_{i}"), &aut);
		}

		assert_eq!(matches, expected);
	}
}

fn write_compound_automaton(basename: String, aut: &CompoundAutomaton) {
	write_automaton(format!("{basename}_prefix.dot"), &aut.prefix);
	write_automaton(format!("{basename}_root.dot"), &aut.root.get(&()).unwrap());
	write_automaton(
		format!("{basename}_suffix.dot"),
		&aut.suffix.get(&()).unwrap(),
	);
}

fn write_automaton(path: String, aut: &NFA) {
	fs::write(&path, aut.dot().to_string()).unwrap();
}
