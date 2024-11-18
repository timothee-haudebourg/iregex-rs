//! IRegEx (or IRE) is an intermediate representation for Regular Expressions
//! with a well-defined semantics providing a foundation for common Regular
//! Expression dialects. It also aims at providing easy tools to inspect finite
//! automata built from regular expressions, or manually.
//!
//! If you are instead looking for a ready-to-use and feature-rich regular
//! expression library, please use the [`regex`] library.
//!
//! [`regex`]: <https://github.com/rust-lang/regex>
pub use iregex_automata as automata;

mod ir;
pub use ir::*;

mod compiled;
pub use compiled::*;
