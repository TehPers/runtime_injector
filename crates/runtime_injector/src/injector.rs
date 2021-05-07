use crate::{
    InjectResult, InjectorBuilder, Interface, Provider, Request, RequestInfo,
    ServiceInfo, Services,
};
use std::collections::HashMap;

pub(crate) type ProviderMap =
    HashMap<ServiceInfo, Option<Vec<Box<dyn Provider>>>>;

pub(crate) trait MapContainerEx<T> {
    fn new(value: T) -> Self;
    fn with_inner<R, F: FnOnce(&T) -> R>(&self, f: F) -> R;
    fn with_inner_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R;
}

#[cfg(feature = "rc")]
mod types {
    use super::MapContainerEx;
    use std::{cell::RefCell, rc::Rc};

    pub type MapContainer<T> = Rc<RefCell<T>>;

    impl<T> MapContainerEx<T> for MapContainer<T> {
        fn new(value: T) -> Self {
            Rc::new(RefCell::new(value))
        }

        fn with_inner<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
            f(&*self.borrow())
        }

        fn with_inner_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
            f(&mut *self.borrow_mut())
        }
    }
}

#[cfg(feature = "arc")]
mod types {
    use super::MapContainerEx;
    use std::sync::{Arc, Mutex};

    pub type MapContainer<T> = Arc<Mutex<T>>;

    impl<T> MapContainerEx<T> for MapContainer<T> {
        fn new(value: T) -> Self {
            Arc::new(Mutex::new(value))
        }

        fn with_inner<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
            f(&*self.lock().unwrap())
        }

        fn with_inner_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
            f(&mut *self.lock().unwrap())
        }
    }
}

#[allow(clippy::wildcard_imports)]
pub(crate) use types::*;

/// A runtime dependency injection container. This holds all the bindings
/// between service types and their providers, as well as all the mappings from
/// interfaces to their implementations (if they differ).
///
/// # Injecting the injector
///
/// Cloning the injector does not clone the providers inside of it. Instead,
/// both injectors will use the same providers, meaning that an injector can be
/// passed to a service as a dependency. The injector can be requested as
/// itself without using a service pointer. It does not need to be registered
/// as a dependency in the builder beforehand.
///
/// Note that requesting the injector inside of your services is generally bad
/// practice, and is known as the service locator antipattern. This is mostly
/// useful for service factories where you can create instances of your
/// services on demand.
///
/// ```
/// use runtime_injector::{
///     constant, InjectResult, Injector, IntoSingleton, IntoTransient, Svc,
/// };
/// use std::sync::Mutex;
///
/// struct FloatFactory(Injector);
///
/// impl FloatFactory {
///     pub fn new(injector: Injector) -> Self {
///         FloatFactory(injector)
///     }
///
///     pub fn get(&self) -> InjectResult<f32> {
///         let int: Svc<i32> = self.0.get()?;
///         Ok(*int as f32)
///     }
/// }
///
/// fn count(counter: Svc<Mutex<i32>>) -> i32 {
///     let mut counter = counter.lock().unwrap();
///     *counter += 1;
///     *counter
/// }
///
/// let mut builder = Injector::builder();
/// builder.provide(constant(Mutex::new(0i32)));
/// builder.provide(count.transient());
/// builder.provide(FloatFactory::new.singleton());
///
/// let injector = builder.build();
/// let float_factory: Svc<FloatFactory> = injector.get().unwrap();
/// let value1 = float_factory.get().unwrap();
/// let value2 = float_factory.get().unwrap();
///
/// assert_eq!(1.0, value1);
/// assert_eq!(2.0, value2);
/// ```
#[derive(Clone)]
pub struct Injector {
    provider_map: MapContainer<ProviderMap>,
}

impl Injector {
    /// Creates a builder for an injector. This is the preferred way of
    /// creating an injector.
    #[must_use]
    pub fn builder() -> InjectorBuilder {
        InjectorBuilder::default()
    }

    /// Creates a new injector directly from its providers and implementations.
    /// Prefer [`Injector::builder()`] for creating new injectors instead.
    #[must_use]
    pub fn new(providers: ProviderMap) -> Self {
        Injector {
            provider_map: MapContainerEx::new(providers),
        }
    }

