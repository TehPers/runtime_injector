use crate::{
    InjectError, InjectResult, Injector, Interface, MapContainer,
    MapContainerEx, Provider, ProviderMap, Service, ServiceInfo, Svc,
};
use std::marker::PhantomData;

pub struct Services<I: ?Sized + Interface> {
    pub(crate) injector: Injector,
    pub(crate) service_info: ServiceInfo,
    pub(crate) provider_map: MapContainer<ProviderMap>,
    pub(crate) providers: Option<Vec<Box<dyn Provider>>>,
    pub(crate) marker: PhantomData<*const I>,
}

impl<I: ?Sized + Interface> Services<I> {
    pub fn get_all(&mut self) -> ServicesIter<'_, I> {
        ServicesIter {
            providers: self.providers.as_mut().unwrap(),
            injector: &self.injector,
            service_info: self.service_info,
            index: 0,
            marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.providers.as_ref().unwrap().len()
    }
}

impl<I: ?Sized + Interface> Drop for Services<I> {
    fn drop(&mut self) {
        let Services {
            ref service_info,
            ref mut provider_map,
            ref mut providers,
            ..
        } = self;

        let result = provider_map.with_inner_mut(|provider_map| {
            let provider_entry =
                provider_map.get_mut(service_info).ok_or_else(|| {
                    InjectError::InternalError(format!(
                        "activated provider for {} is no longer registered",
                        service_info.name()
                    ))
                })?;

            if provider_entry.replace(providers.take().unwrap()).is_some() {
                Err(InjectError::InternalError(format!(
                    "another provider for {} was added during its activation",
                    service_info.name()
                )))
            } else {
                Ok(())
            }
        });

        if let Err(error) = result {
            eprintln!(
                "An error occurred while releasing providiers for {}: {}",
                service_info.name(),
                error
            );
        }
    }
}

pub struct ServicesIter<'a, I: ?Sized + Interface> {
    providers: &'a mut Vec<Box<dyn Provider>>,
    injector: &'a Injector,
    service_info: ServiceInfo,
    index: usize,
    marker: PhantomData<*const I>,
}

impl<'a, I: ?Sized + Interface> Iterator for ServicesIter<'a, I> {
    type Item = InjectResult<Svc<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.providers.get_mut(self.index) {
            None => None,
            Some(provider) => {
                self.index += 1;
                Some(
                    provider
                        .provide(self.injector)
                        .and_then(|result| I::downcast(result)),
                )
            }
        }
    }
}

// TODO
pub struct Interfaces<T: ?Sized + Interface> {
    marker: PhantomData<*const T>,
}
