use crate::{
    InjectError, InjectResult, Injector, Interface, InterfaceFor, RequestInfo,
    Service, ServiceInfo, Svc,
};

/// Weakly typed service provider.
///
/// Given an injector, this can provide an instance of an interface. This is
/// automatically implemented for all types that implement [`TypedProvider`],
/// and [`TypedProvider`] should be preferred if possible for custom service
/// providers to allow for stronger type checking.
pub trait Provider: Service {
    /// The interface this provider is providing for.
    type Interface: ?Sized + Interface;

    /// The [`ServiceInfo`] which describes the type returned by this provider.
    fn result(&self) -> ServiceInfo;

    /// Provides an instance of the service.
    fn provide(
        &mut self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Svc<Self::Interface>>;

    /// Provides an owned instance of the service.
    fn provide_owned(
        &mut self,
        _injector: &Injector,
        _request_info: &RequestInfo,
    ) -> InjectResult<Box<Self::Interface>> {
        Err(InjectError::OwnedNotSupported {
            service_info: self.result(),
        })
    }
}

impl<T> Provider for T
where
    T: TypedProvider,
{
    type Interface = <T as TypedProvider>::Interface;

    fn result(&self) -> ServiceInfo {
        ServiceInfo::of::<T::Result>()
    }

    fn provide(
        &mut self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Svc<Self::Interface>> {
        let service = self.provide_typed(injector, request_info)?;
        Ok(Self::Interface::from_svc(service))
    }

    fn provide_owned(
        &mut self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Box<Self::Interface>> {
        let service = self.provide_owned_typed(injector, request_info)?;
        Ok(Self::Interface::from_owned_svc(service))
    }
}

/// A strongly-typed service provider.
///
/// Types which implement this trait can provide strongly-typed instances of a
/// particular service type. Examples of typed providers include providers
/// created from service factories or constant providers. This should be
/// preferred over [`Provider`] for custom service providers if possible due to
/// the strong type guarantees this provides. [`Provider`] is automatically
/// implemented for all types which implement [`TypedProvider`].
///
/// ## Example
///
/// ```
/// use runtime_injector::{
///     InjectResult, Injector, RequestInfo, Svc, TypedProvider,
/// };
///
/// struct Foo;
///
/// struct FooProvider;
/// impl TypedProvider for FooProvider {
///     type Result = Foo;
///
///     fn provide_typed(
///         &mut self,
///         _injector: &Injector,
///         _request_info: &RequestInfo,
///     ) -> InjectResult<Svc<Self::Result>> {
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
pub trait TypedProvider:
    Sized + Provider<Interface = <Self as TypedProvider>::Interface>
{
    /// The interface this provider is providing for.
    type Interface: ?Sized + InterfaceFor<Self::Result>;

    /// The type of service this can provide.
    type Result: Service;

    /// Provides an instance of the service. The [`Injector`] passed in can be
    /// used to retrieve instances of any dependencies this service has.
    fn provide_typed(
        &mut self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Svc<Self::Result>>;

    /// Provides an owned instance of the service. Not all providers can
    /// provide an owned variant of the service.
    fn provide_owned_typed(
        &mut self,
        _injector: &Injector,
        _request_info: &RequestInfo,
    ) -> InjectResult<Box<Self::Result>> {
        Err(InjectError::OwnedNotSupported {
            service_info: ServiceInfo::of::<Self::Result>(),
        })
    }
}
