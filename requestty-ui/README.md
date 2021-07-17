# Requestty-ui

A widget based terminal ui rendering library.

This crate provides an abstraction over terminal manipulation in the
form of the `Widget` trait. It also provides some default widgets
available in `widgets`.

While this crate was built for the [`requestty`] crate and other crates
which implement the `Prompt` trait in [`requestty`], it can be used
otherwise as well.

[`requestty`]: https://crates.io/crates/requestty

# Backends

This crate currently supports 2 backends:

- [`crossterm`](https://crates.io/crates/crossterm)
- [`termion`](https://crates.io/crates/termion)

The different backends can be enabled using the features of the same name.
