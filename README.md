# runtime_injector


This library provides a powerful, easy to use inversion-of-control (IoC) container with a focus on ergonomics and configurability.

## Status

| Crate | crates.io | docs |
|---|---|---|
| runtime_injector | [![Current version][base-crate-badge]][base-crates-io] | [![Current documentation][base-doc-badge]][base-docs] |
| runtime_injector_actix | [![Current version][actix-crate-badge]][actix-crates-io] | [![Current documentation][actix-doc-badge]][actix-docs] |

## Getting started

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

These libraries are licensed under your choice of either [MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE).

[base-crate-badge]: https://img.shields.io/crates/v/runtime_injector?style=flat-square
[base-doc-badge]: https://img.shields.io/docsrs/runtime_injector?style=flat-square
[base-crates-io]: https://crates.io/crates/runtime_injector
[base-docs]: https://docs.rs/runtime_injector

[actix-crate-badge]: https://img.shields.io/crates/v/runtime_injector_actix?style=flat-square
[actix-doc-badge]: https://img.shields.io/docsrs/runtime_injector_actix?style=flat-square
[actix-crates-io]: https://crates.io/crates/runtime_injector_actix
[actix-docs]: https://docs.rs/runtime_injector_actix
