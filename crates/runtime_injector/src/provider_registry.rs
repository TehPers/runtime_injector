use crate::{
    InjectError, InjectResult, Interface, Provider, Service, ServiceInfo,
};
use std::{
    collections::{hash_map::ValuesMut, HashMap},
    fmt::{Debug, Formatter},
    slice::IterMut,
};

pub(crate) struct Slot<T>(Option<T>);

impl<T> Slot<T> {
    pub fn take(&mut self) -> Option<T> {
        self.0.take()
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        self.0.replace(value)
    }

    pub fn inner(&self) -> Option<&T> {
        self.0.as_ref()
    }

    pub fn inner_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut()
    }

    pub fn with_inner_mut<R, F>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.0.as_mut().map(f)
    }

    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T> Default for Slot<T>
where
    T: Default,
{
    fn default() -> Self {
        Self(Some(Default::default()))
    }
}

impl<T> From<T> for Slot<T> {
    fn from(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T> From<Option<T>> for Slot<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

pub(crate) type ProviderSlot<I> = Slot<Vec<Box<dyn Provider<Interface = I>>>>;

impl<I> Debug for ProviderSlot<I>
where
    I: ?Sized + Interface,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.inner() {
            Some(providers) => f
                .debug_tuple("Slot")
                .field(&format_args!("<{} provider(s)>", providers.len()))
                .finish(),
            None => f.debug_tuple("Slot").field(&"<providers in use>").finish(),
        }
    }
}

impl Debug for Slot<Box<dyn ProviderRegistryType>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Slot").field(&self.0).finish()
    }
}

/// Stores providers for a particular interface
pub(crate) struct ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    providers: HashMap<ServiceInfo, ProviderSlot<I>>,
}

impl<I> ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    pub fn new(providers: HashMap<ServiceInfo, ProviderSlot<I>>) -> Self {
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

    pub fn iter_mut(&mut self) -> ProviderRegistryIterMut<'_, I> {
        ProviderRegistryIterMut {
            values: self.providers.values_mut(),
            cur_slot: None,
        }
    }
}

impl<I> Debug for ProviderRegistry<I>
where
    I: ?Sized + Interface,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("providers", &self.providers)
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

pub(crate) struct ProviderRegistryIterMut<'a, I>
where
    I: ?Sized + Interface,
{
    values: ValuesMut<'a, ServiceInfo, ProviderSlot<I>>,
    cur_slot: Option<IterMut<'a, Box<dyn Provider<Interface = I>>>>,
}

impl<'a, I> Iterator for ProviderRegistryIterMut<'a, I>
where
    I: ?Sized + Interface,
{
    type Item = InjectResult<&'a mut dyn Provider<Interface = I>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get next item in current slot
            if let Some(next) = self.cur_slot.as_mut().and_then(Iterator::next)
            {
                return Some(Ok(next.as_mut()));
            }

            // Try to go to next slot
            let slot = self.values.next()?;
            let providers = match slot.inner_mut() {
                Some(providers) => providers,
                None => {
                    return Some(Err(InjectError::InternalError(format!(
                        "providers for {:?} are in use",
                        ServiceInfo::of::<I>().name()
                    ))))
                }
            };
            self.cur_slot = Some(providers.iter_mut());
        }
    }
}

/// Marker trait for provider registries.
pub(crate) trait ProviderRegistryType: Service + Debug {}
impl<I> ProviderRegistryType for ProviderRegistry<I> where I: ?Sized + Interface {}

#[cfg(feature = "arc")]
downcast_rs::impl_downcast!(sync ProviderRegistryType);

#[cfg(feature = "rc")]
downcast_rs::impl_downcast!(ProviderRegistryType);

#[derive(Debug, Default)]
pub(crate) struct InterfaceRegistry {
    provider_registries:
        HashMap<ServiceInfo, Slot<Box<dyn ProviderRegistryType>>>,
}

impl InterfaceRegistry {
    pub fn new(
        provider_registries: HashMap<
            ServiceInfo,
            Slot<Box<dyn ProviderRegistryType>>,
        >,
    ) -> Self {
        InterfaceRegistry {
            provider_registries,
        }
    }

    pub fn take<I>(&mut self) -> InjectResult<ProviderRegistry<I>>
    where
        I: ?Sized + Interface,
    {
        let interface_info = ServiceInfo::of::<I>();
        self.provider_registries
            .get_mut(&interface_info)
            .ok_or_else(|| InjectError::MissingProvider {
                service_info: interface_info,
            })?
            .take()
            .ok_or_else(|| InjectError::CycleDetected {
                service_info: interface_info,
                cycle: vec![interface_info],
            })?
            .downcast()
            .map_err(|_| {
                InjectError::InternalError(format!(
                    "the provider registry for {:?} is an invalid type",
                    interface_info.name()
                ))
            })
            .map(|registry| *registry)
    }

    pub fn reclaim<I>(
        &mut self,
        provider_registry: ProviderRegistry<I>,
    ) -> InjectResult<()>
    where
        I: ?Sized + Interface,
    {
        // Get the provider registry's slot
        let interface_info = ServiceInfo::of::<I>();
        let slot = self
            .provider_registries
            .get_mut(&interface_info)
            .ok_or_else(|| {
                InjectError::InternalError(format!(
                    "activated providers for {} are no longer registered",
                    interface_info.name()
                ))
            })?;

        // Put the provider registry into the slot
        let replaced = slot.replace(Box::new(provider_registry));
        if let Some(replaced) = replaced {
            slot.replace(replaced);
            return Err(InjectError::InternalError(format!(
                "slot for the provider registry for {:?} has already been reclaimed",
                interface_info.name()
            )));
        }

        Ok(())
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
            .inner_mut()
            .ok_or_else(|| InjectError::CycleDetected {
                service_info,
                cycle: vec![service_info],
            })?
            .downcast_mut()
            .ok_or_else(|| {
                InjectError::InternalError(format!(
                    "provider registry for interface {:?} is the wrong type",
                    interface_info.name()
                ))
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
        let slot = self
            .provider_registries
            .get_mut(&interface_info)
            .ok_or_else(|| {
                InjectError::InternalError(format!(
                    "activated provider for {} is no longer registered",
                    interface_info.name()
                ))
            })?;
        let provider_registry = slot.inner_mut().ok_or_else(|| {
            InjectError::InternalError(format!(
                "activated provider for {} is in use",
                interface_info.name()
            ))
        })?;

        // Downcast the provider registry
        let provider_registry: &mut ProviderRegistry<_> =
            provider_registry.downcast_mut().ok_or_else(|| {
                InjectError::InternalError(format!(
                    "provider registry for interface {:?} is the wrong type",
                    interface_info.name()
                ))
            })?;

        // Reclaim the providers
        provider_registry.reclaim_providers_for(service_info, providers)
    }
}
