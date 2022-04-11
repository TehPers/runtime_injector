use crate::{
    FromProvider, InjectError, InjectResult, Injector, Provider,
    ProvidersLease, RequestInfo, ServiceInfo, Svc,
};
use std::{marker::PhantomData, slice::IterMut};

pub struct Services<S>
where
    S: ?Sized + FromProvider,
{
    injector: Injector,
    request_info: RequestInfo,
    providers: ProvidersLease<S::Interface>,
    _marker: PhantomData<fn() -> S>,
}

impl<S> Services<S>
where
    S: ?Sized + FromProvider,
{
    #[inline]
    pub(crate) fn new(
        injector: Injector,
        request_info: RequestInfo,
        providers: ProvidersLease<S::Interface>,
    ) -> Self {
        Services {
            injector,
            request_info,
            providers,
            _marker: PhantomData,
        }
    }

    /// Lazily gets all provided services of the given type. Each service will
    /// be requested on demand rather than all at once.
    #[inline]
    pub fn iter(&mut self) -> ServicesIter<'_, S> {
        ServicesIter {
            injector: &self.injector,
            request_info: &self.request_info,
            provider_iter: self.providers.iter_mut(),
            _marker: PhantomData,
        }
    }

    /// Lazily gets all provided owned services of the given type. Each service
    /// will be requested on demand rather than all at once.
    #[inline]
    pub fn iter_owned(&mut self) -> OwnedServicesIter<'_, S> {
        OwnedServicesIter {
            injector: &self.injector,
            request_info: &self.request_info,
            provider_iter: self.providers.iter_mut(),
            _marker: PhantomData,
        }
    }

    /// Gets the number of provided services of the given type registered to
    /// this interface. This does not take into account conditional providers
    /// which may not return an implementation of the service.
    #[inline]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Returns whether there are no providers for the given service and
    /// interface. Conditional providers still may not return an implementation
    /// of the service even if this returns `true`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

pub struct ServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    provider_iter: IterMut<'a, Box<dyn Provider<Interface = S::Interface>>>,
    _marker: PhantomData<fn() -> S>,
}

impl<'a, S> Iterator for ServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    type Item = InjectResult<Svc<S>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider_iter.find_map(|provider| {
            // Skip providers that don't match the requested service
            if !S::should_provide(provider.as_ref()) {
                return None;
            }

            // Provide the service
            let service =
                match provider.provide(self.injector, self.request_info) {
                    Ok(service) => service,
                    Err(InjectError::ConditionsNotMet { .. }) => return None,
                    Err(InjectError::CycleDetected { mut cycle, .. }) => {
                        let service_info = ServiceInfo::of::<S>();
                        cycle.push(service_info);
                        return Some(Err(InjectError::CycleDetected {
                            service_info,
                            cycle,
                        }));
                    }
                    Err(error) => return Some(Err(error)),
                };

            // Downcast the service
            let service = S::from_provided(service);
            Some(service)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.provider_iter.size_hint()
    }
}

pub struct OwnedServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    provider_iter: IterMut<'a, Box<dyn Provider<Interface = S::Interface>>>,
    _marker: PhantomData<fn() -> S>,
}

impl<'a, S> Iterator for OwnedServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    type Item = InjectResult<Box<S>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider_iter.find_map(|provider| {
            // Skip providers that don't match the requested service
            if !S::should_provide(provider.as_ref()) {
                return None;
            }

            // Provide the service
            let service = match provider
                .provide_owned(self.injector, self.request_info)
            {
                Ok(service) => service,
                Err(InjectError::ConditionsNotMet { .. }) => return None,
                Err(InjectError::CycleDetected { mut cycle, .. }) => {
                    let service_info = ServiceInfo::of::<S>();
                    cycle.push(service_info);
                    return Some(Err(InjectError::CycleDetected {
                        service_info,
                        cycle,
                    }));
                }
                Err(error) => return Some(Err(error)),
            };

            // Downcast the service
            let service = S::from_provided_owned(service);
            Some(service)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.provider_iter.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
