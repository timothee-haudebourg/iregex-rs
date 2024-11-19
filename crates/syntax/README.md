# POSIX Extended Regular Expression (iregex) parser for Rust

<!-- cargo-rdme start -->

This library provides a parser for POSIX Extended Regular Expressions (iregex).
Once parsed into an abstract syntax tree ([`Ast`]), regular expressions can
then be compiled into a finite automaton running on Unicode scalar values
([`char`] type) using the [`iregex-automata`] library.

[`iregex-automata`]: <https://crates.io/crates/iregex-automata>

<!-- cargo-rdme end -->

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
