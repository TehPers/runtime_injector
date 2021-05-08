# runtime_injector

[![Current version][crate-badge]][crates-io]
[![Current documentation][doc-badge]][docs]

This library provides a powerful, easy to use inversion-of-control (IoC) container with a focus on ergonomics and configurability.

## Getting started

First, configure your injector:

```rust
let module = define_module! {
    services = [MyService::new.transient()],
    interfaces = {
        dyn MyInterface = [MyInterfaceImpl::new.singleton()],
    },
};

let mut builder = Injector::builder();
builder.add_module(module);
builder.provide(constant(MyConfig));
```

Next, create your injector and request your services from it:

```rust
let injector = builder.build();
let my_service: Svc<MyService> = injector.get().unwrap();
let my_interface_impl: Svc<dyn MyInterface> = injector.get().unwrap();

// Since `MyService` is transient, we can also request an owned instance of it
let my_service: Box<MyService> = injector.get().unwrap();
```

## Minimum supported Rust version

As the library is still in development, the only supported Rust version is the most recent version of stable Rust. The library may work on older versions, but there is no guarantee.

## License

This library is licensed under your choice of either [MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE).

[crate-badge]: https://img.shields.io/crates/v/runtime_injector?style=flat-square
[doc-badge]: https://img.shields.io/docsrs/runtime_injector?style=flat-square
[crates-io]: https://crates.io/crates/runtime_injector
[docs]: https://docs.rs/runtime_injector
