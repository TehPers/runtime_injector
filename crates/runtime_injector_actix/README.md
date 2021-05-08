# runtime_injector_actix

[![Current version][crate-badge]][crates-io]
[![Current documentation][doc-badge]][docs]

This library provides types to help with injecting services into actix-web
applications.

## Getting started

Create your injector, then add it as app data to your application:

```rust
#[actix::main]
async fn main() -> std::io::Result<()> {
    // Define a module so the container knows what to inject
    let module = define_module! {
        #[cfg(not(test))]
        services = [
            UserAuthenticator::new.transient(),
        ],
        #[cfg(not(debug_assertions))]
        interfaces = {
            dyn UserDatabase = [SqlUserDatabase::new.singleton()],
        },
        #[cfg(debug_assertions)]
        interfaces = {
            dyn UserDatabase = [JsonUserDatabase::new.singleton()],
        },
    };

    // Configure and build the container
    let mut builder = Injector::builder();
    builder.add_module(module);
    let injector = builder.build();

    // Now add it as app data to the application
    HttpServer::new(|| App::new().app_data(injector.clone()).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

Inject dependencies with `Injected<R>` in your request handlers:

```rust
#[get("/")]
async fn index(
    user_db: Injected<Svc<dyn UserDatabase>>,
    user_auth: Injected<Box<UserAuthenticator>>,
) -> impl Responder {
    todo!()
}
```

## Minimum supported Rust version

As the library is still in development, the only supported Rust version is the most recent version of stable Rust. The library may work on older versions, but there is no guarantee.

## License

This library is licensed under your choice of either [MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE).

[crate-badge]: https://img.shields.io/crates/v/runtime_injector_actix?style=flat-square
[doc-badge]: https://img.shields.io/docsrs/runtime_injector_actix?style=flat-square
[crates-io]: https://crates.io/crates/runtime_injector_actix
[docs]: https://docs.rs/runtime_injector_actix
