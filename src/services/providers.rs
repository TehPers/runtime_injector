use crate::{
    DynSvc, InjectError, InjectResult, Injector, MapContainer, MapContainerEx,
    ProviderMap, Service, ServiceInfo, Svc,
};

/// Weakly typed service provider. Given an injector, this will provide an
/// implementation of a service. This is automatically implemented for all
/// types that implement `TypedProvider`, and `TypedProvider` should be
/// preferred if possible to allow for stronger type checking.
pub trait Provider: Service {
    /// The `ServiceInfo` which describes the type returned by this provider.
    fn result(&self) -> ServiceInfo;

    /// Provides an instance of the service.
    fn provide(&mut self, injector: &Injector) -> InjectResult<DynSvc>;
}

impl<T> Provider for T
where
    T: TypedProvider,
{
    fn result(&self) -> ServiceInfo {
        ServiceInfo::of::<T::Result>()
    }

    fn provide(&mut self, injector: &Injector) -> InjectResult<DynSvc> {
        let result = self.provide_typed(injector)?;
        Ok(result as DynSvc)
    }
}

/// A strongly-typed service provider. Types which implement this provide
/// instances of a service type when requested. Examples of typed providers
/// include providers created from service factories or constant providers.
/// This should be preferred over `Provider` for custom service providers if
/// possible due to the strong type guarantees this provides. `Provider` is
/// automatically implemented for all types which implement `TypedProvider`.
///
/// # Example
///
/// ```
/// use runtime_injector::{TypedProvider, Injector, InjectResult, Svc};
///
/// struct Foo;
///
/// struct FooProvider;
/// impl TypedProvider for FooProvider {
///     type Result = Foo;
///
///     fn provide_typed(&mut self, _injector: &Injector) -> InjectResult<Svc<Self::Result>> {
///         Ok(Svc::new(Foo))
///     }
/// }
///
/// let mut builder = Injector::builder();
/// builder.provide(FooProvider);
///
/// let injector = builder.build();
/// let _foo: Svc<Foo> = injector.get().unwrap();
/// ```
pub trait TypedProvider: Provider {
    /// The type of service this provider can activate.
    type Result: Service;

    /// Provides an instance of the service. The `Injector` passed in can be
    /// used to retrieve instances of any dependencies this service has.
    fn provide_typed(
        &mut self,
        injector: &Injector,
    ) -> InjectResult<Svc<Self::Result>>;
}

pub struct ServiceIter<'a> {
    providers: &'a mut Vec<Box<dyn Provider>>,
    injector: &'a Injector,
    index: usize,
}

impl<'a> Iterator for ServiceIter<'a> {
    type Item = InjectResult<DynSvc>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.providers.get_mut(self.index) {
            Some(provider) => {
                self.index += 1;
                Some(provider.provide(self.injector))
            }
            None => None,
        }
    }
}
