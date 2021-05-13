use crate::{
    InjectError, InjectResult, Injector, Interface, MapContainer,
    MapContainerEx, Provider, ProviderMap, RequestInfo, ServiceInfo, Svc,
};
use std::{marker::PhantomData, slice::IterMut};

/// A collection of all the providers for a particular interface.
///
/// If an interface will only have one implementation registered for it, then
/// it may be easier to request [`Svc<T>`] from the container instead. However,
/// if multiple implementations are registered (or no implementations are
/// registered), then this will allow all of those implementations to be
/// iterated over.
///
/// An iterator over all the implementations of an interface. Each service is
/// activated on demand.
///
/// ```
/// use runtime_injector::{
///     interface, Injector, IntoTransient, Services, Svc, TypedProvider,
/// };
///
/// trait Fooable: Send + Sync {
///     fn baz(&self) {}
/// }
///
/// interface!(dyn Fooable = [Foo, Bar]);
///
/// #[derive(Default)]
/// struct Foo;
/// impl Fooable for Foo {}
///
/// #[derive(Default)]
/// struct Bar;
/// impl Fooable for Bar {}
///
/// let mut builder = Injector::builder();
/// builder.provide(Foo::default.transient().with_interface::<dyn Fooable>());
/// builder.provide(Bar::default.transient().with_interface::<dyn Fooable>());
///
/// let injector = builder.build();
/// let mut counter = 0;
/// let mut fooables: Services<dyn Fooable> = injector.get().unwrap();
/// for foo in fooables.get_all() {
///     counter += 1;
///     foo.unwrap().baz();
/// }
///
/// assert_eq!(2, counter);
/// ```
pub struct Services<I: ?Sized + Interface> {
    injector: Injector,
    service_info: ServiceInfo,
    request_info: RequestInfo,
    provider_map: MapContainer<ProviderMap>,
    providers: Option<Vec<Box<dyn Provider>>>,
    marker: PhantomData<fn() -> I>,
}

impl<I: ?Sized + Interface> Services<I> {
    pub(crate) fn new(
        injector: Injector,
        provider_map: MapContainer<ProviderMap>,
        request_info: RequestInfo,
    ) -> InjectResult<Self> {
        let service_info = ServiceInfo::of::<I>();
        let providers = provider_map.with_inner_mut(|provider_map| {
            provider_map
                .get_mut(&service_info)
                .map(|providers| {
                    providers.take().ok_or_else(|| InjectError::CycleDetected {
                        service_info,
                        cycle: vec![service_info],
                    })
                })
                .transpose()?
                .ok_or(InjectError::MissingProvider { service_info })
        })?;

        Ok(Services {
            injector,
            service_info,
            request_info,
            provider_map,
            providers: Some(providers),
            marker: PhantomData,
        })
    }

    /// Lazily gets all the implementations of this interface. Each service
    /// will be requested on demand rather than all at once.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_all(&mut self) -> ServicesIter<'_, I> {
        ServicesIter {
            provider_iter: self.providers.as_mut().unwrap().iter_mut(), /* Should never panic */
            injector: &self.injector,
            request_info: &self.request_info,
            marker: PhantomData,
        }
    }

    /// Lazily gets all the implementations of this interface as owned service
    /// pointers. Each service will be requested on demand rather than all at
    /// once. Not all providers can provide owned service pointers, so some
    /// requests may fail.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_all_owned(&mut self) -> OwnedServicesIter<'_, I> {
        OwnedServicesIter {
            provider_iter: self.providers.as_mut().unwrap().iter_mut(), /* Should never panic */
            injector: &self.injector,
            request_info: &self.request_info,
            marker: PhantomData,
        }
    }

    /// Gets the number of implementations of this interface.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn len(&self) -> usize {
        self.providers
            .as_ref()
            .unwrap() // Should never panic
            .len()
    }

    /// Returns `true` if there are no implementations of this interface.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn is_empty(&self) -> bool {
        self.providers
            .as_ref()
            .unwrap() // Should never panic
            .is_empty()
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

            #[allow(clippy::missing_panics_doc)] // Should never panic
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

