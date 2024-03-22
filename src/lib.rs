//! This library provides an implementation of the *POSIX Extended Regular
//! Expression* (ERE) class of regular expressions, and nothing more.
//! It also aims at providing easy tools to inspect finite automata built from
//! regular expressions, or manually.
//!
//! If you are looking for more advanced regular expression, please use the
//! [`regex`] library.
//!
//! [`regex`]: <https://github.com/rust-lang/regex>
pub use ere_automata as automata;
pub use ere_syntax as syntax;
