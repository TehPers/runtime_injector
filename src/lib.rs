//! Runtime dependency injection.
//!
//! By default, services provided by the `Injector` are not thread-safe. This
//! is because `Rc<T>` is used to hold instances of the services, which is not
//! a thread-safe pointer type. This can be changed by disabling default
//! features and enabling the "arc" feature:
//!
//! ```text
//! runtime_injector = {
//!     version = "*",
//!     default_features = false,
//!     features = ["arc"]
//! }
//! ```
//!
//! # Runtime dependency injection (rather than compile-time)
//!
//! Runtime dependency injection allows for custom configuration of services
//! during runtime rather than needing to determine what services are used at
//! compile time. This means you can read a config when your application
//! starts, determine what implementations you want to use for your interfaces,
//! and assign those at runtime. This is also slower than compile-time
//! dependency injection, so if pointer indirection, dynamic dispatch, or heap
//! allocations are a concern, then a compile-time dependency injection library
//! might be preferred instead.
//!
//! # Interfaces
//!
//! Proper inversion of control requires that each service requests its
//! dependencies without actually caring how those dependencies are
//! implemented. For instance, suppose you are working with a database. A
//! service which depends on interacting with that database may request a
//! dependency that can interact with that database without needing to know the
//! concrete type being used. This is done using dynamic dispatch to allow the
//! concrete type to be determined at runtime (rather than using generics to
//! determine the implementations at compile time).
//!
//! # Service lifetimes
//!
//! Lifetimes of services created by the `Injector` are controlled by the
//! provider used to construct those lifetimes. Currently, there are three
//! built-in service provider types:
//!
//! - Singleton: A service is created only the first time it is requested and
//!   that single instance is reused for each future request.
//! - Transient: A service is created each time it is requested.
//! - Constant: Used for services that are not created using a factory function
//!   and instead can have their instance provided to the container directly.
//!   This behaves similar to singleton in that the same instance is provided
//!   each time the service is requested.
//!
//! Custom service providers can also be created by implementing the
//! `TypedProvider` trait.
//!
//! # Example
//!
//! ```
//! use runtime_injector::{interface, Injector, Svc, IntoSingleton};
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
//! interface!(DataService = [ SqlDataService, MockDataService ]);
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
//!     builder.provide(UserService::new.singleton());
//!     builder.provide(SqlDataService::default.singleton());
//!     builder.provide(MockDataService::default.singleton());
//!
//!     // Note that we can register closures as providers as well
//!     builder.provide((|_: Svc<dyn DataService>| "Hello, world!").singleton());
//!     
//!     // Let's choose to use the MockDataService as our data service
//!     builder.implement::<dyn DataService, MockDataService>();
//!     
//!     // Now that we've registered all our providers and implementations, we
//!     // can start relying on our container to create our services for us!
//!     let mut injector = builder.build();
//!     let user_service: Svc<UserService> = injector.get()?;
//!     let _user = user_service.get_user("john");
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value
)]

#[cfg(not(any(feature = "arc", feature = "rc")))]
compile_error!(
    "Either the 'arc' or 'rc' feature must be enabled (but not both)."
);

#[cfg(all(feature = "arc", feature = "rc"))]
compile_error!(
    "The 'arc' and 'rc' features are mutually exclusive and cannot be enabled together."
);

mod builder;
mod injector;
mod request;
mod services;

pub use builder::*;
pub use injector::*;
pub use request::*;
pub use services::*;

#[cfg(test)]
mod tests;
