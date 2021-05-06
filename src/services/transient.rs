use crate::{
    InjectResult, Injector, RequestInfo, Service, ServiceFactory, Svc,
    TypedProvider,
};
use std::marker::PhantomData;

/// A service provider that creates an instance of the service each time it is
/// requested. This will never return two service pointers to the same instance
/// of a service.
pub struct TransientProvider<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    factory: F,
    marker: PhantomData<fn(D) -> InjectResult<R>>,
}

impl<D, R, F> TransientProvider<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    /// Creates a new [`TransientProvider`] using a service factory.
    #[must_use]
    pub fn new(func: F) -> Self {
        TransientProvider {
            factory: func,
            marker: PhantomData,
        }
    }
}

impl<D, R, F> TypedProvider for TransientProvider<D, R, F>
where
    D: Service,
    R: Service,
    F: ServiceFactory<D, Result = R> + Service,
{
    type Result = R;

    fn provide_typed(
        &mut self,
        injector: &Injector,
        request_info: RequestInfo,
    ) -> InjectResult<Svc<Self::Result>> {
        let result = self.factory.invoke(injector, request_info)?;
        Ok(Svc::new(result))
    }
}

/// Defines a conversion into a transient provider. This trait is automatically
/// implemented for all service factories.
pub trait IntoTransient<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    /// Creates a transient provider. Transient providers create their values
    /// each time the service is requested and will never return service
    /// pointers to the same instance more than once.
    ///
    /// # Example
    ///
    /// ```
    /// use runtime_injector::{IntoTransient, Injector, Svc};
    ///
    /// #[derive(Default)]
    /// struct Foo;
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Foo::default.transient());
    ///
    /// let injector = builder.build();
    /// let foo1: Svc<Foo> = injector.get().unwrap();
    /// let foo2: Svc<Foo> = injector.get().unwrap();
    ///
    /// assert!(!Svc::ptr_eq(&foo1, &foo2));
    /// ```
    #[must_use]
    fn transient(self) -> TransientProvider<D, R, F>;
}

impl<D, R, F> IntoTransient<D, R, F> for F
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    fn transient(self) -> TransientProvider<D, R, F> {
        TransientProvider::new(self)
    }
}

impl<D, R, F> From<F> for TransientProvider<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    fn from(func: F) -> Self {
        func.transient()
    }
}
