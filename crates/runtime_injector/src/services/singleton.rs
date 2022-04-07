use crate::{
    InjectResult, Injector, RequestInfo, Service, ServiceFactory, Svc,
    TypedProvider,
};
use std::marker::PhantomData;

/// A service provider that only creates a single instance of the service.
/// The service is created only during its first request. Any subsequent
/// requests return service pointers to the same service.
pub struct SingletonProvider<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    factory: F,
    result: Option<Svc<R>>,
    marker: PhantomData<fn(D) -> R>,
}

impl<D, R, F> SingletonProvider<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    /// Creates a new [`SingletonProvider`] using a service factory.
    #[must_use]
    pub fn new(func: F) -> Self {
        SingletonProvider {
            factory: func,
            result: None,
            marker: PhantomData,
        }
    }
}

impl<D, R, F> TypedProvider for SingletonProvider<D, R, F>
where
    D: Service,
    R: Service,
    F: Service + ServiceFactory<D, Result = R>,
{
    type Result = R;

    fn provide_typed(
        &mut self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Svc<Self::Result>> {
        if let Some(ref service) = self.result {
            return Ok(service.clone());
        }

        let result = self.factory.invoke(injector, request_info)?;
        let result = Svc::new(result);
        self.result = Some(result.clone());
        Ok(result)
    }
}

/// Defines a conversion into a singleton provider. This trait is automatically
/// implemented for all service factories.
pub trait IntoSingleton<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    /// Creates a singleton provider. Singleton providers create their values
    /// only once (when first requested) and reuse that value for each future
    /// request.
    ///
    /// ## Example
    ///
    /// ```
    /// use runtime_injector::{Injector, IntoSingleton, Svc};
    ///
    /// #[derive(Default)]
    /// struct Foo;
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Foo::default.singleton());
    ///
    /// let injector = builder.build();
    /// let foo1: Svc<Foo> = injector.get().unwrap();
    /// let foo2: Svc<Foo> = injector.get().unwrap();
    ///
    /// assert!(Svc::ptr_eq(&foo1, &foo2));
    /// ```
    #[must_use]
    fn singleton(self) -> SingletonProvider<D, R, F>;
}

impl<D, R, F> IntoSingleton<D, R, F> for F
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    fn singleton(self) -> SingletonProvider<D, R, F> {
        SingletonProvider::new(self)
    }
}

impl<D, R, F> From<F> for SingletonProvider<D, R, F>
where
    R: Service,
    F: ServiceFactory<D, Result = R>,
{
    fn from(func: F) -> Self {
        func.singleton()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    struct Foo(i32);

    /// Singleton provider provides the correct value.
    #[test]
    fn singleton_provider_provides_correct_value() {
        let mut builder = Injector::builder();
        builder.provide((|| Foo(42)).singleton());

        let injector = builder.build();
        let foo: Svc<Foo> = injector.get().unwrap();
        assert_eq!(&*foo, &Foo(42));
    }

    /// When value is mutated, the provider returns the mutated value.
    #[test]
    fn singleton_provider_returns_mutated_value() {
        let mut builder = Injector::builder();
        builder.provide((|| Mutex::new(Foo(0))).singleton());

        let injector = builder.build();
        let foo: Svc<Mutex<Foo>> = injector.get().unwrap();
        let mut foo = foo.lock().unwrap();
        foo.0 = 42;
        drop(foo);

        let foo: Svc<Mutex<Foo>> = injector.get().unwrap();
        let foo = foo.lock().unwrap();
        assert_eq!(&*foo, &Foo(42));
    }
}
