use std::{fmt, ops::Bound};

use btree_range_map::{AnyRange, Directed, RangeSet};

use crate::NFA;

pub trait DotDisplay {
	fn dot(&self) -> DotDisplayed<Self> {
		DotDisplayed(self)
	}

	fn dot_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl DotDisplay for u32 {
	fn dot_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "q{self}")
	}
}

pub struct DotDisplayed<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized + DotDisplay> fmt::Display for DotDisplayed<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.dot_fmt(f)
	}
}

pub trait DotLabelDisplay {
	fn dot_label(&self) -> DotLabelDisplayed<Self> {
		DotLabelDisplayed(self)
	}

	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl DotLabelDisplay for char {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.is_ascii_graphic() {
			fmt::Display::fmt(self, f)
		} else {
			write!(f, "\\\\u{{{:x}}}", *self as u32)
		}
	}
}

impl DotLabelDisplay for u8 {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self:x}")
	}
}

impl DotLabelDisplay for u32 {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "q{self}")
	}
}

impl<'a, T: DotLabelDisplay> DotLabelDisplay for &'a T {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		T::dot_label_fmt(*self, f)
	}
}

impl<'a, T: DotLabelDisplay> DotLabelDisplay for Directed<&'a Bound<T>> {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Directed::Start(Bound::Unbounded) => Ok(()),
			Directed::Start(Bound::Included(t)) => t.dot_label_fmt(f),
			Directed::Start(Bound::Excluded(t)) => write!(f, "{}.", t.dot_label()),
			Directed::End(Bound::Unbounded) => Ok(()),
			Directed::End(Bound::Included(t)) => write!(f, "={}", t.dot_label()),
			Directed::End(Bound::Excluded(t)) => t.dot_label_fmt(f),
		}
	}
}

impl<T: DotLabelDisplay> DotLabelDisplay for AnyRange<T> {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		Directed::Start(&self.start).dot_label_fmt(f)?;
		f.write_str("..")?;
		Directed::End(&self.end).dot_label_fmt(f)
	}
}

impl<T: DotLabelDisplay> DotLabelDisplay for RangeSet<T> {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, range) in self.iter().enumerate() {
			if i > 0 {
				f.write_str(",")?;
			}

			range.dot_label_fmt(f)?;
		}

		Ok(())
	}
}

impl<T: DotLabelDisplay> DotLabelDisplay for Option<T> {
	fn dot_label_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Some(t) => t.dot_label_fmt(f),
			None => Ok(()),
		}
	}
}

pub struct DotLabelDisplayed<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized + DotLabelDisplay> fmt::Display for DotLabelDisplayed<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.dot_label_fmt(f)
	}
}

impl<T: DotLabelDisplay, Q: DotDisplay + DotLabelDisplay> DotDisplay for NFA<Q, T> {
	fn dot_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "digraph {{")?;

		for q in self.states() {
			writeln!(f, "\t{} [label = \"{}\"]", q.dot(), q.dot_label())?;
		}

		for (q, transitions) in self.transitions() {
			for (label, targets) in transitions {
				for r in targets {
					writeln!(
						f,
						"\t{} -> {} [label = \"{}\"]",
						q.dot(),
						r.dot(),
						label.dot_label()
					)?;
				}
			}
		}

		write!(f, "}}")
	}
}
