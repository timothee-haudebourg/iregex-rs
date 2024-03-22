use core::fmt;
use ere_automata::AnyRange;

use crate::Ast;

const CHAR_COUNT: u64 = 0xd7ff + 0x10ffff - 0xe000;

impl Ast {
	/// Display this regular expression as a sub expression.
	///
	/// This will enclose it between parenthesis if necessary.
	pub fn display_sub(&self) -> DisplaySub {
		DisplaySub(self)
	}
}

impl fmt::Display for Ast {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Any => write!(f, "."),
			Self::Set(charset) => {
				if charset.len() == 1 {
					let c = charset.iter().next().unwrap().first().unwrap();
					fmt_char(c, f)
				} else {
					write!(f, "[")?;
					if charset.len() > CHAR_COUNT / 2 {
						write!(f, "^")?;
						for range in charset.gaps() {
							fmt_range(range.cloned(), f)?
						}
					} else {
						for range in charset {
							fmt_range(*range, f)?
						}
					}

					write!(f, "]")
				}
			}
			Self::Sequence(seq) => {
				for item in seq {
					if seq.len() > 1 {
						item.display_sub().fmt(f)?
					} else {
						item.fmt(f)?
					}
				}

				Ok(())
			}
			Self::Repeat(e, 0, 1) => write!(f, "{}?", e.display_sub()),
			Self::Repeat(e, 0, u32::MAX) => write!(f, "{}*", e.display_sub()),
			Self::Repeat(e, 1, u32::MAX) => write!(f, "{}+", e.display_sub()),
			Self::Repeat(e, min, u32::MAX) => write!(f, "{}{{{},}}", e.display_sub(), min),
			Self::Repeat(e, 0, max) => write!(f, "{}{{,{}}}", e.display_sub(), max),
			Self::Repeat(e, min, max) => {
				if min == max {
					write!(f, "{}{{{}}}", e.display_sub(), min)
				} else {
					write!(f, "{}{{{},{}}}", e.display_sub(), min, max)
				}
			}
			Self::Union(items) => {
				for (i, item) in items.iter().enumerate() {
					if i > 0 {
						write!(f, "|")?
					}

					item.display_sub().fmt(f)?
				}

				Ok(())
			}
		}
	}
}

/// Display the inner regular expression as a sub expression.
///
/// This will enclose it between parenthesis if necessary.
pub struct DisplaySub<'a>(&'a Ast);

impl<'a> fmt::Display for DisplaySub<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.0.is_simple() {
			self.0.fmt(f)
		} else {
			write!(f, "({})", self.0)
		}
	}
}

fn fmt_range(range: AnyRange<char>, f: &mut fmt::Formatter) -> fmt::Result {
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

fn fmt_char(c: char, f: &mut fmt::Formatter) -> fmt::Result {
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
