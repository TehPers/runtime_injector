use crate::{
    InjectError, InjectResult, Injector, Request, RequestInfo, Service,
    ServiceInfo, TypedProvider,
};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

/// Allows custom pre-defined values to be passed as arguments to services.
///
/// See [WithArg::with_arg()].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub struct Arg<T: Service + Clone>(T);

impl<T: Service + Clone> Arg<T> {
    /// Gets the parameter name of an [`Arg<T>`] requested by a particular
    /// service.
    pub fn param_name(target: ServiceInfo) -> String {
        format!(
            "runtime_injector::Arg[target={:?},type={:?}]",
            target.id(),
            ServiceInfo::of::<T>().id()
        )
    }

    /// Converts an argument into its inner value.
    pub fn into_inner(arg: Self) -> T {
        arg.0
    }
}

impl<T: Service + Clone> Deref for Arg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Service + Clone> DerefMut for Arg<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display + Service + Clone> Display for Arg<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Allows custom pre-defined values to be passed as arguments to services.
impl<T: Service + Clone> Request for Arg<T> {
    fn request(_injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        let parent_request = info.service_path().last().ok_or_else(|| {
            InjectError::ActivationFailed {
                service_info: ServiceInfo::of::<Self>(),
                inner: Box::new(ArgRequestError::NoParentRequest),
            }
        })?;

        let request_name = Self::param_name(*parent_request);
        let param = info.get_parameter(&request_name).ok_or_else(|| {
            InjectError::ActivationFailed {
                service_info: ServiceInfo::of::<Self>(),
                inner: Box::new(ArgRequestError::MissingParameter),
            }
        })?;

        let param: &T = param.downcast_ref().ok_or_else(|| {
            InjectError::ActivationFailed {
                service_info: ServiceInfo::of::<Self>(),
                inner: Box::new(ArgRequestError::ParameterTypeInvalid),
            }
        })?;

        Ok(Arg(param.clone()))
    }
}

/// An error occurred while injecting an instance of [`Arg<T>`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ArgRequestError {
    /// The argument value was not provided.
    MissingParameter,
    /// The argument value is the wrong type. This should never happen.
    ParameterTypeInvalid,
    /// There is no parent request.
    NoParentRequest,
}

impl Error for ArgRequestError {}

impl Display for ArgRequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgRequestError::MissingParameter => {
                write!(f, "no value assigned for this argument")
            }
            ArgRequestError::ParameterTypeInvalid => {
                write!(f, "argument value is the wrong type")
            }
            ArgRequestError::NoParentRequest => {
                write!(f, "no parent request was found")
            }
        }
    }
}

/// A service provider that attaches an argument to a service. This can be used
/// to pass custom values to service factories. See [`WithArg::with_arg()`].
pub struct ArgProvider<P, T>
where
    P: TypedProvider,
    P::Result: Sized,
    T: Service + Clone,
{
    inner: P,
    arg: T,
}

impl<P, T> TypedProvider for ArgProvider<P, T>
where
    P: TypedProvider,
    P::Result: Sized,
    T: Service + Clone,
{
    type Interface = <P as TypedProvider>::Interface;
    type Result = P::Result;

    fn provide_typed(
        &self,
        injector: &crate::Injector,
        request_info: &crate::RequestInfo,
    ) -> crate::InjectResult<crate::Svc<Self::Result>> {
        let mut request_info = request_info.clone();
        let _ = request_info.insert_parameter(
            &Arg::<T>::param_name(ServiceInfo::of::<Self::Result>()),
            self.arg.clone(),
        );
        self.inner.provide_typed(injector, &request_info)
    }

    fn provide_owned_typed(
        &self,
        injector: &crate::Injector,
        request_info: &crate::RequestInfo,
    ) -> crate::InjectResult<Box<Self::Result>> {
        let mut request_info = request_info.clone();
        let _ = request_info.insert_parameter(
            &Arg::<T>::param_name(ServiceInfo::of::<Self::Result>()),
            self.arg.clone(),
        );
        self.inner.provide_owned_typed(injector, &request_info)
    }
}

