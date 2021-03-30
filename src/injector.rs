use crate::{
    constant, DynSvc, InjectError, InjectResult, InjectorBuilder, Provider,
    Request, Service, ServiceInfo, Svc,
};
use std::collections::HashMap;

type ProviderMap = HashMap<ServiceInfo, Option<Box<dyn Provider>>>;
type ImplementationMap = HashMap<ServiceInfo, ServiceInfo>;

trait MapContainerEx<T> {
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
use types::*;

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
/// use runtime_injector::{Injector, Svc, IntoTransient, IntoSingleton, constant, InjectResult};
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
    providers: MapContainer<ProviderMap>,
    implementations: MapContainer<ImplementationMap>,
}

impl Injector {
    /// Creates a build for this injector. This is the preferred way of
    /// creating an injector.
    #[must_use]
    pub fn builder() -> InjectorBuilder {
        InjectorBuilder::default()
    }

    /// Creates a new injector directly from its providers and implementations.
    /// Prefer `Injector::builder()` for creating new injectors instead.
    #[must_use]
    pub fn new(
        providers: ProviderMap,
        implementations: ImplementationMap,
    ) -> Self {
        let injector = Injector {
            providers: MapContainerEx::new(providers),
            implementations: MapContainerEx::new(implementations),
        };

        // Insert the injector as a service if there isn't an injector already.
        injector.providers.with_inner_mut(|providers| {
            providers
                .entry(ServiceInfo::of::<Injector>())
                .or_insert_with(|| Some(Box::new(constant(injector.clone()))));
        });

        injector
    }

    /// Performs a request for a service. There are several types of requests
    /// that can be made to the service container by default:
    ///
    /// - `Svc<I>`: Request a service pointer to the given interface and create
    ///   an instance of the service if needed.
    /// - `Option<Svc<I>>`: Request a service pointer to the given interface and
    ///   create an instance of the service if needed. If no provider for that
    ///   service is registered, then return `Ok(None)` rather than throwing an
    ///   error.
    ///
    /// Requests to service pointers of sized types will attempt to use the
    /// a registered provider to retrieve an instance of that service. For
    /// instance, a request for a singleton service will create an instance of
    /// that service if one doesn't exist already, and either return a service
    /// pointer to the instance that was already created, or return a service
    /// pointer to the new instance (if one didn't exist already).
    ///
    /// ```
    /// use runtime_injector::{Injector, Svc, IntoSingleton};
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
    /// use runtime_injector::{interface, Injector, Svc, IntoSingleton};
    ///
    /// trait Foo: Send + Sync {}
    /// interface!(Foo = [Bar]);
    ///
    /// #[derive(Default)]
    /// struct Bar;
    /// impl Foo for Bar {}
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Bar::default.singleton());
    /// builder.implement::<dyn Foo, Bar>();
    ///
    /// let injector = builder.build();
    /// let _bar: Svc<dyn Foo> = injector.get().unwrap();
    /// ```
    ///
    /// Custom request types can also be used by implementing `Request`.
    pub fn get<R: Request>(&self) -> InjectResult<R> {
        R::request(self)
    }

    /// Gets the service info for the registered implementation of a particular
    /// interface. This is only used by `dyn Trait` interface types to request
    /// the registered implementation of that trait. For sized service types,
    /// the implementation is always the type itself.
    #[must_use]
    pub fn get_implementation(
        &self,
        interface: ServiceInfo,
    ) -> Option<ServiceInfo> {
        // TODO: not clone every time here, maybe get rid of this function
        // entirely
        self.implementations.with_inner(|implementations| {
            implementations.get(&interface).copied()
        })
    }

    /// Gets an instance of the service with exactly the type that was
    /// requested. This will not attempt to find the type registered as an
    /// implementation of a particular trait. In fact, dynamic types (`dyn
    /// Trait`) cannot be used with this function.
    #[allow(clippy::clippy::map_err_ignore)]
    pub fn get_exact<T: Service>(&self) -> InjectResult<Svc<T>> {
        let service_info = ServiceInfo::of::<T>();
        self.get_dyn_exact(service_info)?
            .downcast()
            .map_err(|_| InjectError::InvalidProvider { service_info })
    }

    /// Similar to `get_exact`, but returns an instance of `dyn Any` instead,
    /// and does not need the type passed in via a type parameter.
    pub fn get_dyn_exact(
        &self,
        service_info: ServiceInfo,
    ) -> InjectResult<DynSvc> {
        // Extract the provider for the requested type so that the lock can be
        // freed before requesting the service (since it's recursive)
        let mut provider = self.providers.with_inner_mut(|providers| {
            providers
                .get_mut(&service_info)
                .ok_or(InjectError::MissingProvider { service_info })?
                .take()
                .ok_or(InjectError::CycleDetected {
                    service_info,
                    cycle: vec![service_info],
                })
        })?;

        // Request the service from the provider now that the lock is freed
        let result = match provider.provide(self) {
            Ok(result) => result,
            Err(InjectError::CycleDetected { mut cycle, .. }) => {
                cycle.push(service_info);
                return Err(InjectError::CycleDetected {
                    service_info,
                    cycle,
                });
            }
            Err(e) => return Err(e),
        };

        // Reinsert the provider back into the map so it can be reused
        self.providers.with_inner_mut(move |providers| {
            let provider_entry =
                providers.get_mut(&service_info).ok_or_else(|| {
                    InjectError::InternalError(format!(
                        "activated provider for {} is no longer registered",
                        service_info.name()
                    ))
                })?;

            if provider_entry.replace(provider).is_some() {
                Err(InjectError::InternalError(format!(
                    "another provider for {} was added during its activation",
                    service_info.name()
                )))
            } else {
                Ok(result)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::{
        DynSvc, InjectError, InjectResult, Injector, Provider, ServiceInfo, Svc,
    };

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
            ) -> InjectResult<DynSvc> {
                Ok(Svc::new(1.2_f32))
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
