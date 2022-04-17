use crate::{
    FromProvider, Provider, ProviderRegistry, ProviderRegistryIter, Svc,
};

/// A collection of all the providers for a particular service or interface. No
/// services are activated during iteration of this collection.
///
/// ```
/// use runtime_injector::{
///     interface, Injector, IntoTransient, Providers, Service, Svc,
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
/// let mut fooables: Providers<dyn Fooable> = injector.get().unwrap();
/// assert_eq!(2, fooables.iter().count());
/// ```
pub struct Providers<S>
where
    S: ?Sized + FromProvider,
{
    parent_registry: Svc<ProviderRegistry<S::Interface>>,
}

impl<S> Providers<S>
where
    S: ?Sized + FromProvider,
{
    pub(crate) fn new(
        parent_registry: Svc<ProviderRegistry<S::Interface>>,
    ) -> Self {
        Self { parent_registry }
    }

    /// Gets all the providers for the given type. No services are activated
    /// during iteration of this collection.
    #[inline]
    pub fn iter(&mut self) -> ProviderIter<'_, S> {
        ProviderIter {
            inner: self.parent_registry.iter(),
        }
    }
}

impl<'a, S> IntoIterator for &'a mut Providers<S>
where
    S: ?Sized + FromProvider,
{
    type Item = &'a dyn Provider<Interface = S::Interface>;
    type IntoIter = ProviderIter<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the providers for the given service or interface type.
pub struct ProviderIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    inner: ProviderRegistryIter<'a, S::Interface>,
}

impl<'a, S> Iterator for ProviderIter<'a, S>
where
    S: ?Sized + FromProvider,
{
    type Item = &'a dyn Provider<Interface = S::Interface>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|provider| {
            // Skip providers that don't match the filter
            if !S::should_provide(provider) {
                return None;
            }

            // Return the provider
            Some(provider)
        })
    }
}
