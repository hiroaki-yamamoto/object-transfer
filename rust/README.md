# Object Transfer Library for Rust

## CI/CD Status

| Service | Status |
|---------|--------|
| Crates.io | [![Crates.io Version Img]][Crates.io] |
| Code Test | [![Test Rust Code Img]][Test Rust Code] |

[Test Rust Code Img]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_rust.yml/badge.svg
[Test Rust Code]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_rust.yml
[Crates.io Version Img]: https://img.shields.io/crates/v/object_transfer
[Crates.io]: https://crates.io/crates/object_transfer


## Description

This library provides a simple and efficient way to transfer objects between
different parts of an application or between different applications through
message brokers like NATS.
It supports serialization and/or deserialization of various data formats, making
it easy to send and/or receive complex data structures.

## Installation

If you have cargo-edit installed, you can add this library to your project
by running:

```sh
cargo add object_transfer
```

Alternatively, you can manually add the following line to your `Cargo.toml` file:

```toml
[dependencies]
object_transfer = "x.x.x"
```

where `x.x.x` is the desired version of the library. For the latst version,
please check [Crates.io](https://crates.io/crates/object_transfer).
