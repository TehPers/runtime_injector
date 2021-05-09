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
/// # Example
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
pub struct Arg<T: Service + AsAny + Clone + Debug>(T);

impl<T: Service + AsAny + Clone + Debug> Arg<T> {
    pub(crate) fn param_name(target: ServiceInfo) -> String {
        format!(
            "runtime_injector::Arg[target={:?},type={:?}]",
            target.name(),
            ServiceInfo::of::<T>().name()
        )
    }

    /// Converts an argument into its inner value.
    pub fn into_inner(arg: Self) -> T {
        arg.0
    }
}

impl<T: Service + AsAny + Clone + Debug> Deref for Arg<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Service + AsAny + Clone + Debug> DerefMut for Arg<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Service + AsAny + Clone + Debug> Request for Arg<T> {
    fn request(_injector: &Injector, info: RequestInfo) -> InjectResult<Self> {
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
                inner: Box::new(ArgRequestError::NoParentRequest),
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
pub enum ArgRequestError {
    /// The argument value was not provided.
    MissingParameter,
    /// The argument value is the wrong type.
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
    fn with_arg<S: Service, T: Service + AsAny + Clone + Debug>(
        &mut self,
        value: T,
    ) -> Option<Box<dyn RequestParameter>>;
}

impl WithArg for RequestInfo {
    fn with_arg<S: Service, T: Service + AsAny + Clone + Debug>(
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
    fn with_arg<S: Service, T: Service + AsAny + Clone + Debug>(
        &mut self,
        value: T,
    ) -> Option<Box<dyn RequestParameter>> {
        self.root_info_mut().with_arg::<S, T>(value)
    }
}

impl WithArg for Module {
    fn with_arg<S: Service, T: Service + AsAny + Clone + Debug>(
        &mut self,
        value: T,
    ) -> Option<Box<dyn RequestParameter>> {
        self.insert_parameter(
            &Arg::<T>::param_name(ServiceInfo::of::<S>()),
            value,
        )
    }
}