/// Allows defining pre-defined arguments to services.
pub trait WithArg: TypedProvider
where
    Self::Result: Sized,
{
    /// Adds an argument for a service.
    ///
    /// ## Example
    ///
    /// ```
    /// use runtime_injector::{Arg, Injector, IntoSingleton, Svc, WithArg};
    ///
    /// struct DatabaseConnection {
    ///     connection_string: String,
    /// }
    ///
    /// impl DatabaseConnection {
    ///     fn new(connection_string: Arg<String>) -> Self {
    ///         Self {
    ///             connection_string: Arg::into_inner(connection_string),
    ///         }
    ///     }
    /// }
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(
    ///     DatabaseConnection::new
    ///         .singleton()
    ///         .with_arg("<connection string>".to_string()),
    /// );
    ///
    /// let injector = builder.build();
    /// let db: Svc<DatabaseConnection> = injector.get().unwrap();
    /// assert_eq!("<connection string>", db.connection_string);
    /// ```
    fn with_arg<T>(self, arg: T) -> ArgProvider<Self, T>
    where
        T: Service + Clone;
}

impl<P> WithArg for P
where
    P: TypedProvider,
    P::Result: Sized,
{
    fn with_arg<T>(self, arg: T) -> ArgProvider<Self, T>
    where
        T: Service + Clone,
    {
        ArgProvider { inner: self, arg }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        define_module, interface, Arg, ArgRequestError, InjectError, Injector,
        IntoSingleton, Service, ServiceInfo, Svc, WithArg, WithInterface,
    };

    #[derive(Debug, Default)]
    struct Foo(Arg<i32>);

    trait Fooable: Service {
        fn value(&self) -> i32;
    }
    interface!(Fooable);
    impl Fooable for Foo {
        fn value(&self) -> i32 {
            self.0.0
        }
    }

    #[test]
    fn request_fails_if_missing_arg() {
        // Create a module with a single service.
        let module = define_module! {
            services = [Foo.singleton()],
        };

        // Create an injector with the module.
        let mut builder = Injector::builder();
        builder.add_module(module);

        // Attempt to get the service.
        let injector = builder.build();
        let error = injector.get::<Svc<Foo>>().unwrap_err();
        match error {
            InjectError::ActivationFailed {
                service_info,
                inner,
            } => {
                // Check that the service info is correct.
                assert_eq!(ServiceInfo::of::<Arg<i32>>(), service_info,);

                // Check that the inner error is correct.
                match inner.downcast_ref::<ArgRequestError>() {
                    Some(ArgRequestError::MissingParameter) => (),
                    _ => panic!("unexpected error: {:?}", inner),
                }
            }
            _ => panic!("unexpected error: {:?}", error),
        }
    }

    #[test]
    fn request_fails_if_arg_has_no_parent_request() {
        let builder = Injector::builder();
        let injector = builder.build();
        match injector.get::<Arg<i32>>() {
            Ok(_) => unreachable!("request should have failed"),
            Err(InjectError::ActivationFailed {
                service_info,
                inner,
            }) => {
                assert_eq!(ServiceInfo::of::<Arg<i32>>(), service_info);
                let inner: &ArgRequestError =
                    inner.downcast_ref().expect("failed to downcast error");
                match inner {
                    ArgRequestError::NoParentRequest => {}
                    inner => Err(inner).unwrap(),
                }
            }
            Err(error) => Err(error).unwrap(),
        }
    }

    #[test]
    fn request_fails_if_arg_is_wrong_type() {
        let mut builder = Injector::builder();
        builder.provide(Foo.singleton().with_arg(42u32));

        let injector = builder.build();
        match injector.get::<Svc<Foo>>() {
            Ok(_) => unreachable!("request should have failed"),
            Err(InjectError::ActivationFailed {
                service_info,
                inner,
            }) => {
                assert_eq!(ServiceInfo::of::<Arg<i32>>(), service_info);
                let inner: &ArgRequestError =
                    inner.downcast_ref().expect("failed to downcast error");
                match inner {
                    ArgRequestError::MissingParameter => {}
                    inner => Err(inner).unwrap(),
                }
            }
            Err(error) => Err(error).unwrap(),
        }
    }

    #[test]
    fn request_succeeds_if_arg_is_correct_type() {
        let mut builder = Injector::builder();
        builder.provide(Foo.singleton().with_arg(42i32));

        let injector = builder.build();
        let foo = injector.get::<Svc<Foo>>().unwrap();
        assert_eq!(42, foo.value());
    }

    #[test]
    fn request_succeeds_with_interface_provider() {
        let mut builder = Injector::builder();
        builder.provide(
            Foo.singleton()
                .with_arg(42i32)
                .with_interface::<dyn Fooable>(),
        );

        let injector = builder.build();
        let foo = injector.get::<Svc<dyn Fooable>>().unwrap();
        assert_eq!(42, foo.value());
    }

    #[test]
    fn request_succeeds_with_multiple_providers() {
        let mut builder = Injector::builder();
        builder.provide(Foo.singleton().with_arg(1i32));
        builder.provide(Foo.singleton().with_arg(2i32));

        let injector = builder.build();
        let foos = injector.get::<Vec<Svc<Foo>>>().unwrap();
        assert_eq!(2, foos.len());
        assert!(foos.iter().any(|foo| foo.value() == 1));
        assert!(foos.iter().any(|foo| foo.value() == 2));
    }

    #[test]
    fn request_succeeds_with_multiple_args() {
        struct Bar(Arg<i32>, Arg<&'static str>);

        let mut builder = Injector::builder();
        builder.provide(Bar.singleton().with_arg(1i32).with_arg("foo"));

        let injector = builder.build();
        let bar = injector.get::<Svc<Bar>>().unwrap();
        assert_eq!(1, bar.0.0);
        assert_eq!("foo", bar.1.0);
    }
}
