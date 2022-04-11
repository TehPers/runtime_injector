use crate::{
    InjectError, InjectResult, Injector, Interface, Provider, ProvidersLease,
    RequestInfo, Service, ServiceInfo, Services, Svc,
};
use std::slice::IterMut;

/// A collection of all the providers for a particular interface. Each service
/// is activated only during iteration of this collection.
///
/// If an interface will only have one implementation registered for it, then
/// it may be easier to request [`Svc<I>`] from the container instead. However,
/// if multiple implementations are registered (or no implementations are
/// registered), then this will allow all of those implementations to be
/// iterated over.
///
/// ```
/// use runtime_injector::{
///     interface, Injector, IntoTransient, Services, Svc, TypedProvider, Service
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
/// for foo in fooables.get_all() {
///     counter += 1;
///     foo.unwrap().baz();
/// }
///
/// assert_eq!(2, counter);
/// ```
pub struct ServicesOf<I>
where
    I: ?Sized + Interface,
{
    injector: Injector,
    request_info: RequestInfo,
    providers: ProvidersLease<I>,
}

impl<I> ServicesOf<I>
where
    I: ?Sized + Interface,
{
    #[inline]
    pub(crate) fn new(
        injector: Injector,
        request_info: RequestInfo,
        providers: ProvidersLease<I>,
    ) -> Self {
        ServicesOf {
            injector,
            request_info,
            providers,
        }
    }

    /// Lazily gets all the implementations of this interface. Each service
    /// will be requested on demand rather than all at once.
    #[inline]
    pub fn iter(&mut self) -> ServicesOfIter<'_, I> {
        ServicesOfIter {
            injector: &self.injector,
            request_info: &self.request_info,
            provider_iter: self.providers.iter_mut(),
        }
    }

    /// Lazily gets all the implementations of this interface as owned service
    /// pointers. Each service will be requested on demand rather than all at
    /// once. Not all providers can provide owned service pointers, so some
    /// requests may fail.
    #[inline]
    pub fn iter_owned(&mut self) -> OwnedServicesOfIter<'_, I> {
        OwnedServicesOfIter {
            injector: &self.injector,
            request_info: &self.request_info,
            provider_iter: self.providers.iter_mut(),
        }
    }

    /// Gets the max number of possible implementations of this interface. This
    /// does not take into account conditional providers, which may not return
    /// an implementation of the service.
    #[inline]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Returns `true` if there are no possible implementations of this
    /// interface. This does not take into account conditional providers, which
    /// may not return an implementation of the service.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl ServicesOf<dyn Service> {
    pub fn of_service<S>(self) -> Services<S>
    where
        S: Service,
    {
        Services::new(self.injector, self.request_info, self.providers)
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
pub struct ServicesOfIter<'a, I>
where
    I: ?Sized + Interface,
{
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    provider_iter: IterMut<'a, Box<dyn Provider<Interface = I>>>,
}

impl<'a, I> Iterator for ServicesOfIter<'a, I>
where
    I: ?Sized + Interface,
{
    type Item = InjectResult<Svc<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider_iter.find_map(|provider| {
            match provider.provide(self.injector, self.request_info) {
                Ok(service) => Some(Ok(service)),
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
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.provider_iter.size_hint()
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
pub struct OwnedServicesOfIter<'a, I>
where
    I: ?Sized + Interface,
{
    injector: &'a Injector,
    request_info: &'a RequestInfo,
    provider_iter: IterMut<'a, Box<dyn Provider<Interface = I>>>,
}

impl<'a, I> Iterator for OwnedServicesOfIter<'a, I>
where
    I: ?Sized + Interface,
{
    type Item = InjectResult<Box<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider_iter.find_map(|provider| {
            match provider.provide_owned(self.injector, self.request_info) {
                Ok(service) => Some(Ok(service)),
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
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.provider_iter.size_hint()
    }
}
