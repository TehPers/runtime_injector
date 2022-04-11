use crate::{
    InjectError, InjectResult, Interface, Provider, Service, ServiceInfo,
};
use std::collections::HashMap;

/// Stores providers for a particular interface
#[derive(Default)]
pub(crate) struct ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    providers:
        HashMap<ServiceInfo, Option<Vec<Box<dyn Provider<Interface = I>>>>>,
}

impl<I> ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    pub fn new(
        providers: HashMap<
            ServiceInfo,
            Option<Vec<Box<dyn Provider<Interface = I>>>>,
        >,
    ) -> Self {
        ProviderRegistry { providers }
    }

    /// Gets the providers for a particular service type.
    pub fn take_providers_for(
        &mut self,
        service_info: ServiceInfo,
    ) -> InjectResult<Vec<Box<dyn Provider<Interface = I>>>> {
        // Get the provider list slot
        let slot = self
            .providers
            .get_mut(&service_info)
            .ok_or_else(|| InjectError::MissingProvider { service_info })?;

        // Ensure the providers are not in use
        slot.take().ok_or_else(|| InjectError::CycleDetected {
            service_info,
            cycle: vec![service_info],
        })
    }

    /// Reclaims the providers for a particular service type.
    pub fn reclaim_providers_for(
        &mut self,
        service_info: ServiceInfo,
        providers: Vec<Box<dyn Provider<Interface = I>>>,
    ) -> InjectResult<()> {
        // Get the provider list slot
        let slot = self.providers.get_mut(&service_info).ok_or_else(|| {
            InjectError::InternalError(format!(
                "activated provider for {} is no longer registered",
                service_info.name()
            ))
        })?;

        // Insert the providers back into the list, ensuring the list is in use
        if slot.replace(providers).is_some() {
            Err(InjectError::InternalError(format!(
                "another provider for {} was added during its activation",
                service_info.name()
            )))
        } else {
            Ok(())
        }
    }
}

/// Marker trait for provider registries.
pub(crate) trait ProviderRegistryType: Service {}
impl<I> ProviderRegistryType for ProviderRegistry<I> where I: ?Sized + Interface {}

#[cfg(feature = "arc")]
downcast_rs::impl_downcast!(sync ProviderRegistryType);

#[cfg(feature = "rc")]
downcast_rs::impl_downcast!(ProviderRegistryType);

#[derive(Default)]
pub(crate) struct InterfaceRegistry {
    provider_registries: HashMap<ServiceInfo, Box<dyn ProviderRegistryType>>,
}

impl InterfaceRegistry {
    pub fn new(
        provider_registries: HashMap<
            ServiceInfo,
            Box<dyn ProviderRegistryType>,
        >,
    ) -> Self {
        InterfaceRegistry {
            provider_registries,
        }
    }

    pub fn take_providers_for<I>(
        &mut self,
        service_info: ServiceInfo,
    ) -> InjectResult<Vec<Box<dyn Provider<Interface = I>>>>
    where
        I: ?Sized + Interface,
    {
        // Get provider registry
        let interface_info = ServiceInfo::of::<I>();
        let provider_registry = self
            .provider_registries
            .get_mut(&interface_info)
            .ok_or_else(|| InjectError::MissingProvider { service_info })?;

        // Downcast provider list
        let provider_registry: &mut ProviderRegistry<I> = provider_registry
            .downcast_mut()
            .ok_or_else(|| InjectError::InvalidProvider {
                service_info: { interface_info },
            })?;

        // Get providers
        provider_registry.take_providers_for(service_info)
    }

    pub fn reclaim_providers_for<I>(
        &mut self,
        service_info: ServiceInfo,
        providers: Vec<Box<dyn Provider<Interface = I>>>,
    ) -> InjectResult<()>
    where
        I: ?Sized + Interface,
    {
        // Get the provider registry
        let interface_info = ServiceInfo::of::<I>();
        let provider_registry = self
            .provider_registries
            .get_mut(&interface_info)
            .ok_or_else(|| {
                InjectError::InternalError(format!(
                    "activated provider for {} is no longer registered",
                    interface_info.name()
                ))
            })?;

        // Downcast the provider registry
        let provider_registry: &mut ProviderRegistry<_> =
            provider_registry.downcast_mut().ok_or_else(|| {
                InjectError::InternalError(format!(
                    "provider for {} is the wrong type",
                    interface_info.name()
                ))
            })?;

        // Reclaim the providers
        provider_registry.reclaim_providers_for(service_info, providers)
    }
}
