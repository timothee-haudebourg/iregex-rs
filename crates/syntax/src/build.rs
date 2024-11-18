use iregex::automata::{any_char, AnyRange, RangeSet};

use crate::{Ast, Atom, Charset, Class, Classes, Disjunction, Repeat, Sequence};

impl Ast {
	pub fn build(&self) -> iregex::IRegEx {
		let root = self.disjunction.build();

		iregex::IRegEx {
			root,
			prefix: if self.start_anchor {
				iregex::Affix::Anchor
			} else {
				iregex::Affix::Any
			},
			suffix: if self.end_anchor {
				iregex::Affix::Anchor
			} else {
				iregex::Affix::Any
			},
		}
	}
}

impl Disjunction {
	pub fn build(&self) -> iregex::Alternation {
		self.iter().map(Sequence::build).collect()
	}
}

impl Sequence {
	pub fn build(&self) -> iregex::Concatenation {
		self.iter().map(Atom::build).collect()
	}
}

impl Atom {
	pub fn build(&self) -> iregex::Atom {
		match self {
			Self::Any => iregex::Atom::Token(any_char()),
			Self::Char(c) => iregex::Atom::Token(RangeSet::from_iter([*c])),
			Self::Set(set) => iregex::Atom::Token(set.build()),
			Self::Group(g) => iregex::Atom::alternation(g.build()),
			Self::Repeat(atom, repeat) => iregex::Atom::Repeat(atom.build().into(), repeat.build()),
		}
	}
}

impl Classes {
	pub fn build(&self) -> iregex::automata::RangeSet<char> {
		let mut result = iregex::automata::RangeSet::new();

		for c in self {
			result.extend(c.build());
		}

		result
	}
}

impl Class {
	pub fn build(&self) -> iregex::automata::RangeSet<char> {
		todo!()
	}
}

impl Charset {
	pub fn build(&self) -> iregex::automata::RangeSet<char> {
		let mut result = self.set.clone();
		result.extend(self.classes.build());

		if self.negative {
			return result.gaps().map(AnyRange::cloned).collect();
		} else {
			result
		}
	}
}

impl Repeat {
	pub fn build(&self) -> iregex::Repeat {
		iregex::Repeat {
			min: self.min,
			max: self.max,
		}
	}
}
