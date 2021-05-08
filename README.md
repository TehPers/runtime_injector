# runtime_injector

[![Current version][crate-badge]][crates-io]
[![Current documentation][doc-badge]][docs]

This library provides a powerful, easy to use inversion-of-control (IoC) container with a focus on ergonomics and configurability.

## Getting started

For using the library, check out the [docs].

For local development of runtime_injector, clone the repository, then build the project with cargo:

```bash
git clone https://github.com/TehPers/runtime_injector
cd runtime_injector
cargo build
```

If you want to build the project using the "rc" feature instead, disable default features, and enable the "rc" feature:

```bash
cargo build -p runtime_injector --no-default-features --features rc
```

Note that not all crates support the "rc" feature, so you will need to specify which crate you want to build.

## License

This library is licensed under your choice of either [MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE).

[crate-badge]: https://img.shields.io/crates/v/runtime_injector?style=flat-square
[doc-badge]: https://img.shields.io/docsrs/runtime_injector?style=flat-square
[crates-io]: https://crates.io/crates/runtime_injector
[docs]: https://docs.rs/runtime_injector
