# POSIX Extended Regular Expressions (ERE) for Rust

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/timothee-haudebourg/ere-rs/ci.yml?style=flat-square&logo=github)](https://github.com/timothee-haudebourg/ere-rs/actions)
[![Crate informations](https://img.shields.io/crates/v/ere.svg?style=flat-square)](https://crates.io/crates/ere)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/ere?style=flat-square)](https://crates.io/crates/ere)
[![License](https://img.shields.io/crates/l/ere.svg?style=flat-square)](https://github.com/timothee-haudebourg/ere-rs#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/ere)

<!-- cargo-rdme start -->

This library provides an implementation of the *POSIX Extended Regular
Expression* (ERE) class of regular expressions, and nothing more.
It also aims at providing easy tools to inspect finite automata built from
regular expressions, or manually.

If you are looking for more advanced regular expression, please use the
[`regex`] library.

[`regex`]: <https://github.com/rust-lang/regex>

<!-- cargo-rdme end -->

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
