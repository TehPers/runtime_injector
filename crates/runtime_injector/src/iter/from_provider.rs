use crate::{
    InjectError, InjectResult, Interface, Provider, Service, ServiceInfo, Svc,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ServiceType {
    Service,
    Interface,
}

pub trait FromProvider: Service {
    type Interface: ?Sized + Interface;

    const SERVICE_TYPE: ServiceType;

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

    const SERVICE_TYPE: ServiceType = ServiceType::Service;

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
        let provided = service.downcast_rc().map_err(|_| {
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
