//! # Inversion of control
//!
//!
//!
//! ## Testing and mocking
//!
//! As an application grows, different components within the application need
//! to be tested in isolation. Additionally, different deployment
//! configurations may require different implementations of a service. What
//! happens when you have code that relies on all these complex configurations?
//! If you aren't careful, different components of your application will start
//! becoming tightly coupled, and your code will become hard to maintain.
//!
//! For example, suppose you want to authenticate users by looking them up in
//! a database, however you need to support a different database for unit
//! testing, integration testing, and production:
//!
//! ```no_run
//! use std::sync::Arc;
//!
//! #[derive(Clone, Debug)]
//! struct User {
//!     scopes: Vec<String>,
//! }
//!
//! // Suppose we need to abstract out our database so we can use different
//! // implementations for different environments. We need our database to be
//! // thread-safe since we're planning to multi-thread our application
//! trait UserDatabase: Send + Sync {
//!     fn get_user(&self, id: u32) -> User {
//!         todo!()
//!     }
//! }
//!
//! // We need to be able to unit test code that depends on a user database
//! #[derive(Default)]
//! struct MockUserDatabase;
//! impl UserDatabase for MockUserDatabase {}
//!
//! // We also need a real implementation that connects to a SQL database.
//! // We'll need a connection string to establish the connection, so we'll
//! // include a field for that.
//! struct SqlUserDatabase(String);
//! impl UserDatabase for SqlUserDatabase {}
//!
//! // We're using a special configuration for integration testing, so we'll
//! // need an implementation for that environment as well. We still need a
//! // connection string for the database, but now we also want to configure it
//! // based on a set of testing parameters. For example, should we force this
//! // database to fail so we can test that?
//! # struct IntegrationTestParameters;
//! struct IntegrationUserDatabase(String, IntegrationTestParameters);
//! impl UserDatabase for IntegrationUserDatabase {}
//!
//! // Now suppose we want to authenticate users. How might we accomplish this
//! // task? Well, we'd look them up in the database and check what permissions
//! // they have!
//! trait UserAuthenticator: Send + Sync {
//!     fn has_access(&self, user_id: u32, scope: &str) -> bool;
//! }
//!
//! // We don't need a database for our mock authenticator since we aren't
//! // looking them up at all.
//! #[derive(Default)]
//! struct MockUserAuthenticator;
//! impl UserAuthenticator for MockUserAuthenticator {
//!     fn has_access(&self, _user_id: u32, _scope: &str) -> bool {
//!         true
//!     }
//! }
//!
//! // We *do* need a database for production though, since we want to look
//! // them up in our database to check their permissions.
//! struct DatabaseUserAuthenticator<DB: UserDatabase>(Arc<DB>);
//! impl<DB: UserDatabase> UserAuthenticator for DatabaseUserAuthenticator<DB> {
//!     fn has_access(&self, user_id: u32, scope: &str) -> bool {
//!         let user = self.0.get_user(user_id);
//!         user.scopes.iter().any(|s| s.as_str() == scope)
//!     }
//! }
//!
//! fn main() {
//!     // Now we need to configure which implementations to use for our
//!     // environment! We can use feature flags for this, but it's ugly and
//!     // gets unmaintainable fast as our number of services grow.
//!     // Additionally, if we want to construct any services later during our
//!     // program's execution, we need a whole mess of cfg attributes again.
//!     // This turns into a mess of helper functions for constructing our
//!     // dependencies.
//!
//!     // Let's make a helper for creating our user database. The actual
//!     // implementation depends on if we're in our integration testing
//!     // environment, so we'll return `impl UserDatabase` so we don't need to
//!     // worry about specifying the concrete type
//!     fn make_database(connection_string: String) -> impl UserDatabase {
//!         #[cfg(feature = "integration")]
//!         {
//!             IntegrationUserDatabase(
//!                 connection_string,
//!                 // How do we get the integration test parameters? For now,
//!                 // let's globally configure it somewhere and retrieve them
//!                 // like that. This way, we can make sure our helper is easy
//!                 // to use.
//!                 get_integration_test_parameters(),
//!             )
//!         }
//!         #[cfg(not(feature = "integration"))]
//!         {
//!             SqlUserDatabase(connection_string)
//!         }
//!     }
//!
//!     // Now we need a way to get our integration test parameters. Since we
//!     // want to configure this at runtime, this can quickly become
//!     // complicated, possibly involving global state with static variables
//!     // so that we can call this whenever we need to. Since it is unsafe to
//!     // use mutable static variables, we'll probably want to store it as
//!     // `Arc<Mutex<Option<IntegrationTestParameters>>>` and have a way of
//!     // setting the value before our test. Let's leave that out for now...
//!     fn get_integration_test_parameters() -> IntegrationTestParameters {
//!         todo!()
//!     }
//!
//!     // Since we sometimes want to mock our database, let's make an
//!     // additional helper for this. Note that we can't pass a bool into our
//!     // make_database function to configure this because we'd end up having
//!     // two possible return types from our function, causing a compile error
//!     fn make_database_mock() -> impl UserDatabase {
//!         MockUserDatabase
//!     }
//!
//!     // Let's create a helper for creating our user authenticator. Since we
//!     // don't have a special implementation for integration testing, this
//!     // is much simpler than our user database helper
//!     fn make_authenticator(
//!         database: Arc<impl UserDatabase>,
//!     ) -> impl UserAuthenticator {
//!         DatabaseUserAuthenticator(database)
//!     }
//!
//!     // Our mock authenticator has a different set of dependencies! Let's
//!     // make another helper for creating our mock authenticator since we
//!     // don't need a database to construct it.
//!     fn make_authenticator_mock() -> impl UserAuthenticator {
//!         MockUserAuthenticator
//!     }
//! }
//! ```
//!
//! That's quite a bit of code to setup even just a simple application with
//! multiple target environments! This doesn't even include our business logic.
//!
//! ## Simplifying our code with runtime_injector
//!
//! As our application grows, so does the number of helper functions we need to
//! create to handle all the different implementations of our services. This
//! quickly becomes ugly and unmaintainable. What happens if we let something
//! else create our services for us instead? Let's let an
//! [`Injector`](crate::Injector) manage our services for us to help us
//! simplify our code and make it more maintainable.
//!
//! ```
//! use runtime_injector::{
//!     constant, define_module, interface, Arg, Injector, IntoSingleton,
//!     IntoTransient, Service, Svc, WithArg,
//! };
//!
//! #[derive(Clone, Debug)]
//! struct User {
//!     scopes: Vec<String>,
//! }
//!
//! // We still want our services to be thread-safe. `Service` is a trait that
//! // is automatically implemented for all `Send + Sync + 'static` types, so
//! // we can use it here instead. Additionally, if we decide that we no longer
//! // need to multi-thread this later, we can switch to the "rc" feature to
//! // use `Rc` for our service pointers, and this trait will automatically be
//! // implemented for all `'static` types instead, regardless of thread safety
//! trait UserDatabase: Service {
//!     fn get_user(&self, id: u32) -> User {
//!         todo!()
//!     }
//! }
//!
//! #[derive(Default)]
//! struct MockUserDatabase;
//! impl UserDatabase for MockUserDatabase {}
//!
//! // Let's pass our connection string as an argument so we don't need to
//! // hardcode it.
//! struct SqlUserDatabase(Arg<String>);
//! impl UserDatabase for SqlUserDatabase {}
//!
//! # #[derive(Default)]
//! # struct IntegrationTestParameters;
//! // We'll also inject our connection string and test parameters here
//! struct IntegrationUserDatabase(Arg<String>, Svc<IntegrationTestParameters>);
//! impl UserDatabase for IntegrationUserDatabase {}
//!
//! trait UserAuthenticator: Service {
//!     fn has_access(&self, user_id: u32, scope: &str) -> bool;
//! }
//!
//! #[derive(Default)]
//! struct MockUserAuthenticator;
//! impl UserAuthenticator for MockUserAuthenticator {
//!     fn has_access(&self, _user_id: u32, _scope: &str) -> bool {
//!         true
//!     }
//! }
//!
//! // We're switching to dynamic dispatch here which is marginally slower than
//! // static dispatch, but we're going to lose most of our performance to
//! // network I/O anyway when making database requests.
//! struct DatabaseUserAuthenticator(Svc<dyn UserDatabase>);
//! impl UserAuthenticator for DatabaseUserAuthenticator {
//!     fn has_access(&self, user_id: u32, scope: &str) -> bool {
//!         let user = self.0.get_user(user_id);
//!         user.scopes.iter().any(|s| s.as_str() == scope)
//!     }
//! }
//!
//! // Now we need to declare our interfaces. These are the traits we want to
//! // have abstracted away.
//! interface!(UserDatabase);
//! interface!(UserAuthenticator);
//!
//! fn main() {
//!     // We can easily determine which implementations we will use in one
//!     // place by creating a module. If we add more implementations later, we
//!     // only need to change a few lines of code in one place rather than
//!     // adding #[cfg] attributes all over our code
//!     let connection_string = "our_secret_connection_string".to_string();
//!     let module = define_module! {
//!         interfaces = {
//!             dyn UserAuthenticator = [
//!                 DatabaseUserAuthenticator.singleton()
//!             ],
//!             #[cfg(feature = "integration")]
//!             dyn UserDatabase = [
//!                 IntegrationUserDatabase
//!                     .singleton()
//!                     .with_arg(connection_string)
//!                     .with_arg(IntegrationTestParameters::default())
//!             ],
//!             #[cfg(not(feature = "integration"))]
//!             dyn UserDatabase = [
//!                 SqlUserDatabase.singleton().with_arg(connection_string)
//!             ],
//!         },
//!     };
//!
//!     // Now we'll start to create our container. Using a builder, we can
//!     // easily tell our injector what services it should be able to provide.
//!     // We'll start by adding our module to it
//!     let mut builder = Injector::builder();
//!     builder.add_module(module);
//!
//!     // Now we're ready to start creating our services! We have one single
//!     // way of creating each of our services, regardless of what the actual
//!     // implementation of that service is. Let's create our container now
//!     let injector = builder.build();
//!
//!     // We can get any service we want with `injector.get()`. Here, we're
//!     // relying on our container to pass in the connection string and test
//!     // parameters (if we're doing an integration test) without needing any
//!     // complicated logic to construct those types
//!     let database: Svc<dyn UserDatabase> = injector.get().unwrap();
//!
//!     // We've already created our database, and we don't want to create it
//!     // again. Since we've declared our database as a singleton service, our
//!     // container will reuse the same instance when we get our authenticator
//!     let auth: Svc<dyn UserAuthenticator> = injector.get().unwrap();
//!
//!     // If we want to mock out any of our services, all we need to do is
//!     // create a module which provides the mock implementation instead
//!     let _module = define_module! {
//!         interfaces = {
//!             dyn UserAuthenticator = [
//!                 DatabaseUserAuthenticator.singleton()
//!             ],
//!             dyn UserDatabase = [MockUserDatabase::default.singleton()],
//!         },
//!     };
//! }
//! ```
//!
//! We still have all our different implementations of `UserDatabase` and
//! `UserAuthenticator`, but now it's super easy to get the correct
//! implementations of each of those services when we need to! We don't need
//! any complicated helper functions, and we certainly don't need to litter our
//! business logic with `#[cfg]` to be able to use it with unit and integration
//! testing. Instead, rather than relying on our helpers to create the right
//! implementations for us, we're just asking for an implementation and letting
//! our container handle the rest.
//!
//! Something else that you might notice is that we are able to rely on our
//! container to create a single instance of our user database and provide that
//! single instance whenever we need it. Not only can we rely on our injector
//! to call our constructors for us, but we can also rely on it to manage the
//! lifetimes of our services. This would normally be very difficult without a
//! container to do it for you. For example, suppose you want to create a
//! single instance of your user database throughout the entire lifetime of
//! your application since you don't want to open unnecessary connections to
//! your database. Rather than relying on a static variable to hold that
//! instance, we can instead rely on our container to create and provide that
//! instance when we need it. If we wanted to provide a new instance each time,
//! we could configure our container to do that instead! Similarly, if we
//! wanted to create a single instance of our service for every HTTP request
//! that we get, that's possible as well by creating a custom provider. We have
//! complete control over our services, yet we don't have any of the extra
//! complexities that normally comes with that level of control.
