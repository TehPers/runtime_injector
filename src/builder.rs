use crate::{Injector, InterfaceFor, Provider, Service, ServiceInfo};
use std::collections::HashMap;

/// A builder for an `Injector`.
#[derive(Default)]
pub struct InjectorBuilder {
    providers: HashMap<ServiceInfo, Option<Box<dyn Provider>>>,
    implementations: HashMap<ServiceInfo, ServiceInfo>,
}

impl InjectorBuilder {
    /// Assigns the provider for a service type. If a provider was already
    /// registered for the same service type, then that old provider is
    /// returned and the new provider is used instead.
    pub fn provide<P: Provider>(
        &mut self,
        provider: P,
    ) -> Option<Box<dyn Provider>> {
        let result = provider.result();
        let provider = Box::new(provider);
        self.providers.insert(result, Some(provider)).flatten()
    }

    /// Assigns the implementation of an interface for a service type.
    pub fn implement<Interface, Implementation>(
        &mut self,
    ) -> Option<ServiceInfo>
    where
        Interface: ?Sized + InterfaceFor<Implementation>,
        Implementation: Service,
    {
        self.implementations.insert(
            ServiceInfo::of::<Interface>(),
            ServiceInfo::of::<Implementation>(),
        )
    }

    /// Builds the injector.
    #[must_use]
    pub fn build(self) -> Injector {
        Injector::new(self.providers, self.implementations)
    }
}
