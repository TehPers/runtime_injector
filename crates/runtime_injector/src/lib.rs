//! # Runtime dependency injection.
//!
//! By default, services provided by the [`Injector`] use thread-safe pointers.
//! This is because [`Arc<T>`](std::sync::Arc) is used to hold instances of the
//! services. This can be changed to [`Rc<T>`](std::rc::Rc) by disabling
//! default features and enabling the "rc" feature:
//!
//! ```text
//! [dependencies.runtime_injector]
//! version = "*" # Replace with the version you want to use
//! default-features = false
//! features = ["rc"]
//! ```
//!
//! ## Getting started
//!
//! If you are unfamiliar with dependency injection, then you may want to check
//! about how a container can help
//! [simplify your application][ioc]. Otherwise,
//! check out the [getting started guide][getting-started]
//!
//! [ioc]: crate::docs::inversion_of_control
//! [getting-started]: crate::docs::getting_started
//!
//! ## Dependency injection at runtime (rather than compile-time)
//!
//! Runtime dependency injection allows for advanced configuration of services
//! during runtime rather than needing to decide what services your application
//! will use at compile time. This means you can read a config when your
//! application starts, decide what implementations you want to use for your
//! interfaces, and assign those at runtime. This is also slightly slower than
//! compile-time dependency injection, so if pointer indirection, dynamic
//! dispatch, or heap allocations are a concern, then a compile-time dependency
//! injection library might work better instead. However, in asynchronous,
//! I/O-based applications like a web server, the additional overhead is
//! probably insignificant compared to the additional flexibility you get with
//! runtime_injector.
//!
//! ## Interfaces
//!
//! Using interfaces allows you to write your services without worrying about
//! how its dependencies are implemented. You can think of them like generic
//! type parameters for your service, except rather than needing to add a new
//! type parameter, you use a service pointer to the interface for your
//! dependency. This makes your code easier to read and faster to write, and
//! keeps your services decoupled from their dependencies and dependents.
//!
//! Interfaces are implemented as trait objects in runtime_injector. For
//! instance, you may define a trait `UserDatabase` and implement it for
//! several different types. [`Svc<dyn UserDatabase>`](crate::Svc<T>) is a
//! reference-counted service pointer to an implementation of your trait.
//! Similarly, `dyn UserDatabase` is your interface. You can read more about
//! how interfaces work and how they're created in the
//! [type-level docs](crate::Interface).
//!
//! ## Service lifetimes
//!
//! Lifetimes of services created by the [`Injector`] are controlled by the
//! [`Provider`] used to construct those lifetimes. Currently, there are three
//! built-in service provider types:
//!
//! - **[Transient](crate::TransientProvider):** A service is created each time
//!   it is requested. This will never return the same instance of a service
//!   more than once.
//! - **[Singleton](crate::SingletonProvider):** A service is created only the
//!   first time it is requested, then that single instance is reused for each
//!   future request.
//! - **[Constant](crate::ConstantProvider):** Used for services that are not
//!   created using a service factory and instead can have their instance
//!   provided to the container directly. This behaves similar to singleton in
//!   that the same instance is provided each time the service is requested.
//!
//! Custom service providers can also be created by implementing either the
//! [`TypedProvider`] or [`Provider`] trait.
//!
//! ## Fallible service factories
//!
//! Not all types can always be successfully created. Sometimes, creating an
//! instance of a service might fail. Rather than panicking on error, it's
//! possible to instead return a [`Result<T, E>`] from your constructors and
//! inject the result as a [`Svc<T>`]. Read more in the
//! [docs for `IntoFallible`](crate::IntoFallible).
//!
//! ## Owned service pointers
//!
//! In general, providers need to be able to provide their services via
//! reference-counted service pointers, or [`Svc<T>`]. The issue with this is
//! that you cannot get mutable or owned access to the contents of those
//! pointers since they are shared pointers. As a result, you may need to clone
//! some dependencies in your constructors if you want to be able to own them.
//!
//! If your dependency is a transient service, then it might make more sense
//! to inject it as a [`Box<T>`] than clone it from a reference-counted service
//! pointer. In these cases, you can request a [`Box<T>`] directly from the
//! injector and avoid needing to clone your dependency entirely!
//!
//! ## Custom target-specific arguments
//!
//! Sometimes it's useful to be able to pass a specific value into your
//! services. For example, if you're writing a database service and you need a
//! connection string, you could define a new `ConnectionString` struct as a
//! newtype for [`String`], but that would be a bit excessive for passing in a
//! single value. If you had several arguments you needed to pass in this way,
//! then that would mean you would need a new type for each one.
//!
//! Rather than creating a bunch of newtypes, you can use [`Arg<T>`] to pass in
//! pre-defined values directly to your services. For example, you can use
//! `Arg<String>` to pass in your connection string, plus you can use
//! `Arg<usize>` to set the max size of your connection pool, and another
//! `Arg<String>` in your logging service to set your logging format without
//! needing to worry about accidentally using your connection string as your
//! logging format!
//!
//! ## Example
//!
//! ```
//! use runtime_injector::{
//!     define_module, Module, interface, Injector, Svc, IntoSingleton,
//!     TypedProvider, IntoTransient, constant
//! };
//! use std::error::Error;
//!
//! // Some type that represents a user
//! struct User;
//!
//! // This is our interface. In practice, multiple structs can implement this
//! // trait, and we don't care what the concrete type is most of the time in
//! // our other services as long as it implements this trait. Because of this,
//! // we're going to use dynamic dispatch later so that we can determine the
//! // concrete type at runtime (vs. generics, which are determined instead at
//! // compile time).
//! //
//! // The `Send` and `Sync` supertrait requirements are only necessary when
//! // compiling with the "arc" feature to allow for service pointer
//! // downcasting.
//! trait DataService: Send + Sync {
//!     fn get_user(&self, user_id: &str) -> Option<User>;
//! }
//!
//! // We can use a data service which connects to a SQL database.
//! #[derive(Default)]
//! struct SqlDataService;
//! impl DataService for SqlDataService {
//!     fn get_user(&self, _user_id: &str) -> Option<User> { todo!() }
//! }
//!
//! // ... Or we can mock out the data service entirely!
//! #[derive(Default)]
//! struct MockDataService;
//! impl DataService for MockDataService {
//!     fn get_user(&self, _user_id: &str) -> Option<User> { Some(User) }
//! }
//!
//! // Specify which types implement the DataService interface. This does not
//! // determine the actual implementation used. It only registers the types as
//! // possible implementations of the DataService interface.
//! interface!(DataService = [SqlDataService, MockDataService]);
//!
//! // Here's another service our application uses. This service depends on our
//! // data service, however it doesn't care how that service is actually
//! // implemented as long as it works. Because of that, we're using dynamic
//! // dispatch to allow the implementation to be determined at runtime.
//! struct UserService {
//!     data_service: Svc<dyn DataService>,
//! }
//!
//! impl UserService {
//!     // This is just a normal constructor. The only requirement is that each
//!     // parameter is a valid injectable dependency.
//!     pub fn new(data_service: Svc<dyn DataService>) -> Self {
//!         UserService { data_service }
//!     }
//!
//!     pub fn get_user(&self, user_id: &str) -> Option<User> {
//!         // UserService doesn't care how the user is actually retrieved
//!         self.data_service.get_user(user_id)
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     // This is where we register our services. Each call to `.provide` adds
//!     // a new service provider to our container, however nothing is actually
//!     // created until it is requested. This means we can add providers for
//!     // types we aren't actually going to use without worrying about
//!     // constructing instances of those types that we aren't actually using.
//!     let mut builder = Injector::builder();
//!
//!     // We can manually add providers to our builder
//!     builder.provide(UserService::new.singleton());
//!
//!     struct Foo(Svc<dyn DataService>);
//!     
//!     // Alternatively, modules can be used to group providers and
//!     // configurations together, and can be defined via the
//!     // define_module! macro
//!     let module = define_module! {
//!         services = [
//!             // Simple tuple structs can be registered as services directly without
//!             // defining any additional constructors
//!             Foo.singleton(),
//!             
//!             // Note that we can register closures as providers as well
//!             (|_: Svc<dyn DataService>| "Hello, world!").singleton(),
//!             (|_: Option<Svc<i32>>| 120.9).transient(),
//!
//!             // Since we know our dependency is transient, we can request an
//!             // owned pointer to it rather than a reference-counted pointer
//!             (|value: Box<f32>| format!("{}", value)).transient(),
//!
//!             // We can also provide constant values directly to our services
//!             constant(8usize),
//!         ],
//!         interfaces = {
//!             // Let's choose to use the MockDataService as our data service
//!             dyn DataService = [MockDataService::default.singleton()],
//!         },
//!     };
//!
//!     // You can easily add a module to your builder
//!     builder.add_module(module);
//!
//!     // Now that we've registered all our providers and implementations, we
//!     // can start relying on our container to create our services for us!
//!     let injector = builder.build();
//!     let user_service: Svc<UserService> = injector.get()?;
//!     let _user = user_service.get_user("john");
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::needless_doctest_main
)]

#[cfg(not(any(feature = "arc", feature = "rc")))]
compile_error!(
    "Either the 'arc' or 'rc' feature must be enabled (but not both)."
);

#[cfg(all(feature = "arc", feature = "rc"))]
compile_error!(
    "The 'arc' and 'rc' features are mutually exclusive and cannot be enabled together."
);

mod any;
mod builder;
mod injector;
mod iter;
mod module;
mod requests;
mod services;

pub use any::*;
pub use builder::*;
pub use injector::*;
pub use iter::*;
pub use module::*;
pub use requests::*;
pub use services::*;

pub mod docs;

#[cfg(test)]
mod tests;
