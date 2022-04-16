use crate::{
    InjectError, InjectResult, Interface, Provider, Service, ServiceInfo, Svc,
};

/// The type of provider that should be requested.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ProviderType {
    /// Providers for implementations of services should be requested.
    Service,
    /// Providers for implementations of an interface should be requested.
    Interface,
}

/// A type that can be requested from a provider.
pub trait FromProvider: Service {
    /// The interface to request providers for.
    type Interface: ?Sized + Interface;

    /// The type of providers to request. Providers can either provide
    /// implementations of a service type or of an interface type.
    const PROVIDER_TYPE: ProviderType;

    fn should_provide(
        provider: &dyn Provider<Interface = Self::Interface>,
    ) -> bool;

    fn from_provided(provided: Svc<Self::Interface>)
        -> InjectResult<Svc<Self>>;

    fn from_provided_owned(
        provided: Box<Self::Interface>,
    ) -> InjectResult<Box<Self>>;
}

impl<S: Service> FromProvider for S {
    type Interface = dyn Service;

    const PROVIDER_TYPE: ProviderType = ProviderType::Service;

    #[inline]
    fn should_provide(
        provider: &dyn Provider<Interface = Self::Interface>,
    ) -> bool {
        provider.result() == ServiceInfo::of::<Self>()
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
