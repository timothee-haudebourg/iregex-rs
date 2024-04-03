use std::ops::Range;

use iregex::{Alternation, Atom, Concatenation, IRegEx};
use iregex_automata::{any_char, nfa::U32StateBuilder, RangeSet};

#[test]
fn no_matches_anchored() {
	let vectors = [(
		Atom::star(Atom::Token(['a', 'b', 'c'].into_iter().collect()).into()).into(),
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
		(Concatenation::new().into(), ""),
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

	for (root, haystack, expected) in vectors {
		let ire = IRegEx::unanchored(root);
		let aut = ire.compile(U32StateBuilder::default()).unwrap();
		let matches: Vec<_> = aut.matches(haystack.chars()).collect();
		assert_eq!(matches, expected);
	}
}
