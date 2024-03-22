use std::str::FromStr;

use ere_automata::RangeSet;

use crate::Ast;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("missing opening `(`")]
	UnmatchedClosingParenthesis,

	#[error("missing closing `)`")]
	MissingClosingParenthesis,

	#[error("incomplete escape sequence")]
	IncompleteEscapeSequence,

	#[error("incomplete character set")]
	IncompleteCharacterSet,

	#[error("nothing to repeat")]
	NothingToRepeat,
}

impl Ast {
	pub fn parse(s: &str) -> Result<Self, Error> {
		struct Disjunction(Vec<Vec<Ast>>);

		impl Disjunction {
			fn new() -> Self {
				Self(vec![Vec::new()])
			}

			fn last_sequence_mut(&mut self) -> &mut Vec<Ast> {
				self.0.last_mut().unwrap()
			}

			fn last_regexp_mut(&mut self) -> Option<&mut Ast> {
				self.last_sequence_mut().last_mut()
			}

			fn push(&mut self) {
				self.0.push(Vec::new())
			}

			fn union(&mut self, other: Self) {
				self.0.extend(other.0)
			}

			fn into_regexp(self) -> Ast {
				Ast::Union(self.0.into_iter().map(Ast::Sequence).collect())
			}
		}

		struct Stack(Vec<Disjunction>);

		impl Stack {
			fn new() -> Self {
				Self(vec![Disjunction::new()])
			}

			fn last_mut(&mut self) -> &mut Disjunction {
				self.0.last_mut().unwrap()
			}

			fn last_sequence_mut(&mut self) -> &mut Vec<Ast> {
				self.last_mut().last_sequence_mut()
			}

			fn last_regexp_mut(&mut self) -> Option<&mut Ast> {
				self.last_mut().last_regexp_mut()
			}

			fn push(&mut self) {
				self.0.push(Disjunction::new())
			}

			fn pop(&mut self) -> Result<(), Error> {
				let b = self.0.pop().unwrap();
				let a = self
					.0
					.last_mut()
					.ok_or(Error::UnmatchedClosingParenthesis)?;
				a.union(b);
				Ok(())
			}

			fn into_regexp(self) -> Result<Ast, Error> {
				match self.0.len() {
					0 => unreachable!(),
					1 => Ok(self
						.0
						.into_iter()
						.next()
						.unwrap()
						.into_regexp()
						.simplified()),
					_ => Err(Error::MissingClosingParenthesis),
				}
			}
		}

		let mut stack = Stack::new();
		let mut chars = s.chars();

		while let Some(c) = chars.next() {
			match c {
				'(' => stack.push(),
				')' => stack.pop()?,
				'|' => stack.last_mut().push(),
				'[' => {
					let charset = parse_charset(&mut chars)?;
					stack.last_sequence_mut().push(Ast::Set(charset))
				}
				'\\' => {
					let c = parse_escaped_char(&mut chars)?;
					let mut charset = RangeSet::new();
					charset.insert(c);
					stack.last_sequence_mut().push(Ast::Set(charset))
				}
				'?' => stack
					.last_regexp_mut()
					.ok_or(Error::NothingToRepeat)?
					.repeat(0, 1),
				'*' => stack
					.last_regexp_mut()
					.ok_or(Error::NothingToRepeat)?
					.repeat(0, u32::MAX),
				'+' => stack
					.last_regexp_mut()
					.ok_or(Error::NothingToRepeat)?
					.repeat(1, u32::MAX),
				c => {
					let mut charset = RangeSet::new();
					charset.insert(c);
					stack.last_sequence_mut().push(Ast::Set(charset))
				}
			}
		}

		stack.into_regexp()
	}
}

impl FromStr for Ast {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(s)
	}
}

impl<S: AsRef<str>> From<S> for Ast {
	fn from(s: S) -> Self {
		let mut regexp = Self::empty();
		for c in s.as_ref().chars() {
			let mut charset = RangeSet::new();
			charset.insert(c);
			regexp.push(Self::Set(charset))
		}
		regexp
	}
}

fn parse_charset(chars: &mut impl Iterator<Item = char>) -> Result<RangeSet<char>, Error> {
	#[derive(PartialEq, Eq)]
	enum State {
		Start,
		RangeStart,
		RangeDashOrStart,
		RangeEnd,
	}

	let mut state = State::Start;
	let mut negate = false;
	let mut set = RangeSet::new();

	let mut range_start = None;

	loop {
		match chars.next() {
			Some(c) => match c {
				'^' if state == State::Start => {
					negate = true;
					state = State::RangeStart;
				}
				c => match state {
					State::RangeDashOrStart if c == '-' => state = State::RangeEnd,
					State::Start | State::RangeStart | State::RangeDashOrStart if c == ']' => {
						if let Some(start) = range_start.take() {
							set.insert(start);
						}

						if negate {
							set = set.complement();
						}

						break Ok(set);
					}
					State::Start | State::RangeStart | State::RangeDashOrStart => {
						if let Some(start) = range_start.take() {
							set.insert(start);
						}

						let c = match c {
							'\\' => parse_escaped_char(chars)?,
							c => c,
						};

						range_start = Some(c);
						state = State::RangeDashOrStart
					}
					State::RangeEnd => {
						let c = match c {
							'\\' => parse_escaped_char(chars)?,
							c => c,
						};

						set.insert(range_start.take().unwrap()..=c);
						state = State::RangeStart
					}
				},
			},
			None => break Err(Error::IncompleteCharacterSet),
		}
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
		None => Err(Error::IncompleteEscapeSequence),
	}
}
