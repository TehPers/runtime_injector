use crate::{
    provider_registry::{ProviderRegistry, ProviderRegistryType},
    Injector, Interface, InterfaceRegistry, Module, Provider, RequestInfo,
    Service, ServiceInfo,
};
use downcast_rs::impl_downcast;
use std::collections::{hash_map::Entry, HashMap};

/// A builder for an [`Injector`].
#[derive(Default)]
pub struct InjectorBuilder {
    registry_builder: InterfaceRegistryBuilder,
    root_info: RequestInfo,
}

impl InjectorBuilder {
    /// Assigns the provider for a service type. Multiple providers can be
    /// registered for a service.
    pub fn provide<P>(&mut self, provider: P)
    where
        P: Provider,
    {
        self.add_provider(Box::new(provider))
    }

    /// Adds a provider to the injector.
    #[allow(clippy::missing_panics_doc)]
    pub fn add_provider<I>(
        &mut self,
        provider: Box<dyn Provider<Interface = I>>,
    ) where
        I: ?Sized + Interface,
    {
        self.registry_builder
            .ensure_providers_mut()
            .add_provider_for(provider.result(), provider);
    }

    /// Removes all providers for a service type.
    pub fn remove_providers(
        &mut self,
        service_info: ServiceInfo,
    ) -> Vec<Box<dyn Provider<Interface = dyn Service>>> {
        self.remove_providers_for::<dyn Service>(service_info)
    }

    /// Removes all providers for a service type from an interface.
    pub fn remove_providers_for<I>(
        &mut self,
        service_info: ServiceInfo,
    ) -> Vec<Box<dyn Provider<Interface = I>>>
    where
        I: ?Sized + Interface,
    {
        self.registry_builder
            .providers_mut::<I>()
            .map(|providers| providers.remove_providers_for(service_info))
            .unwrap_or_default()
    }

    /// Clears all providers.
    pub fn clear_providers(&mut self) {
        self.registry_builder.clear();
    }

    /// Clears all providers for an interface.
    pub fn clear_providers_for<I>(&mut self)
    where
        I: ?Sized + Interface,
    {
        self.registry_builder
            .providers
            .remove(&ServiceInfo::of::<I>());
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
        self.registry_builder.merge(module.registry_builder);

        // Merge parameters
        for (key, value) in module.parameters {
            self.root_info_mut().insert_parameter_boxed(&key, value);
        }
    }

    /// Builds the injector.
    #[must_use]
    pub fn build(self) -> Injector {
        Injector::new_from_parts(self.registry_builder.build(), self.root_info)
    }
}

pub(crate) struct ProviderRegistryBuilder<I>
where
    I: ?Sized + Interface,
{
    providers:
        HashMap<ServiceInfo, Option<Vec<Box<dyn Provider<Interface = I>>>>>,
}

impl<I> ProviderRegistryBuilder<I>
where
    I: ?Sized + Interface,
{
    pub fn add_provider_for(
        &mut self,
        service_info: ServiceInfo,
        provider: Box<dyn Provider<Interface = I>>,
    ) {
        #[allow(clippy::missing_panics_doc)]
        self.providers
            .entry(service_info)
            .or_insert_with(|| Some(Vec::new()))
            .as_mut()
            .unwrap()
            .push(provider);
    }

    pub fn remove_providers_for(
        &mut self,
        service_info: ServiceInfo,
    ) -> Vec<Box<dyn Provider<Interface = I>>> {
        #[allow(clippy::missing_panics_doc)]
        self.providers
            .remove(&service_info)
            .map(Option::unwrap)
            .unwrap_or_default()
    }
}

impl<I> Default for ProviderRegistryBuilder<I>
where
    I: ?Sized + Interface,
{
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
}

pub(crate) trait ProviderRegistryBuilderType: Service {
    fn merge(
        &mut self,
        other: Box<dyn ProviderRegistryBuilderType>,
    ) -> Result<(), Box<dyn ProviderRegistryBuilderType>>;

    fn build(&mut self) -> Box<dyn ProviderRegistryType>;
}

impl<I> ProviderRegistryBuilderType for ProviderRegistryBuilder<I>
where
    I: ?Sized + Interface,
{
    fn merge(
        &mut self,
        other: Box<dyn ProviderRegistryBuilderType>,
    ) -> Result<(), Box<dyn ProviderRegistryBuilderType>> {
        let other: Box<Self> = other.downcast()?;
        for (service_info, other_providers) in other.providers {
            #[allow(clippy::missing_panics_doc)]
            let mut other_providers = other_providers.unwrap();
            self.providers
                .entry(service_info)
                .or_insert_with(|| Some(Vec::new()))
                .as_mut()
                .unwrap()
                .append(&mut other_providers);
        }

        Ok(())
    }

    fn build(&mut self) -> Box<dyn ProviderRegistryType> {
        Box::new(ProviderRegistry::new(std::mem::take(&mut self.providers)))
    }
}

#[cfg(feature = "arc")]
impl_downcast!(sync ProviderRegistryBuilderType);

#[cfg(feature = "rc")]
impl_downcast!(ProviderRegistryBuilderType);

#[derive(Default)]
pub(crate) struct InterfaceRegistryBuilder {
    providers: HashMap<ServiceInfo, Box<dyn ProviderRegistryBuilderType>>,
}

impl InterfaceRegistryBuilder {
    pub fn providers_mut<I>(
        &mut self,
    ) -> Option<&mut ProviderRegistryBuilder<I>>
    where
        I: ?Sized + Interface,
    {
        #[allow(clippy::missing_panics_doc)]
        self.providers
            .get_mut(&ServiceInfo::of::<I>())
            .map(|providers| {
                providers
                    .downcast_mut::<ProviderRegistryBuilder<I>>()
                    .unwrap()
            })
    }

    pub fn ensure_providers_mut<I>(&mut self) -> &mut ProviderRegistryBuilder<I>
    where
        I: ?Sized + Interface,
    {
        #[allow(clippy::missing_panics_doc)]
        self.providers
            .entry(ServiceInfo::of::<I>())
            .or_insert_with(
                || Box::new(ProviderRegistryBuilder::<I>::default()),
            )
            .downcast_mut()
            .unwrap()
    }

    pub fn clear(&mut self) {
        self.providers.clear();
    }

    pub fn merge(&mut self, other: InterfaceRegistryBuilder) {
        for (service_info, other_providers) in other.providers {
            match self.providers.entry(service_info) {
                Entry::Occupied(entry) => {
                    #[allow(clippy::missing_panics_doc)]
                    entry
                        .into_mut()
                        .merge(other_providers)
                        .map_err(|_| "error merging provider builders")
                        .unwrap();
                }
                Entry::Vacant(entry) => {
                    entry.insert(other_providers);
                }
            }
        }
    }

    pub fn build(self) -> InterfaceRegistry {
        let providers = self
            .providers
            .into_iter()
            .map(|(service_info, mut providers)| {
                (service_info, providers.build())
            })
            .collect();
        InterfaceRegistry::new(providers)
    }
}
