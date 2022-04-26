use crate::{
    FromProvider, InjectError, InjectResult, Injector, ProviderIter, Providers,
    RequestInfo, Svc,
};

/// A collection of all the implementations for a particular service or
/// interface. Each service is activated only during iteration of this
/// collection.
///
/// If a type only has one implementation registered for it, then it may be
/// easier to request [`Svc<I>`] from the container instead. However, if
/// multiple implementations are registered (or no implementations are
/// registered), then this will allow all of those implementations to be
/// iterated over.
///
/// ```
/// use runtime_injector::{
///     interface, Injector, IntoTransient, Service, Services, Svc,
///     TypedProvider, WithInterface,
/// };
///
/// trait Fooable: Service {
///     fn baz(&self) {}
/// }
///
/// interface!(Fooable);
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
/// for foo in fooables.iter() {
///     counter += 1;
///     foo.unwrap().baz();
/// }
///
/// assert_eq!(2, counter);
/// ```
pub struct Services<S>
where
    S: ?Sized + FromProvider,
{
    injector: Injector,
    request_info: RequestInfo,
    providers: Providers<S>,
}

impl<S> Services<S>
where
    S: ?Sized + FromProvider,
{
    #[inline]
    pub(crate) fn new(
        injector: Injector,
        request_info: RequestInfo,
        providers: Providers<S>,
    ) -> Self {
        Services {
            injector,
            request_info,
            providers,
        }
    }

    /// Lazily gets all provided services of the given type. Each service will
    /// be requested on demand rather than all at once.
    #[inline]
    pub fn iter(&mut self) -> ServicesIter<'_, S> {
        ServicesIter {
            injector: &self.injector,
            request_info: &self.request_info,
            provider_iter: self.providers.iter(),
        }
    }

    /// Lazily gets all provided owned services of the given type. Each service
    /// will be requested on demand rather than all at once.
    #[inline]
    pub fn iter_owned(&mut self) -> OwnedServicesIter<'_, S> {
        OwnedServicesIter {
            injector: &self.injector,
            request_info: &self.request_info,
            provider_iter: self.providers.iter(),
        }
    }
}

/// An iterator over the provided services of the given type. Each service is
/// activated on demand. Because activation of a service may fail, this
/// iterator returns [`InjectResult<S>`].
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
/// let mut iter = foos.iter();
/// assert_eq!(0, *counter.lock().unwrap());
/// assert!(iter.next().is_some());
/// assert_eq!(1, *counter.lock().unwrap());
/// assert!(iter.next().is_none());
/// ```
pub struct ServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    provider_iter: ProviderIter<'a, S>,
}

impl<'a, S> Iterator for ServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    type Item = InjectResult<Svc<S>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider_iter.find_map(|provider| {
            // Provide the service
            let service =
                match provider.provide(self.injector, self.request_info) {
                    Ok(service) => service,
                    Err(InjectError::ConditionsNotMet { .. }) => return None,
                    Err(error) => return Some(Err(error)),
                };

            // Downcast the service
            let service = S::from_interface(service);
            Some(service)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.provider_iter.size_hint()
    }
}

/// An iterator over the provided services of the given type. Each service is
/// activated on demand.
///
/// Not all providers can provide owned pointers to their service. Only owned
/// services are returned, the rest are ignored.
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
/// let mut iter = foos.iter_owned();
/// assert_eq!(0, *counter.lock().unwrap());
/// assert_eq!(Foo(1), *iter.next().unwrap().unwrap());
/// assert_eq!(1, *counter.lock().unwrap());
/// assert!(iter.next().is_none());
/// ```
pub struct OwnedServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    provider_iter: ProviderIter<'a, S>,
}

impl<'a, S> Iterator for OwnedServicesIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    type Item = InjectResult<Box<S>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider_iter.find_map(|provider| {
            // Provide the service
            let service = match provider
                .provide_owned(self.injector, self.request_info)
            {
                Ok(service) => service,
                Err(InjectError::ConditionsNotMet { .. }) => return None,
                Err(error) => return Some(Err(error)),
            };

            // Downcast the service
            let service = S::from_interface_owned(service);
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
    use crate::{constant, IntoSingleton};
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn service_initialized_only_on_iteration() {
        struct Counter;
        impl Counter {
            fn new(flag: Svc<AtomicBool>) -> Self {
                flag.store(true, Ordering::Relaxed);
                Counter
            }
        }

        // Setup injector
        let mut builder = Injector::builder();
        builder.provide(Counter::new.singleton());
        builder.provide(constant(AtomicBool::new(false)));

        let injector = builder.build();
        let mut services: Services<Counter> = injector.get().unwrap();
        let initialized: Svc<AtomicBool> = injector.get().unwrap();

        // Check that it isn't initialized yet
        assert!(!initialized.load(Ordering::Relaxed));

        // Check that it is initialized after iteration
        let _next = services.iter().next().unwrap().unwrap();
        assert!(initialized.load(Ordering::Relaxed));
    }
}