    /// Performs a request for a service. There are several types of requests
    /// that can be made to the service container by default:
    ///
    /// - [`Svc<T>`](crate::Svc): Requests a service pointer to the given
    ///   interface and creates an instance of the service if needed. If
    ///   multiple service providers are registered for that interface, then
    ///   returns an error instead.
    /// - [`Option<Svc<T>>`]: Requests a service pointer to the given interface
    ///   and create an instance of the service if needed. If no provider for
    ///   that service is registered, then returns `Ok(None)` rather than
    ///   returning an error. If multiple providers are registered, then
    ///   instead returns an error.
    /// - [`Services<T>`]: Requests all the implementations of an interface.
    ///   This will lazily create the services on demand. See the
    ///   [documentation for `Services<T>`](Services<T>) for more details.
    /// - [`Vec<Svc<T>>`]: Requests all the implementations of an interface.
    ///   This will eagerly create the services as part of the request.
    /// - [`Injector`]: Requests a clone of the injector. While it doesn't make
    ///   much sense to request this directly from the injector itself, this
    ///   allows the injector to be requested as a dependency inside of
    ///   services (for instance, factories).
    /// - [`RequestInfo`]: Requests information about the current request,
    ///   including the current resolution path.
    ///
    /// See the [documentation for `Request`](Request) for more information on
    /// what can be requested.
    ///
    /// Requests to service pointers of sized types will attempt to use the
    /// registered provider to retrieve an instance of that service. For
    /// instance, a request for a singleton service will create an instance of
    /// that service if one doesn't exist already, and either return a service
    /// pointer to the instance that was already created, or return a service
    /// pointer to the new instance (if one didn't exist already).
    ///
    /// ```
    /// use runtime_injector::{Injector, IntoSingleton, Svc};
    ///
    /// #[derive(Default)]
    /// struct Bar;
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Bar::default.singleton());
    ///
    /// let injector = builder.build();
    /// let _bar: Svc<Bar> = injector.get().unwrap();
    /// ```
    ///
    /// Requests to service pointers of `dyn Trait` interface types will
    /// instead request the implementation of that interface type. For example,
    /// if `dyn Foo`'s registered implementation is for the service type `Bar`,
    /// then a request for a service pointer of `dyn Foo` will return a service
    /// pointer to a `Bar`, although the return type will be `Svc<dyn Foo>`.
    ///
    /// ```
    /// use runtime_injector::{
    ///     interface, Injector, IntoSingleton, Svc, TypedProvider,
    /// };
    ///
    /// trait Foo: Send + Sync {}
    /// interface!(Foo = [Bar]);
    ///
    /// #[derive(Default)]
    /// struct Bar;
    /// impl Foo for Bar {}
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Bar::default.singleton().with_interface::<dyn Foo>());
    ///
    /// let injector = builder.build();
    /// let _bar: Svc<dyn Foo> = injector.get().unwrap();
    /// ```
    ///
    /// If multiple providers for a service exist, then a request for a single
    /// service pointer to that service will fail:
    ///
    /// ```
    /// use runtime_injector::{
    ///     interface, Injector, IntoSingleton, Svc, TypedProvider,
    /// };
    ///
    /// trait Foo: Send + Sync {}
    /// interface!(Foo = [Bar, Baz]);
    ///
    /// #[derive(Default)]
    /// struct Bar;
    /// impl Foo for Bar {}
    ///
    /// #[derive(Default)]
    /// struct Baz;
    /// impl Foo for Baz {}
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Bar::default.singleton().with_interface::<dyn Foo>());
    /// builder.provide(Baz::default.singleton().with_interface::<dyn Foo>());
    ///
    /// let injector = builder.build();
    /// assert!(injector.get::<Svc<dyn Foo>>().is_err());
    /// ```
    ///
    /// Custom request types can also be used by implementing [`Request`].
    pub fn get<R: Request>(&self) -> InjectResult<R> {
        self.get_with(RequestInfo::new())
    }

    /// Performs a request for a service with additional request information.
    ///
    /// # Example
    ///
    /// ```
    /// use runtime_injector::{
    ///     Injector, IntoTransient, Request, RequestInfo, Svc,
    /// };
    ///
    /// struct Foo(String);
    /// impl Foo {
    ///     fn new(request_info: RequestInfo) -> Self {
    ///         let value = request_info
    ///             .get_parameter("value")
    ///             .unwrap()
    ///             .downcast_ref::<String>()
    ///             .unwrap()
    ///             .clone();
    ///         Foo(value)
    ///     }
    /// }
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Foo::new.transient());
    ///
    /// let injector = builder.build();
    /// let mut request_info = RequestInfo::default();
    /// request_info.insert_parameter("value", "foo".to_owned());
    ///
    /// let foo: Svc<Foo> = injector.get_with(request_info).unwrap();
    /// assert_eq!("foo", foo.0);
    /// ```
    pub fn get_with<R: Request>(
        &self,
        request_info: RequestInfo,
    ) -> InjectResult<R> {
        R::request(self, request_info)
    }

    /// Gets implementations of a service from the container. This is
    /// equivalent to requesting [`Services<T>`] from [`Injector::get()`].
    pub(crate) fn get_service<I: ?Sized + Interface>(
        &self,
        request_info: RequestInfo,
    ) -> InjectResult<Services<I>> {
        Services::new(self.clone(), self.provider_map.clone(), request_info)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DynSvc, InjectError, InjectResult, Injector, Provider, RequestInfo,
        ServiceInfo, Svc,
    };
    use core::panic;

    #[test]
    fn get_exact_returns_error_on_invalid_provider() {
        struct BadProvider;
        impl Provider for BadProvider {
            fn result(&self) -> ServiceInfo {
                ServiceInfo::of::<i32>()
            }

            fn provide(
                &mut self,
                _injector: &Injector,
                _request_info: RequestInfo,
            ) -> InjectResult<DynSvc> {
                Ok(Svc::new(1.2f32))
            }
        }

        let mut builder = Injector::builder();
        builder.provide(BadProvider);

        let injector = builder.build();
        let bad: InjectResult<Svc<i32>> = injector.get();

        match bad {
            Err(InjectError::InvalidProvider { service_info })
                if service_info == ServiceInfo::of::<i32>() => {}
            Err(error) => Err(error).unwrap(),
            Ok(value) => {
                panic!("Value of {} was provided by an invalid provider", value)
            }
        }
    }
}
