use std::{
    any::{Any, TypeId},
    error::Error,
    fmt::{Display, Formatter},
};

#[cfg(feature = "rc")]
macro_rules! feature_unique {
    ({ $($common:tt)* }, { $($rc:tt)* }, { $($_arc:tt)* }) => {
        $($common)*
        $($rc)*
    };
}

#[cfg(feature = "arc")]
macro_rules! feature_unique {
    ({ $($common:tt)* }, { $($_rc:tt)* }, { $($arc:tt)* }) => {
        $($common)*
        $($arc)*
    };
}

feature_unique!(
    {
        /// A reference-counted pointer holding a service. The pointer type is
        /// determined by the feature flags passed to this crate.
        ///
        /// - **rc**: Pointer type is `Rc<T>`
        /// - **arc**: Pointer type is `Arc<T>`
    },
    {
        pub type Svc<T> = std::rc::Rc<T>;
    },
    {
        pub type Svc<T> = std::sync::Arc<T>;
    }
);

feature_unique!(
    {
        /// A reference-counted service pointer holding an instance of `dyn
        /// Any`.
    },
    {
        pub type DynSvc = Svc<dyn Any>;
    },
    {
        pub type DynSvc = Svc<dyn Any + Send + Sync>;
    }
);

feature_unique!(
    {
        /// Implemented automatically on types that are capable of being a
        /// service.
    },
    {
        pub trait Service: Any {}
        impl<T: ?Sized + Any> Service for T {}
    },
    {
        pub trait Service: Any + Send + Sync {}
        impl<T: ?Sized + Any + Send + Sync> Service for T {}
    }
);

/// A result from attempting to inject dependencies into a service and
/// construct an instance of it.
pub type InjectResult<T> = Result<T, InjectError>;

/// Type information about a service.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ServiceInfo {
    id: TypeId,
    name: &'static str,
}

impl ServiceInfo {
    /// Creates a [`ServiceInfo`] for the given type.
    #[must_use]
    pub fn of<T: ?Sized + Any>() -> Self {
        ServiceInfo {
            id: TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
        }
    }

    /// Gets the [`TypeId`] for this service.
    #[must_use]
    pub fn id(&self) -> TypeId {
        self.id
    }

    /// Gets the type name of this service.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// An error that has occurred during creation of a service.
#[derive(Debug)]
pub enum InjectError {
    /// Failed to find a provider for the requested type.
    MissingProvider {
        /// The service that was requested.
        service_info: ServiceInfo,
    },

    /// A provider for a dependency of the requested service is missing.
    MissingDependency {
        /// The service that was requested.
        service_info: ServiceInfo,

        /// The dependency that is missing a provider.
        dependency_info: ServiceInfo,
    },

    /// A cycle was detected during activation of a service.
    CycleDetected {
        /// The service that was requested.
        service_info: ServiceInfo,

        /// The chain of services that were requested during resolution of this
        /// service.
        cycle: Vec<ServiceInfo>,
    },

    /// The requested implementer is not valid for the requested service.
    InvalidImplementation {
        /// The service that was requested.
        service_info: ServiceInfo,

        /// The implementation that was requested for this service.
        implementation: ServiceInfo,
    },

    /// The registered provider returned the wrong service type.
    InvalidProvider {
        /// The service that was requested.
        service_info: ServiceInfo,
    },

    /// The requested service has too many providers registered.
    MultipleProviders {
        /// The service that was requested.
        service_info: ServiceInfo,
        /// The number of providers registered for that service.
        providers: usize,
    },

    /// An error occurred during activation of a service.
    ActivationFailed {
        /// The service that was requested.
        service_info: ServiceInfo,
        /// The error that was thrown during service initialization.
        inner: Box<dyn Error + 'static>,
    },

    /// An unexpected error has occurred. This is usually caused by a bug in
    /// the library itself.
    InternalError(String),
}

impl Error for InjectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InjectError::ActivationFailed { inner, .. } => Some(inner.as_ref()),
            _ => None,
        }
    }
}

impl Display for InjectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "an error occurred during injection: ")?;
        match self {
            InjectError::MissingProvider { service_info } => {
                write!(f, "{} has no provider", service_info.name())?
            }
            InjectError::MissingDependency {
                service_info,
                ..
            } => write!(f, "{} is missing a dependency", service_info.name())?,
            InjectError::CycleDetected {
                service_info,
                cycle,
            } => write!(
                f,
                "a cycle was detected during activation of {} [{}]",
                service_info.name(),
                fmt_cycle(cycle)
            )?,
            InjectError::InvalidImplementation {
                service_info,
                implementation,
            } => write!(
                f,
                "{} is not registered as an implementer of {}",
                implementation.name(),
                service_info.name()
            )?,
            InjectError::InvalidProvider { service_info } => {
                write!(f, "the registered provider for {} returned the wrong type", service_info.name())?
            }
            InjectError::MultipleProviders {
                service_info,
                providers,
            } => write!(
                f,
                "the requested service {} has {} providers registered (did you mean to request a Services<T> instead?)",
                service_info.name(),
                providers
            )?,
            InjectError::ActivationFailed { service_info, .. } => {
                write!(f, "an error occurred during activation of {}", service_info.name())?
            },
            InjectError::InternalError(message) => {
                write!(f, "an unexpected error occurred (please report this): {}", message)?
            },
        };

        Ok(())
    }
}

fn fmt_cycle(cycle: &[ServiceInfo]) -> String {
    let mut joined = String::new();
    for item in cycle.iter().rev() {
        if !joined.is_empty() {
            joined.push_str(" -> ");
        }
        joined.push_str(item.name());
    }
    joined
}
