#![allow(clippy::used_underscore_binding)]

use derive_more::{Display, Error};
use std::any::{Any, TypeId};

#[cfg(feature = "arc")]
mod types {
    use crate::InjectError;
    use std::{any::Any, sync::Arc};

    /// A reference-counted pointer holding a service. The pointer type is
    /// determined by the feature flags passed to this crate.
    pub type Svc<T> = Arc<T>;

    /// A reference-counted service pointer holding an instance of `dyn Any`.
    pub type DynSvc = Arc<dyn Any + Send + Sync>;

    /// A result from attempting to inject dependencies into a service and
    /// construct an instance of it.
    pub type InjectResult<T> = Result<T, InjectError>;

    /// Implemented automatically on types that are capable of being a service.
    pub trait Service: Any + Send + Sync {}
    impl<T: ?Sized + Any + Send + Sync> Service for T {}
}

#[cfg(feature = "rc")]
mod types {
    use crate::InjectError;
    use std::{any::Any, rc::Rc};

    /// A reference-counted pointer holding a service. The pointer type is
    /// determined by the feature flags passed to this crate.
    pub type Svc<T> = Rc<T>;

    /// A reference-counted service pointer holding an instance of `dyn Any`.
    pub type DynSvc = Rc<dyn Any>;

    /// A result from attempting to inject dependencies into a service and
    /// construct an instance of it.
    pub type InjectResult<T> = Result<T, InjectError>;

    /// Implemented automatically on types that are capable of being a service.
    pub trait Service: Any {}
    impl<T: ?Sized + Any> Service for T {}
}

pub use types::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ServiceInfo {
    id: TypeId,
    name: &'static str,
}

impl ServiceInfo {
    #[must_use]
    pub fn of<T: ?Sized + Any>() -> Self {
        ServiceInfo {
            id: TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
        }
    }

    #[must_use]
    pub fn id(&self) -> TypeId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug, Display, Error)]
#[display(fmt = "an error occurred during injection: {}")]
pub enum InjectError {
    /// Failed to find a provider for the requested type.
    #[display(fmt = "{} has no provider", "service_info.name()")]
    MissingProvider { service_info: ServiceInfo },

    /// A provider for a dependency of the requested service is missing.
    #[display(fmt = "{} is missing a dependency", "service_info.name()")]
    MissingDependency {
        service_info: ServiceInfo,
        dependency_info: ServiceInfo,
    },

    /// A cycle was detected during activation of a service.
    #[display(
        fmt = "a cycle was detected during activation of {} [{}]",
        "service_info.name()",
        "fmt_cycle(cycle)"
    )]
    CycleDetected {
        service_info: ServiceInfo,
        cycle: Vec<ServiceInfo>,
    },

    /// The requested implementer is not valid for the requested service.
    #[display(
        fmt = "{} is not registered as an implementer of {}",
        "implementation.name()",
        "service_info.name()"
    )]
    InvalidImplementation {
        service_info: ServiceInfo,
        implementation: ServiceInfo,
    },

    #[display(fmt = "the registered provider returned the wrong type")]
    InvalidProvider { service_info: ServiceInfo },

    /// An unexpected error has occurred. This is usually caused by a bug in
    /// the library itself.
    #[display(
        fmt = "an unexpected error occurred (please report this): {}",
        _0
    )]
    InternalError(#[error(ignore)] String),
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