/// An iterator over all the implementations of an interface. Each service is
/// activated on demand.
///
/// ```
/// use runtime_injector::{constant, Injector, IntoTransient, Services, Svc};
/// use std::sync::Mutex;
///
/// struct Foo;
///
/// fn make_foo(counter: Svc<Mutex<usize>>) -> Foo {
///     // Increment the counter to track how many Foos have been created
///     let mut counter = counter.lock().unwrap();
///     *counter += 1;
///     Foo
/// }
///
/// let mut builder = Injector::builder();
/// builder.provide(constant(Mutex::new(0usize)));
/// builder.provide(make_foo.transient());
///
/// let injector = builder.build();
/// let counter: Svc<Mutex<usize>> = injector.get().unwrap();
/// let mut foos: Services<Foo> = injector.get().unwrap();
///
/// let mut iter = foos.get_all();
/// assert_eq!(0, *counter.lock().unwrap());
/// assert!(iter.next().is_some());
/// assert_eq!(1, *counter.lock().unwrap());
/// assert!(iter.next().is_none());
/// ```
pub struct ServicesIter<'a, I: ?Sized + Interface> {
    provider_iter: IterMut<'a, Box<dyn Provider>>,
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    marker: PhantomData<fn() -> I>,
}

impl<'a, I: ?Sized + Interface> Iterator for ServicesIter<'a, I> {
    type Item = InjectResult<Svc<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ServicesIter {
            provider_iter,
            injector,
            request_info,
            ..
        } = self;

        provider_iter
            .flat_map(|provider| {
                match provider.provide(injector, request_info) {
                    Ok(result) => Some(I::downcast(result)),
                    Err(InjectError::ConditionsNotMet { .. }) => None,
                    Err(InjectError::CycleDetected { mut cycle, .. }) => {
                        let service_info = ServiceInfo::of::<I>();
                        cycle.push(service_info);
                        Some(Err(InjectError::CycleDetected {
                            service_info,
                            cycle,
                        }))
                    }
                    Err(error) => Some(Err(error)),
                }
            })
            .next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.provider_iter.len()))
    }
}

/// An iterator over all the implementations of an interface. Each service is
/// activated on demand.
///
/// ```
/// use runtime_injector::{constant, Injector, IntoTransient, Services, Svc};
/// use std::sync::Mutex;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// struct Foo(usize);
///
/// fn make_foo(counter: Svc<Mutex<usize>>) -> Foo {
///     // Increment the counter to track how many Foos have been created
///     let mut counter = counter.lock().unwrap();
///     *counter += 1;
///     Foo(*counter)
/// }
///
/// let mut builder = Injector::builder();
/// builder.provide(constant(Mutex::new(0usize)));
/// builder.provide(make_foo.transient());
///
/// let injector = builder.build();
/// let counter: Svc<Mutex<usize>> = injector.get().unwrap();
/// let mut foos: Services<Foo> = injector.get().unwrap();
///
/// let mut iter = foos.get_all_owned();
/// assert_eq!(0, *counter.lock().unwrap());
/// assert_eq!(Foo(1), *iter.next().unwrap().unwrap());
/// assert_eq!(1, *counter.lock().unwrap());
/// assert!(iter.next().is_none());
/// ```
pub struct OwnedServicesIter<'a, I: ?Sized + Interface> {
    provider_iter: IterMut<'a, Box<dyn Provider>>,
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    marker: PhantomData<fn() -> I>,
}

impl<'a, I: ?Sized + Interface> Iterator for OwnedServicesIter<'a, I> {
    type Item = InjectResult<Box<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        let OwnedServicesIter {
            provider_iter,
            injector,
            request_info,
            ..
        } = self;

        provider_iter
            .flat_map(|provider| {
                match provider.provide_owned(injector, request_info) {
                    Ok(result) => Some(I::downcast_owned(result)),
                    Err(InjectError::ConditionsNotMet { .. }) => None,
                    Err(InjectError::CycleDetected { mut cycle, .. }) => {
                        let service_info = ServiceInfo::of::<I>();
                        cycle.push(service_info);
                        Some(Err(InjectError::CycleDetected {
                            service_info,
                            cycle,
                        }))
                    }
                    Err(error) => Some(Err(error)),
                }
            })
            .next()
    }
}
