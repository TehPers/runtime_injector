use crate::{Injector, InterfaceFor, Provider, Service, ServiceInfo};
use std::collections::HashMap;

#[derive(Default)]
pub struct InjectorBuilder {
    providers: HashMap<ServiceInfo, Option<Box<dyn Provider>>>,
    implementations: HashMap<ServiceInfo, ServiceInfo>,
}

impl InjectorBuilder {
    pub fn provide<P: Provider>(
        &mut self,
        provider: P,
    ) -> Option<Box<dyn Provider>> {
        let result = provider.result();
        let provider = Box::new(provider);
        self.providers.insert(result, Some(provider)).flatten()
    }

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

    #[must_use]
    pub fn build(self) -> Injector {
        Injector::new(self.providers, self.implementations)
    }
}
