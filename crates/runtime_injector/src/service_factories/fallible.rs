use crate::{
    InjectError, InjectResult, Injector, RequestInfo, Service, ServiceFactory,
    ServiceInfo,
};
use std::{error::Error, marker::PhantomData};

/// A service factory that may fail during service creation with a custom error
/// type. During activation failure, an instance of
/// [`InjectError::ActivationFailed`] is returned as an error.
pub struct FallibleServiceFactory<D, R, E, F>
where
    D: Service,
    R: Service,
    E: Service + Error,
    F: ServiceFactory<D, Result = Result<R, E>>,
{
    inner: F,
    marker: PhantomData<fn(D) -> Result<R, E>>,
}

impl<D, R, E, F> ServiceFactory<D> for FallibleServiceFactory<D, R, E, F>
where
    D: Service,
    R: Service,
    E: Service + Error,
    F: ServiceFactory<D, Result = Result<R, E>>,
{
    type Result = R;

    fn invoke(
        &self,
        injector: &Injector,
        request_info: &RequestInfo,
    ) -> InjectResult<Self::Result> {
        let result = self.inner.invoke(injector, request_info)?;
        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(InjectError::ActivationFailed {
                service_info: ServiceInfo::of::<R>(),
                inner: Box::new(error),
            }),
        }
    }
}

/// Defines a conversion into a fallible service factory. This trait is
/// automatically implemented for all service factories that return a
/// [`Result<T, E>`] with an error type that implements [`Error`] and
/// [`Service`].
pub trait IntoFallible<D, R, E, F>
where
    D: Service,
    R: Service,
    E: Service + Error,
    F: ServiceFactory<D, Result = Result<R, E>>,
{
    /// Marks a service factory as being able to fail. On failure, an injection
    /// error is returned during activation. On success, the service is
    /// injected unwrapped from the result. In other words, a [`Result<T, E>`]
    /// can be requested as a [`Svc<T>`](crate::Svc), however if the
    /// constructor fails, an injection error is returned from the request.
    ///
    /// ## Example
    ///
    /// ```
    /// use runtime_injector::{
    ///     InjectError, InjectResult, Injector, IntoFallible, IntoTransient, Svc,
    /// };
    /// use std::{
    ///     error::Error,
    ///     fmt::{Display, Formatter},
    /// };
    ///
    /// #[derive(Debug)]
    /// struct FooError;
    ///
    /// impl Error for FooError {}
    /// impl Display for FooError {
    ///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "An error occurred while creating a Foo")
    ///     }
    /// }
    ///
    /// struct Foo(Svc<i32>);
    /// fn make_foo(a: Svc<i32>) -> Result<Foo, FooError> {
    ///     Err(FooError)
    /// }
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(make_foo.fallible().transient());
    /// builder.provide((|| 0).transient());
    ///
    /// let injector = builder.build();
    /// let foo_result: InjectResult<Svc<Foo>> = injector.get();
    /// match foo_result {
    ///     Err(InjectError::ActivationFailed { .. }) => {}
    ///     Err(error) => Err(error).unwrap(),
    ///     _ => unreachable!("activation should have failed"),
    /// }
    /// ```
    #[must_use]
    fn fallible(self) -> FallibleServiceFactory<D, R, E, F>;
}

impl<D, R, E, F> IntoFallible<D, R, E, F> for F
where
    D: Service,
    R: Service,
    E: Service + Error,
    F: ServiceFactory<D, Result = Result<R, E>>,
{
    fn fallible(self) -> FallibleServiceFactory<D, R, E, F> {
        FallibleServiceFactory {
            inner: self,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constant, IntoTransient, Svc};
    use std::{
        fmt::{Display, Formatter},
        sync::Mutex,
    };

    #[derive(Debug)]
    struct FooError;

    impl Error for FooError {}

    impl Display for FooError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "An error occurred while creating a Foo")
        }
    }

    struct Foo;
    fn make_foo(succeed: Svc<bool>) -> Result<Foo, FooError> {
        if *succeed { Ok(Foo) } else { Err(FooError) }
    }

    /// A value is returned if the service factory succeeds.
    #[test]
    fn test_fallible_service_factory_success() {
        let mut builder = Injector::builder();
        builder.provide(make_foo.fallible().transient());
        builder.provide(constant(true));

        let injector = builder.build();
        let _foo: Svc<Foo> = injector.get().unwrap();
    }

    /// A value is not returned if the service factory fails.
    #[test]
    fn test_fallible_service_factory_failure() {
        let mut builder = Injector::builder();
        builder.provide(make_foo.fallible().transient());
        builder.provide(constant(false));

        let injector = builder.build();
        let foo_result: InjectResult<Svc<Foo>> = injector.get();
        assert!(foo_result.is_err());
    }

    /// If a value fails after succeeding, an error is returned.
    #[test]
    fn test_fallible_service_factory_failure_after_success() {
        let mut builder = Injector::builder();
        builder.provide(make_foo.fallible().transient());
        builder.provide(constant(Mutex::new(true)));
        builder.provide(
            (|should_succeed: Svc<Mutex<bool>>| {
                *should_succeed.lock().unwrap()
            })
            .transient(),
        );

        // First request succeeds
        let injector = builder.build();
        let foo_result: InjectResult<Svc<Foo>> = injector.get();
        assert!(foo_result.is_ok());

        // Second request fails
        let should_succeed: Svc<Mutex<bool>> = injector.get().unwrap();
        *should_succeed.lock().unwrap() = false;
        drop(should_succeed);
        let foo_result: InjectResult<Svc<Foo>> = injector.get();
        assert!(foo_result.is_err());
    }

    /// If a value succeeds after failing, a value is returned.
    #[test]
    fn test_fallible_service_factory_success_after_failure() {
        let mut builder = Injector::builder();
        builder.provide(make_foo.fallible().transient());
        builder.provide(constant(Mutex::new(false)));
        builder.provide(
            (|should_succeed: Svc<Mutex<bool>>| {
                *should_succeed.lock().unwrap()
            })
            .transient(),
        );

        // First request fails
        let injector = builder.build();
        let foo_result: InjectResult<Svc<Foo>> = injector.get();
        assert!(foo_result.is_err());

        // Second request succeeds
        let should_succeed: Svc<Mutex<bool>> = injector.get().unwrap();
        *should_succeed.lock().unwrap() = true;
        drop(should_succeed);
        let foo_result: InjectResult<Svc<Foo>> = injector.get();
        assert!(foo_result.is_ok());
    }
}
