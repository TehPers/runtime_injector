use crate::{
    provider_registry::{
        InterfaceRegistry, ProviderRegistry, ProviderRegistryIterMut,
    },
    InjectError, InjectResult, Interface, MapContainer, Provider, ServiceInfo,
};
use std::slice::IterMut;

enum ProvidersSource<I>
where
    I: ?Sized + Interface,
{
    Services {
        providers: Vec<Box<dyn Provider<Interface = I>>>,
        service_info: ServiceInfo,
    },
    Interface {
        provider_registry: ProviderRegistry<I>,
    },
}

pub struct Providers<I>
where
    I: ?Sized + Interface,
{
    parent_registry: MapContainer<InterfaceRegistry>,
    providers_source: ProvidersSource<I>,
}

impl<I> Providers<I>
where
    I: ?Sized + Interface,
{
    #[inline]
    pub(crate) fn services(
        parent_registry: MapContainer<InterfaceRegistry>,
        providers: Vec<Box<dyn Provider<Interface = I>>>,
        service_info: ServiceInfo,
    ) -> Self {
        Providers {
            parent_registry,
            providers_source: ProvidersSource::Services {
                providers,
                service_info,
            },
        }
    }

    #[inline]
    pub(crate) fn interface(
        parent_registry: MapContainer<InterfaceRegistry>,
        provider_registry: ProviderRegistry<I>,
    ) -> Self {
        Providers {
            parent_registry,
            providers_source: ProvidersSource::Interface { provider_registry },
        }
    }

    #[inline]
    pub fn iter(&mut self) -> ProviderIter<'_, I> {
        match self.providers_source {
            ProvidersSource::Services {
                ref mut providers,
                service_info,
            } => ProviderIter::Services(ServiceProviderIter {
                providers: providers.iter_mut(),
                service_info,
            }),
            ProvidersSource::Interface {
                ref mut provider_registry,
            } => ProviderIter::Interface(InterfaceProviderIter {
                inner: provider_registry.iter_mut(),
            }),
        }
    }
}

impl<I> Drop for Providers<I>
where
    I: ?Sized + Interface,
{
    fn drop(&mut self) {
        let result = self
            .parent_registry
            .lock()
            .map_err(|_| {
                InjectError::InternalError(
                    "failed to acquire lock for interface registry".into(),
                )
            })
            .and_then(|mut registry| match self.providers_source {
                ProvidersSource::Services {
                    ref mut providers,
                    service_info,
                } => {
                    let providers = std::mem::take(providers);
                    registry.reclaim_providers_for(service_info, providers)
                }
                ProvidersSource::Interface {
                    ref mut provider_registry,
                } => {
                    let provider_registry = std::mem::take(provider_registry);
                    registry.reclaim(provider_registry)
                }
            });

        if let Err(error) = result {
            eprintln!(
                "An error occurred while releasing providiers for {}: {:?}",
                ServiceInfo::of::<I>().name(),
                error
            );
        }
    }
}

impl<'a, I> IntoIterator for &'a mut Providers<I>
where
    I: ?Sized + Interface,
{
    type Item = InjectResult<&'a mut dyn Provider<Interface = I>>;
    type IntoIter = ProviderIter<'a, I>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct ServiceProviderIter<'a, I>
where
    I: ?Sized + Interface,
{
    providers: IterMut<'a, Box<dyn Provider<Interface = I>>>,
    service_info: ServiceInfo,
}

impl<'a, I> Iterator for ServiceProviderIter<'a, I>
where
    I: ?Sized + Interface,
{
    type Item = InjectResult<&'a mut dyn Provider<Interface = I>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.providers.find_map(|provider| {
            // Skip providers that don't match the requested service
            if provider.result() != self.service_info {
                return None;
            }

            // Return the provider
            Some(Ok(provider.as_mut()))
        })
    }
}

pub struct InterfaceProviderIter<'a, I>
where
    I: ?Sized + Interface,
{
    inner: ProviderRegistryIterMut<'a, I>,
}

impl<'a, I> Iterator for InterfaceProviderIter<'a, I>
where
    I: ?Sized + Interface,
{
    type Item = InjectResult<&'a mut dyn Provider<Interface = I>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub enum ProviderIter<'a, I>
where
    I: ?Sized + Interface,
{
    Services(ServiceProviderIter<'a, I>),
    Interface(InterfaceProviderIter<'a, I>),
}

impl<'a, I> Iterator for ProviderIter<'a, I>
where
    I: ?Sized + Interface,
{
    type Item = InjectResult<&'a mut dyn Provider<Interface = I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ProviderIter::Services(inner) => inner.next(),
            ProviderIter::Interface(inner) => inner.next(),
        }
    }
}
