use crate::{Dependencies, InjectResult, Injector, ProviderFunction, Service, Svc, TypedProvider};
use std::marker::PhantomData;

pub struct SingletonProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    func: F,
    result: Option<Svc<R>>,
    marker: PhantomData<fn(D) -> InjectResult<R>>,
}

impl<D, R, F> SingletonProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    pub fn new(func: F) -> Self {
        SingletonProvider {
            func,
            result: None,
            marker: PhantomData,
        }
    }
}

impl<D, R, F> TypedProvider for SingletonProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    type Result = R;

    fn provide_typed(&mut self, injector: &mut Injector) -> InjectResult<Svc<Self::Result>> {
        if let Some(ref service) = self.result {
            return Ok(service.clone());
        }

        let result = self.func.invoke(injector)?;
        self.result = Some(result.clone());
        Ok(result)
    }
}

/// Defines a conversion into a singleton provider. This trait is automatically
/// implemented for all functions which implement `ProviderFunction`.
pub trait IntoSingleton<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    /// Creates a singleton provider. Singleton providers create their values
    /// only once (when first requested) and reuse that value for each future
    /// request.
    ///
    /// # Example
    ///
    /// ```
    /// use runtime_injector::{IntoSingleton, Injector, Svc};
    ///
    /// #[derive(Default)]
    /// struct Foo;
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Foo::default.singleton());
    ///
    /// let mut injector = builder.build();
    /// let foo1: Svc<Foo> = injector.get().unwrap();
    /// let foo2: Svc<Foo> = injector.get().unwrap();
    ///
    /// assert!(Svc::ptr_eq(&foo1, &foo2));
    /// ```
    fn singleton(self) -> SingletonProvider<D, R, F>;
}

impl<D, R, F> IntoSingleton<D, R, F> for F
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    fn singleton(self) -> SingletonProvider<D, R, F> {
        SingletonProvider::new(self)
    }
}

impl<D, R, F> From<F> for SingletonProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    fn from(func: F) -> Self {
        func.singleton()
    }
}
