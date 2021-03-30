# runtime_injector

[![Current version][crate-badge]][crates-io]
[![Current documentation][doc-badge]][docs]

This library provides an easy to use dependency injection container with a focus on ergonomics and configurability at the cost of runtime performance. For a more performance-oriented container, look for a compile-time dependency injection library.

Sample code is available on the [docs].

## Getting started

For using the library, check out the [docs].

For local development of runtime_injector, clone the repository, then build the project with cargo:

```bash
git clone https://github.com/TehPers/runtime_injector
cd runtime_injector
cargo build
```

If you want to build the project using the "arc" feature instead, disable default features, and enable the "arc" feature:

```bash
cargo build --no-default-features --features arc
```

## Minimum supported Rust version

As the library is still in development, the only supported Rust version is the most recent version of Rust. The library may work on older versions, but there is no guarantee.

## License

This library is licensed under your choice of either [MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE).

[crate-badge]: https://img.shields.io/crates/v/runtime_injector?style=flat-square
[doc-badge]: https://img.shields.io/docsrs/runtime_injector?style=flat-square
[crates-io]: https://crates.io/crates/runtime_injector
[docs]: https://docs.rs/runtime_injector
