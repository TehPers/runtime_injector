use crate::{
    provider_registry::{ProviderRegistry, ProviderRegistryType},
    Injector, Interface, InterfaceRegistry, Module, Provider, RequestInfo,
    Service, ServiceInfo, Svc,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};

/// A builder for an [`Injector`].
#[derive(Debug, Default)]
pub struct InjectorBuilder {
    registry: InterfaceRegistryBuilder,
    root_info: RequestInfo,
}

impl InjectorBuilder {
    /// Assigns the provider for a service type. Multiple providers can be
    /// registered for a service.
    pub fn provide<P>(&mut self, provider: P)
    where
        P: Provider,
    {
        self.add_provider(Svc::new(provider));
    }

    /// Adds a provider to the injector.
    pub fn add_provider<I>(
        &mut self,
        provider: Svc<dyn Provider<Interface = I>>,
    ) where
        I: ?Sized + Interface,
    {
        self.registry
            .ensure_providers_mut()
            .add_provider_for(provider.result(), provider);
    }

    /// Removes all providers for a service type.
    pub fn remove_providers(
        &mut self,
        service_info: ServiceInfo,
    ) -> Vec<Svc<dyn Provider<Interface = dyn Service>>> {
        self.remove_providers_for::<dyn Service>(service_info)
    }

    /// Removes all providers for a service type from an interface.
    pub fn remove_providers_for<I>(
        &mut self,
        service_info: ServiceInfo,
    ) -> Vec<Svc<dyn Provider<Interface = I>>>
    where
        I: ?Sized + Interface,
    {
        self.registry
            .remove_providers_for::<I>(service_info)
            .unwrap_or_default()
    }

    /// Clears all providers.
    pub fn clear_providers(&mut self) {
        self.registry.clear();
    }

    /// Clears all providers for an interface.
    pub fn clear_providers_for<I>(&mut self)
    where
        I: ?Sized + Interface,
    {
        self.registry.remove_providers::<I>();
    }

    /// Borrows the root [`RequestInfo`] that will be used by calls to
    /// [`Injector::get()`].
    #[must_use]
    pub fn root_info(&self) -> &RequestInfo {
        &self.root_info
    }

    /// Mutably borrows the root [`RequestInfo`] that will be used by calls to
    /// [`Injector::get()`].
    #[must_use]
    pub fn root_info_mut(&mut self) -> &mut RequestInfo {
        &mut self.root_info
    }

    /// Adds all the providers registered in a module. This may cause multiple
    /// providers to be registered for the same service.
    ///
    /// If any conflicting request parameters have been set before adding this
    /// module, they are overridden.
    #[allow(clippy::missing_panics_doc)]
    pub fn add_module(&mut self, module: Module) {
        // Merge providers
        self.registry.merge(module.registry);

        // Merge parameters
        for (key, value) in module.parameters {
            self.root_info_mut().insert_parameter_boxed(&key, value);
        }
    }

    /// Builds the injector.
    #[must_use]
    pub fn build(self) -> Injector {
        Injector::new_from_parts(self.registry.build(), self.root_info)
    }
}

#[derive(Debug, Default)]
pub(crate) struct InterfaceRegistryBuilder {
    registries: HashMap<ServiceInfo, Box<dyn ProviderRegistryType>>,
}

impl InterfaceRegistryBuilder {
    pub fn ensure_providers_mut<I>(&mut self) -> &mut ProviderRegistry<I>
    where
        I: ?Sized + Interface,
    {
        self.registries
            .entry(ServiceInfo::of::<I>())
            .or_insert_with(|| Box::new(ProviderRegistry::<I>::default()))
            .downcast_mut()
            .unwrap()
    }

    pub fn remove_providers<I>(&mut self) -> Option<ProviderRegistry<I>>
    where
        I: ?Sized + Interface,
    {
        let interface_info = ServiceInfo::of::<I>();
        let registry = self.registries.remove(&interface_info)?;
        let registry = registry.downcast().unwrap();
        Some(*registry)
    }

    pub fn remove_providers_for<I>(
        &mut self,
        service_info: ServiceInfo,
    ) -> Option<Vec<Svc<dyn Provider<Interface = I>>>>
    where
        I: ?Sized + Interface,
    {
        let registry = self.registries.get_mut(&service_info)?;
        let registry: &mut ProviderRegistry<I> =
            registry.downcast_mut().unwrap();
        registry.remove_providers_for(service_info)
    }

    pub fn clear(&mut self) {
        self.registries.clear();
    }

    pub fn merge(&mut self, other: InterfaceRegistryBuilder) {
        for (interface_info, other_providers) in other.registries {
            match self.registries.entry(interface_info) {
                Entry::Occupied(entry) => {
                    entry.into_mut().merge(other_providers).unwrap();
                }
                Entry::Vacant(entry) => {
                    entry.insert(other_providers);
                }
            }
        }
    }

    pub fn build(self) -> InterfaceRegistry {
        let registries = self
            .registries
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        InterfaceRegistry::new(registries)
    }
}
