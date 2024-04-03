use core::fmt;
use iregex_automata::AnyRange;
use std::fmt::Write;

use crate::{Ast, Atom, Charset, Disjunction, Repeat, Sequence};

impl fmt::Display for Ast {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.start_anchor {
			f.write_char('^')?;
		}

		self.disjunction.fmt(f)?;

		if self.end_anchor {
			f.write_char('$')
		} else {
			Ok(())
		}
	}
}

impl fmt::Display for Disjunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, sequence) in self.iter().enumerate() {
			if i > 0 {
				f.write_char('|')?;
			}

			sequence.fmt(f)?;
		}

		Ok(())
	}
}

impl fmt::Display for Sequence {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for atom in self {
			atom.fmt(f)?;
		}

		Ok(())
	}
}

impl fmt::Display for Atom {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Any => f.write_char('.'),
			Self::Char(c) => fmt_char(*c, f),
			Self::Set(charset) => charset.fmt(f),
			Self::Repeat(atom, repeat) => {
				atom.fmt(f)?;
				repeat.fmt(f)
			}
			Self::Group(g) => {
				f.write_char('(')?;
				g.fmt(f)?;
				f.write_char(')')
			}
		}
	}
}

impl fmt::Display for Charset {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.negative {
			f.write_char('^')?;
		}

		for &range in &self.set {
			fmt_range(range, f)?
		}

		Ok(())
	}
}

impl fmt::Display for Repeat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.min == 0 && self.max == 1 {
			f.write_char('?')
		} else if self.min == 0 && self.max == u32::MAX {
			f.write_char('*')
		} else if self.min == 1 && self.max == u32::MAX {
			f.write_char('+')
		} else {
			write!(f, "{{{},{}}}", self.min, self.max)
		}
	}
}

pub fn fmt_range(range: AnyRange<char>, f: &mut fmt::Formatter) -> fmt::Result {
	if range.len() == 1 {
		fmt_char(range.first().unwrap(), f)
	} else {
		let a = range.first().unwrap();
		let b = range.last().unwrap();

		fmt_char(a, f)?;
		if a as u32 + 1 < b as u32 {
			write!(f, "-")?;
		}
		fmt_char(b, f)
	}
}

pub fn fmt_char(c: char, f: &mut fmt::Formatter) -> fmt::Result {
	match c {
		'(' => write!(f, "\\("),
		')' => write!(f, "\\)"),
		'[' => write!(f, "\\["),
		']' => write!(f, "\\]"),
		'{' => write!(f, "\\{{"),
		'}' => write!(f, "\\}}"),
		'?' => write!(f, "\\?"),
		'*' => write!(f, "\\*"),
		'+' => write!(f, "\\+"),
		'-' => write!(f, "\\-"),
		'^' => write!(f, "\\^"),
		'|' => write!(f, "\\|"),
		'\\' => write!(f, "\\\\"),
		'\0' => write!(f, "\\0"),
		'\x07' => write!(f, "\\a"),
		'\x08' => write!(f, "\\b"),
		'\t' => write!(f, "\\t"),
		'\n' => write!(f, "\\n"),
		'\x0b' => write!(f, "\\v"),
		'\x0c' => write!(f, "\\f"),
		'\r' => write!(f, "\\r"),
		'\x1b' => write!(f, "\\e"),
		_ => fmt::Display::fmt(&c, f),
	}
}

// #[cfg(test)]
// mod tests {
// 	// Each pair is of the form `(regexp, formatted)`.
// 	// We check that the regexp is correctly parsed by formatting it and
// 	// checking that it matches the expected `formatted` string.
// 	const TESTS: &[(&str, &str)] = &[
// 		("a*", "a*"),
// 		("a\\*", "a\\*"),
// 		("[cab]", "[a-c]"),
// 		("[^cab]", "[^a-c]"),
// 		("(abc)|de", "abc|de"),
// 		("(a|b)?", "(a|b)?"),
// 		("[A-Za-z0-89]", "[0-9A-Za-z]"),
// 		("[a|b]", "[ab\\|]"),
// 	];

// 	#[test]
// 	fn test() {
// 		for &(regexp, formatted) in TESTS {
// 			assert_eq!(
// 				super::Ast::parse(regexp.chars()).unwrap().to_string(),
// 				*formatted
// 			)
// 		}
// 	}
// }
