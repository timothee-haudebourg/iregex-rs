//! IRegEx (or IRE) is an intermediate representation for Regular Expressions
//! with a well-defined semantics providing a foundation for common Regular
//! Expression dialects. It also aims at providing easy tools to inspect finite
//! automata built from regular expressions, or manually.
//!
//! If you are instead looking for a ready-to-use and feature-reach regular
//! expression library, please use the [`regex`] library.
//!
//! [`regex`]: <https://github.com/rust-lang/regex>
use automata::Class;
pub use iregex_automata as automata;

mod ir;
pub use ir::*;

mod compiled;
pub use compiled::*;

pub trait Boundary<T> {
	type Class: Class<T>;

	fn apply(&self, class: &Self::Class) -> Option<Self::Class>;
}

impl<T> Boundary<T> for () {
	type Class = ();

	fn apply(&self, _class: &Self::Class) -> Option<Self::Class> {
		Some(())
	}
}
