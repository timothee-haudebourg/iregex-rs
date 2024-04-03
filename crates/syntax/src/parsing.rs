use std::{borrow::Borrow, iter::Peekable, ops::Bound, str::FromStr};

use iregex_automata::{AnyRange, RangeSet};

use crate::{Ast, Atom, Charset, Class, Classes, Disjunction, Repeat, Sequence};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Unexpected(Unexpected),

	#[error("unexpected metacharacter `{0}`")]
	UnexpectedMetacharacter(char),

	#[error("invalid class name `{0}`")]
	InvalidClassName(String),

	#[error("overflow")]
	Overflow,
}

#[derive(Debug, thiserror::Error)]
pub enum Unexpected {
	#[error("unexpected end of stream")]
	EndOfStream,

	#[error("unexpected character `{0}`")]
	Char(char),
}

impl From<Option<char>> for Unexpected {
	fn from(value: Option<char>) -> Self {
		match value {
			Some(c) => Self::Char(c),
			None => Self::EndOfStream,
		}
	}
}

enum AtomOrRepeat {
	Atom(Atom),
	Repeat(Repeat),
}

impl Atom {
	pub fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Option<Self>, Error> {
		let result = match chars.peek().copied() {
			None | Some(')' | '|' | '$') => return Ok(None),
			Some(c @ ('^' | ']' | '}' | '?' | '*' | '+')) => {
				return Err(Error::UnexpectedMetacharacter(c))
			}
			Some('.') => {
				chars.next();
				Self::Any
			}
			Some('[') => {
				let charset = Charset::parse(chars)?;
				Self::Set(charset)
			}
			Some('(') => {
				chars.next();
				let group = Disjunction::parse(chars)?;
				match chars.next() {
					Some(')') => Self::Group(group),
					other => return Err(Error::Unexpected(other.into())),
				}
			}
			Some('\\') => {
				chars.next();
				let c = parse_escaped_char(chars)?;
				Self::Char(c)
			}
			Some(c) => {
				chars.next();
				Self::Char(c)
			}
		};

		Ok(Some(result))
	}
}

impl AtomOrRepeat {
	pub fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Option<Self>, Error> {
		let result = match chars.peek().copied() {
			None | Some(')' | '|' | '$') => return Ok(None),
			Some(c @ ('^' | ']' | '}')) => return Err(Error::UnexpectedMetacharacter(c)),
			Some('.') => {
				chars.next();
				Self::Atom(Atom::Any)
			}
			Some('[') => {
				let charset = Charset::parse(chars)?;
				Self::Atom(Atom::Set(charset))
			}
			Some('(') => {
				chars.next();
				let group = Disjunction::parse(chars)?;
				match chars.next() {
					Some(')') => Self::Atom(Atom::Group(group)),
					other => return Err(Error::Unexpected(other.into())),
				}
			}
			Some('{') => Self::Repeat(Repeat::parse(chars)?),
			Some('?') => {
				chars.next();
				Self::Repeat(Repeat { min: 0, max: 1 })
			}
			Some('*') => {
				chars.next();
				Self::Repeat(Repeat {
					min: 0,
					max: u32::MAX,
				})
			}
			Some('+') => {
				chars.next();
				Self::Repeat(Repeat {
					min: 1,
					max: u32::MAX,
				})
			}
			Some('\\') => {
				chars.next();
				let c = parse_escaped_char(chars)?;
				Self::Atom(Atom::Char(c))
			}
			Some(c) => {
				chars.next();
				Self::Atom(Atom::Char(c))
			}
		};

		Ok(Some(result))
	}
}

impl Sequence {
	pub fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self, Error> {
		match Atom::parse(chars)? {
			Some(atom) => {
				let mut result = vec![atom];

				while let Some(atom_or_repeat) = AtomOrRepeat::parse(chars)? {
					match atom_or_repeat {
						AtomOrRepeat::Atom(atom) => result.push(atom),
						AtomOrRepeat::Repeat(r) => result.last_mut().unwrap().repeat(r),
					}
				}

				Ok(Self(result))
			}
			None => Ok(Self::new()),
		}
	}
}

impl Disjunction {
	pub fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self, Error> {
		let mut result = vec![Sequence::parse(chars)?];
		while let Some(c) = chars.peek().copied() {
			match c {
				'|' => {
					chars.next();
					result.push(Sequence::parse(chars)?)
				}
				')' | '$' => break,
				c => return Err(Error::UnexpectedMetacharacter(c)),
			}
		}

		Ok(Self(result))
	}
}

impl Ast {
	pub fn parse(chars: impl IntoIterator<Item = char>) -> Result<Self, Error> {
		let mut chars = chars.into_iter().peekable();

		let start_anchor = match chars.peek().copied() {
			Some('^') => {
				chars.next();
				true
			}
			_ => false,
		};

		let inner = Disjunction::parse(&mut chars)?;

		let end_anchor = match chars.next() {
			Some('$') => true,
			None => false,
			Some(c) => return Err(Error::UnexpectedMetacharacter(c)),
		};

		Ok(Self {
			start_anchor,
			end_anchor,
			disjunction: inner,
		})
	}
}

impl FromStr for Ast {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s.chars())
	}
}

impl<S: Borrow<str>> From<S> for Ast {
	fn from(s: S) -> Self {
		let mut seq = Sequence::new();

		for c in s.borrow().chars() {
			seq.push(Atom::Char(c))
		}

		Self {
			start_anchor: true,
			end_anchor: true,
			disjunction: Disjunction(vec![seq]),
		}
	}
}

