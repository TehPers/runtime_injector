use crate::{Interface, Provider, Service, ServiceInfo, Svc};
use std::{
    collections::{hash_map::Values, HashMap},
    fmt::{Debug, Formatter},
    slice::Iter,
};

/// Stores providers for a particular interface
pub(crate) struct ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    providers: HashMap<ServiceInfo, Vec<Svc<dyn Provider<Interface = I>>>>,
}

impl<I> ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    pub fn add_provider_for(
        &mut self,
        service_info: ServiceInfo,
        provider: Svc<dyn Provider<Interface = I>>,
    ) {
        self.providers
            .entry(service_info)
            .or_default()
            .push(provider);
    }

    pub fn remove_providers_for(
        &mut self,
        service_info: ServiceInfo,
    ) -> Option<Vec<Svc<dyn Provider<Interface = I>>>> {
        self.providers.remove(&service_info)
    }

    pub fn iter(&self) -> ProviderRegistryIter<'_, I> {
        ProviderRegistryIter {
            values: self.providers.values(),
            cur_slot: None,
        }
    }
}

impl<I> Debug for ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.providers.iter().filter(|(_, v)| !v.is_empty()).map(
                    |(k, v)| (k.name(), format!("<{} providers>", v.len())),
                ),
            )
            .finish()
    }
}

impl<I> Default for ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    fn default() -> Self {
        Self {
            providers: Default::default(),
        }
    }
}

pub(crate) struct ProviderRegistryIter<'a, I>
where
    I: ?Sized + Interface,
{
    values: Values<'a, ServiceInfo, Vec<Svc<dyn Provider<Interface = I>>>>,
    cur_slot: Option<Iter<'a, Svc<dyn Provider<Interface = I>>>>,
}

impl<'a, I> Iterator for ProviderRegistryIter<'a, I>
where
    I: ?Sized + Interface,
{
    type Item = &'a dyn Provider<Interface = I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get next item in current slot
            if let Some(next) = self.cur_slot.as_mut().and_then(Iterator::next)
            {
                return Some(next.as_ref());
            }

            // Try to go to next slot
            let next_slot = self.values.next()?;
            self.cur_slot = Some(next_slot.iter());
        }
    }
}

/// Marker trait for provider registries.
pub(crate) trait ProviderRegistryType: Service + Debug {
    fn merge(
        &mut self,
        other: Box<dyn ProviderRegistryType>,
    ) -> Result<(), Box<dyn ProviderRegistryType>>;
}

impl<I> ProviderRegistryType for ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    fn merge(
        &mut self,
        other: Box<dyn ProviderRegistryType>,
    ) -> Result<(), Box<dyn ProviderRegistryType>> {
        let other: Box<Self> = other.downcast()?;
        for (service_info, mut other_providers) in other.providers {
            self.providers
                .entry(service_info)
                .or_default()
                .append(&mut other_providers);
        }

        Ok(())
    }
}

#[cfg(feature = "arc")]
downcast_rs::impl_downcast!(sync ProviderRegistryType);

#[cfg(feature = "rc")]
downcast_rs::impl_downcast!(ProviderRegistryType);

#[derive(Debug, Default)]
pub(crate) struct InterfaceRegistry {
    registries: HashMap<ServiceInfo, Svc<dyn ProviderRegistryType>>,
}

impl InterfaceRegistry {
    pub fn new(
        provider_registries: HashMap<
            ServiceInfo,
            Svc<dyn ProviderRegistryType>,
        >,
    ) -> Self {
        InterfaceRegistry {
            registries: provider_registries,
        }
    }

    pub fn get_providers<I>(&self) -> Svc<ProviderRegistry<I>>
    where
        I: ?Sized + Interface,
    {
        // If a provider registry for the given interface is not found, an
        // empty one is returned. This allows requests for interfaces that have
        // no providers registered to still work without returning an error.
        let interface_info = ServiceInfo::of::<I>();
        let registry = self.registries.get(&interface_info).cloned();
        #[cfg(feature = "arc")]
        let registry = registry.map(|registry| {
            registry.downcast_arc::<ProviderRegistry<I>>().unwrap()
        });
        #[cfg(feature = "rc")]
        let registry = registry.map(|registry| {
            registry.downcast_rc::<ProviderRegistry<I>>().unwrap()
        });
        registry.unwrap_or_default()
    }
}
