use std::marker::PhantomData;

use crate::{
    DynSvc, InjectError, InjectResult, Injector, Interface, InterfaceFor,
    OwnedDynSvc, RequestInfo, Service, ServiceInfo, Svc,
};

/// Weakly typed service provider. Given an injector, this will provide an
/// implementation of a service. This is automatically implemented for all
/// types that implement [`TypedProvider`], and [`TypedProvider`] should be
/// preferred if possible to allow for stronger type checking.
pub trait Provider: Service {
    /// The [`ServiceInfo`] which describes the type returned by this provider.
    fn result(&self) -> ServiceInfo;

    /// Provides an instance of the service.
    fn provide(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<DynSvc>;

    /// Provides an owned instance of the service.
    fn provide_owned(
        &mut self,
        _injector: &Injector,
        _request_info: RequestInfo,
    ) -> InjectResult<OwnedDynSvc> {
        Err(InjectError::OwnedNotSupported {
            service_info: self.result(),
        })
    }
}

impl<T> Provider for T
where
    T: TypedProvider,
{
    fn result(&self) -> ServiceInfo {
        ServiceInfo::of::<T::Result>()
    }

    fn provide(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<DynSvc> {
        let result = self.provide_typed(injector, request_info)?;
        Ok(result as DynSvc)
    }

    fn provide_owned(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<OwnedDynSvc> {
        let result = self.provide_owned_typed(injector, request_info)?;
        Ok(result as OwnedDynSvc)
    }
}

/// A strongly-typed service provider. Types which implement this provide
/// instances of a service type when requested. Examples of typed providers
/// include providers created from service factories or constant providers.
/// This should be preferred over [`Provider`] for custom service providers if
/// possible due to the strong type guarantees this provides. [`Provider`] is
/// automatically implemented for all types which implement [`TypedProvider`].
///
/// # Example
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
///         _request_info: RequestInfo,
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
pub trait TypedProvider: Sized + Provider {
    /// The type of service this provider can activate.
    type Result: Interface;

    /// Provides an instance of the service. The [`Injector`] passed in can be
    /// used to retrieve instances of any dependencies this service has.
    fn provide_typed(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<Svc<Self::Result>>;

    /// Provides an owned instance of the service. Not all providers can
    /// provide an owned variant of the service.
    fn provide_owned_typed(
        &mut self,
        _injector: &Injector,
        _request_info: RequestInfo,
    ) -> InjectResult<Box<Self::Result>> {
        Err(InjectError::OwnedNotSupported {
            service_info: ServiceInfo::of::<Self::Result>(),
        })
    }

    /// Provides this service as an implementation of a particular interface.
    /// Rather than requesting this service with its concrete type, it can
    /// instead be requested by its interface type.
    ///
    /// ```
    /// use runtime_injector::{
    ///     interface, InjectResult, Injector, IntoSingleton, Service, Svc,
    ///     TypedProvider,
    /// };
    ///
    /// trait Fooable: Service {
    ///     fn bar(&self) {}
    /// }
    ///
    /// interface!(Fooable = [Foo]);
    ///
    /// #[derive(Default)]
    /// struct Foo;
    /// impl Fooable for Foo {}
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Foo::default.singleton().with_interface::<dyn Fooable>());
    ///
    /// // Foo can now be requested through its interface of `dyn Fooable`.
    /// let injector = builder.build();
    /// let fooable: Svc<dyn Fooable> = injector.get().unwrap();
    /// fooable.bar();
    ///
    /// // It can't be requested through its original type
    /// assert!(injector.get::<Svc<Foo>>().is_err());
    /// ```
    fn with_interface<I: ?Sized + InterfaceFor<Self::Result>>(
        self,
    ) -> InterfaceProvider<I, Self> {
        InterfaceProvider {
            inner: self,
            marker: PhantomData,
        }
    }
}

/// Provides a service as an implementation of an interface. See
/// [`TypedProvider::with_interface()`] for more information.
pub struct InterfaceProvider<I, P>
where
    P: TypedProvider,
    I: ?Sized + InterfaceFor<P::Result>,
{
    inner: P,
    marker: PhantomData<fn() -> I>,
}

impl<I, P> Provider for InterfaceProvider<I, P>
where
    P: TypedProvider,
    I: ?Sized + InterfaceFor<P::Result>,
{
    fn result(&self) -> ServiceInfo {
        ServiceInfo::of::<I>()
    }

    fn provide(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<DynSvc> {
        self.inner.provide(injector, request_info)
    }

    fn provide_owned(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<OwnedDynSvc> {
        self.inner.provide_owned(injector, request_info)
    }
}
