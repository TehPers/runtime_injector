use crate::{
    InjectError, InjectResult, Interface, Provider, Service, ServiceInfo, Svc,
};

/// A type that can be requested from a provider.
pub trait FromProvider: Service {
    /// The interface to request providers for.
    type Interface: ?Sized + Interface;

    /// Whether the given provider is valid for this type.
    fn should_provide(
        provider: &dyn Provider<Interface = Self::Interface>,
    ) -> bool;

    /// Converts a provided service into a service pointer of this type.
    fn from_provided(provided: Svc<Self::Interface>)
        -> InjectResult<Svc<Self>>;

    /// Converts a provided service into an owned service pointer of this type.
    fn from_provided_owned(
        provided: Box<Self::Interface>,
    ) -> InjectResult<Box<Self>>;
}

impl<S: Service> FromProvider for S {
    type Interface = dyn Service;

    fn should_provide(
        provider: &dyn Provider<Interface = Self::Interface>,
    ) -> bool {
        provider.result() == ServiceInfo::of::<S>()
    }

    #[inline]
    fn from_provided(
        provided: Svc<Self::Interface>,
    ) -> InjectResult<Svc<Self>> {
        #[cfg(feature = "arc")]
        let provided = provided.downcast_arc().map_err(|_| {
            InjectError::InvalidProvider {
                service_info: ServiceInfo::of::<Self>(),
            }
        })?;
        #[cfg(feature = "rc")]
        let provided = provided.downcast_rc().map_err(|_| {
            InjectError::InvalidProvider {
                service_info: ServiceInfo::of::<Self>(),
            }
        })?;
        Ok(provided)
    }

    #[inline]
    fn from_provided_owned(
        provided: Box<Self::Interface>,
    ) -> InjectResult<Box<Self>> {
        provided
            .downcast()
            .map_err(|_| InjectError::InvalidProvider {
                service_info: ServiceInfo::of::<Self>(),
            })
    }
}
