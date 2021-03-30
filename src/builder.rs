use crate::{Injector, Interface, Provider, ProviderMap, ServiceInfo};

/// A builder for an `Injector`.
#[derive(Default)]
pub struct InjectorBuilder {
    providers: ProviderMap,
    // implementations: ImplementationMap,
}

impl InjectorBuilder {
    /// Assigns the provider for a service type. Multiple providers can be
    /// registered for a service.
    pub fn provide<P: Provider>(&mut self, provider: P) {
        self.providers
            .entry(provider.result())
            .or_insert_with(|| Some(Vec::new()))
            .as_mut()
            .unwrap()
            .push(Box::new(provider));
    }

    pub fn provide_as<I, P>(&mut self, provider: P)
    where
        I: ?Sized + Interface,
        P: Provider,
    {
        self.providers
            .entry(ServiceInfo::of::<I>())
            .or_insert_with(|| Some(Vec::new()))
            .as_mut()
            .unwrap()
            .push(Box::new(provider));
    }

    /// Builds the injector.
    #[must_use]
    pub fn build(self) -> Injector {
        Injector::new(self.providers)
    }
}
