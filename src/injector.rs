use crate::{
    DynSvc, InjectError, InjectResult, InjectorBuilder, Provider, Request,
    Service, ServiceInfo, Svc,
};
use std::collections::HashMap;

/// A runtime dependency injection container. This holds all the bindings
/// between service types and their providers, as well as all the mappings from
/// interfaces to their implementations (if they differ).
pub struct Injector {
    providers: HashMap<ServiceInfo, Option<Box<dyn Provider>>>,
    implementations: HashMap<ServiceInfo, ServiceInfo>,
}

impl Injector {
    #[must_use]
    pub fn builder() -> InjectorBuilder {
        InjectorBuilder::default()
    }

    #[must_use]
    pub fn new(
        providers: HashMap<ServiceInfo, Option<Box<dyn Provider>>>,
        implementations: HashMap<ServiceInfo, ServiceInfo>,
    ) -> Self {
        Injector {
            providers,
            implementations,
        }
    }

    // /// Gets an implementation of the given type. If the type is a sized type,
    // /// then this will attempt to activate an instance of that type using a
    // /// registered provider. If the type is a dynamic type (`dyn Trait`), then
    // /// an instance of the type registered as the implementation of that trait will
    // /// be activated instead.
    // pub fn get<T: ?Sized + Interface>(&mut self) -> InjectResult<Svc<T>> {
    //     T::resolve(
    //         self,
    //         self.implementations.get(&ServiceInfo::of::<T>()).copied(),
    //     )
    // }

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
    /// let mut injector = builder.build();
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
    /// let mut injector = builder.build();
    /// let _bar: Svc<dyn Foo> = injector.get().unwrap();
    /// ```
    ///
    /// Custom request types can also be used by implementing `Request`.
    pub fn get<R: Request>(&mut self) -> InjectResult<R> {
        R::request(self)
    }

    /// Gets the service info for the registered implementation of a particular
    /// interface. This is only used by `dyn Trait` interface types to request
    /// the registered implementation of that trait. For sized service types,
    /// the implementation is always the type itself.
    pub fn get_implementation(
        &mut self,
        interface: ServiceInfo,
    ) -> Option<ServiceInfo> {
        self.implementations.get(&interface).copied()
    }

    /// Gets an instance of the service with exactly the type that was
    /// requested. This will not attempt to find the type registered as an
    /// implementation of a particular trait. In fact, dynamic types (`dyn
    /// Trait`) cannot be used with this function.
    #[allow(clippy::clippy::map_err_ignore)]
    pub fn get_exact<T: Service>(&mut self) -> InjectResult<Svc<T>> {
        let service_info = ServiceInfo::of::<T>();
        self.get_dyn_exact(service_info)?
            .downcast()
            .map_err(|_| InjectError::InvalidProvider { service_info })
    }

    /// Similar to `get_exact`, but returns an instance of `dyn Any` instead,
    /// and does not need the type passed in via a type parameter.
    pub fn get_dyn_exact(
        &mut self,
        service_info: ServiceInfo,
    ) -> InjectResult<DynSvc> {
        let provider = self
            .providers
            .get_mut(&service_info)
            .ok_or(InjectError::MissingProvider { service_info })?;

        let mut provider =
            provider.take().ok_or(InjectError::CycleDetected {
                service_info,
                cycle: vec![service_info],
            })?;

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

        // Need to get the entry again since it could have been removed by a provider (it shouldn't have though)
        let provider_entry =
            self.providers.get_mut(&service_info).ok_or_else(|| {
                InjectError::InternalError(format!(
                    "activated provider for {} is no longer registered",
                    service_info.name()
                ))
            })?;
        if provider_entry.replace(provider).is_some() {
            return Err(InjectError::InternalError(format!(
                "another provider for {} was added during its activation",
                service_info.name()
            )));
        }

        Ok(result)
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
                _injector: &mut Injector,
            ) -> InjectResult<DynSvc> {
                Ok(Svc::new(1.2_f32))
            }
        }

        let mut builder = Injector::builder();
        builder.provide(BadProvider);

        let mut injector = builder.build();
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
