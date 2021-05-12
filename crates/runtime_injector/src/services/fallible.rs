use crate::{InjectError, InjectResult, Service, ServiceFactory, ServiceInfo};
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
        &mut self,
        injector: &crate::Injector,
        request_info: crate::RequestInfo,
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
