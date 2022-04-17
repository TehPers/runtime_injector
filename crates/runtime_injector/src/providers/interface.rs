use crate::{
    InjectResult, Injector, InterfaceFor, RequestInfo, Svc, TypedProvider,
};
use std::marker::PhantomData;

/// Provides a service as an implementation of an interface. See
/// [`TypedProvider::with_interface()`] for more information.
pub struct InterfaceProvider<I, P>
where
    P: TypedProvider,
    I: ?Sized + InterfaceFor<P::Result>,
{
    inner: P,
    _marker: PhantomData<fn(P::Result) -> I>,
}

impl<I, P> TypedProvider for InterfaceProvider<I, P>
where
    P: TypedProvider,
    I: ?Sized + InterfaceFor<P::Result>,
{
    type Interface = I;
    type Result = P::Result;

    fn provide_typed(
        &self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Svc<Self::Result>> {
        self.inner.provide_typed(injector, request_info)
    }

    fn provide_owned_typed(
        &self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Box<Self::Result>> {
        self.inner.provide_owned_typed(injector, request_info)
    }
}

/// Defines a conversion into an interface provider. This trait is
/// automatically implemented for all types that implement [`TypedProvider<I>`].
pub trait WithInterface: TypedProvider {
    /// Provides this service as an implementation of a particular interface.
    /// Rather than requesting this service with its concrete type, it is
    /// instead requested by its interface type. By default, all services are
    /// assigned to the [`dyn Service`] interface. Any services assigned to the
    /// [`dyn Service`] interface can be requested directly by their concrete
    /// type. Other services cannot be requested by their concrete types once
    /// they has been assigned another interface.
    ///
    /// [`dyn Service`]: crate::Service
    ///
    /// ## Example
    ///
    /// ```
    /// use runtime_injector::{
    ///     interface, InjectResult, Injector, IntoSingleton, Service, Svc,
    ///     TypedProvider, WithInterface,
    /// };
    ///
    /// trait Fooable: Service {
    ///     fn bar(&self) {}
    /// }
    ///
    /// interface!(Fooable);
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
    ) -> InterfaceProvider<I, Self>;
}

impl<P> WithInterface for P
where
    P: TypedProvider,
{
    fn with_interface<I: ?Sized + InterfaceFor<Self::Result>>(
        self,
    ) -> InterfaceProvider<I, Self> {
        InterfaceProvider {
            inner: self,
            _marker: PhantomData,
        }
    }
}