impl Class {
	fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self, Error> {
		match chars.next() {
			Some(':') => {
				let mut name = String::new();

				loop {
					match chars.next() {
						Some(':') => break,
						Some(
							c @ ('(' | ')' | '{' | '}' | '[' | ']' | '|' | '?' | '*' | '+' | '^'),
						) => return Err(Error::UnexpectedMetacharacter(c)),
						Some(c) => name.push(c),
						None => return Err(Error::Unexpected(Unexpected::EndOfStream)),
					}
				}

				match chars.next() {
					Some(']') => match Class::from_name(&name) {
						Some(class) => Ok(class),
						None => Err(Error::InvalidClassName(name)),
					},
					other => Err(Error::Unexpected(other.into())),
				}
			}
			other => Err(Error::Unexpected(other.into())),
		}
	}
}

enum RangeOrClass {
	Range(AnyRange<char>, bool),
	Class(Class),
}

impl RangeOrClass {
	fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Option<Self>, Error> {
		let start = match chars.next() {
			Some(']') => return Ok(None),
			Some('[') => {
				return Ok(Some(Self::Class(Class::parse(chars)?)));
			}
			Some(c) => c,
			None => return Err(Error::Unexpected(Unexpected::EndOfStream)),
		};

		let (end, minus) = match chars.peek().copied() {
			Some('-') => {
				chars.next();
				match chars.peek().copied() {
					Some(']') => (start, true),
					Some(c) => {
						chars.next();
						(c, false)
					}
					None => return Err(Error::Unexpected(Unexpected::EndOfStream)),
				}
			}
			_ => (start, false),
		};

		Ok(Some(Self::Range(
			AnyRange::new(Bound::Included(start), Bound::Included(end)),
			minus,
		)))
	}
}

impl Charset {
	fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self, Error> {
		match chars.next() {
			Some('[') => (),
			other => return Err(Error::Unexpected(other.into())),
		}

		let negative = match chars.peek().copied() {
			Some('^') => {
				chars.next();
				true
			}
			_ => false,
		};

		let mut classes = Classes::none();
		let mut set = RangeSet::new();

		while let Some(range_or_class) = RangeOrClass::parse(chars)? {
			match range_or_class {
				RangeOrClass::Range(range, and_minus) => {
					set.insert(range);
					if and_minus {
						set.insert('-');
					}
				}
				RangeOrClass::Class(class) => {
					classes.insert(class);
				}
			}
		}

		Ok(Self {
			negative,
			classes,
			set,
		})
	}
}

impl Repeat {
	fn parse(chars: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self, Error> {
		match chars.next() {
			Some('{') => (),
			other => return Err(Error::Unexpected(other.into())),
		}

		fn parse_number<T, C: Iterator<Item = char>>(
			chars: &mut Peekable<C>,
			f: impl FnOnce(&mut Peekable<C>, Option<u32>, char) -> Result<T, Error>,
		) -> Result<T, Error> {
			match chars.next() {
				Some(c) => match c.to_digit(10) {
					Some(mut value) => loop {
						match chars.next() {
							Some(c) => match c.to_digit(10) {
								Some(d) => {
									value = value.checked_mul(10).ok_or(Error::Overflow)?;
									value = value.checked_add(d).ok_or(Error::Overflow)?;
								}
								None => break f(chars, Some(value), c),
							},
							None => break Err(Error::Unexpected(Unexpected::EndOfStream)),
						}
					},
					None => f(chars, None, c),
				},
				None => Err(Error::Unexpected(Unexpected::EndOfStream)),
			}
		}

		parse_number(chars, |chars, value, next| match value {
			Some(min) => match next {
				',' => parse_number(chars, |_, value, next| {
					if next == '}' {
						let max = value.unwrap_or(u32::MAX);
						Ok(Self { min, max })
					} else {
						Err(Error::Unexpected(Unexpected::Char(next)))
					}
				}),
				'}' => Ok(Self { min, max: min }),
				c => Err(Error::Unexpected(Unexpected::Char(c))),
			},
			None => Err(Error::Unexpected(Unexpected::Char(next))),
		})
	}
}

fn parse_escaped_char(chars: &mut impl Iterator<Item = char>) -> Result<char, Error> {
	match chars.next() {
		Some(c) => match c {
			'0' => Ok('\0'),
			'a' => Ok('\x07'),
			'b' => Ok('\x08'),
			's' => Ok(' '),
			't' => Ok('\t'),
			'n' => Ok('\n'),
			'v' => Ok('\x0b'),
			'f' => Ok('\x0c'),
			'r' => Ok('\r'),
			'e' => Ok('\x1b'),
			c => Ok(c),
		},
		None => Err(Error::Unexpected(Unexpected::EndOfStream)),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_success() {
		const INPUTS: [&str; 19] = [
			"",
			"abc",
			"(abc)",
			"[abc]",
			"[^abc]",
			"[a^bc]",
			"[abc-]",
			"abc?",
			"abc*",
			"abc+",
			"abc|def",
			"(abc|def)",
			"(abc|def)*",
			"(abc|(def)?)*",
			"[[:alpha:]]",
			"(abc){12,}",
			"(abc){12,34}",
			"(abc){12}",
			"(abc){4294967295}",
		];

		for input in INPUTS {
			if let Err(e) = Ast::parse(input.chars()) {
				panic!("failed to parse `{input}` with {e:?}")
			}
		}
	}

	#[test]
	fn parse_failure() {
		const INPUTS: [&str; 13] = [
			"?",
			"(abc",
			"[[:abc:]]",
			"[abc",
			"[^abc",
			"abc)",
			"abc]",
			"(abc){,}",
			"(abc){,12}",
			"(abc){,12",
			"(abc){12,34",
			"(abc){12",
			"(abc){4294967296}",
		];

		for input in INPUTS {
			if let Ok(ast) = Ast::parse(input.chars()) {
				panic!("failed to reject `{input}`, parsed as {ast:?}")
			}
		}
	}
}
