use crate::{
    AsAny, InjectError, InjectResult, Injector, InjectorBuilder, Module,
    Request, RequestInfo, RequestParameter, Service, ServiceInfo,
};
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    ops::{Deref, DerefMut},
};

/// Allows custom pre-defined values to be passed as arguments to services.
///
/// ## Example
///
/// ```
/// use runtime_injector::{Arg, Injector, IntoTransient, WithArg};
///
/// struct Foo(Arg<i32>);
///
/// let mut builder = Injector::builder();
/// builder.provide(Foo.transient());
/// builder.with_arg::<Foo, i32>(12);
///
/// let injector = builder.build();
/// let foo: Box<Foo> = injector.get().unwrap();
/// assert_eq!(12, *foo.0);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Arg<T: Service + AsAny + Clone>(T);

impl<T: Service + AsAny + Clone> Arg<T> {
    pub(crate) fn param_name(target: ServiceInfo) -> String {
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

impl<T: Service + AsAny + Clone> Deref for Arg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Service + AsAny + Clone> DerefMut for Arg<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display + Service + AsAny + Clone> Display for Arg<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Allows custom pre-defined values to be passed as arguments to services.
impl<T: Service + AsAny + Clone> Request for Arg<T> {
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

/// Allows defining pre-defined arguments to services.
pub trait WithArg {
    /// Adds an argument for a service. See the docs for [`Arg<T>`].
    fn with_arg<S: Service, T: Service + AsAny + Clone>(
        &mut self,
        value: T,
    ) -> Option<Box<dyn RequestParameter>>;
}

impl WithArg for RequestInfo {
    fn with_arg<S: Service, T: Service + AsAny + Clone>(
        &mut self,
        value: T,
    ) -> Option<Box<dyn RequestParameter>> {
        self.insert_parameter(
            &Arg::<T>::param_name(ServiceInfo::of::<S>()),
            value,
        )
    }
}

impl WithArg for InjectorBuilder {
    fn with_arg<S: Service, T: Service + AsAny + Clone>(
        &mut self,
        value: T,
    ) -> Option<Box<dyn RequestParameter>> {
        self.root_info_mut().with_arg::<S, T>(value)
    }
}

impl WithArg for Module {
    fn with_arg<S: Service, T: Service + AsAny + Clone>(
        &mut self,
        value: T,
    ) -> Option<Box<dyn RequestParameter>> {
        self.insert_parameter(
            &Arg::<T>::param_name(ServiceInfo::of::<S>()),
            value,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        define_module, Arg, ArgRequestError, InjectError, Injector,
        IntoSingleton, ServiceInfo, Svc,
    };

    #[derive(Debug)]
    struct Foo(Arg<i32>);

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
                assert_eq!(
                    ServiceInfo::of::<Arg<i32>>(),
                    service_info,
                );
                
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
        struct Foo(Arg<i32>);

        let module = define_module! {
            services = [Foo.singleton()],
            arguments = {
                Foo = [42u32],
            },
        };

        let mut builder = Injector::builder();
        builder.add_module(module);

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
        struct Foo(Arg<i32>);

        let module = define_module! {
            services = [Foo.singleton()],
            arguments = {
                Foo = [42i32],
            },
        };

        let mut builder = Injector::builder();
        builder.add_module(module);

        let injector = builder.build();
        let foo = injector.get::<Svc<Foo>>().unwrap();
        assert_eq!(42, foo.0.0);
    }
}
