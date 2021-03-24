use crate::{Injector, InterfaceFor, Provider, Service, ServiceInfo, TypedProvider};
use std::collections::HashMap;

#[derive(Default)]
pub struct InjectorBuilder {
    providers: HashMap<ServiceInfo, Option<Box<dyn Provider>>>,
    implementations: HashMap<ServiceInfo, ServiceInfo>,
}

impl InjectorBuilder {
    pub fn provide<P: TypedProvider>(&mut self, provider: P) -> Option<Box<dyn Provider>> {
        let result = ServiceInfo::of::<P::Result>();
        let provider = Box::new(provider);
        self.providers.insert(result, Some(provider)).flatten()
    }

    pub fn implement<Interface, Implementation>(&mut self) -> Option<ServiceInfo>
    where
        Interface: ?Sized + InterfaceFor<Implementation>,
        Implementation: Service,
    {
        self.implementations.insert(
            ServiceInfo::of::<Interface>(),
            ServiceInfo::of::<Implementation>(),
        )
    }

    pub fn build(self) -> Injector {
        Injector::new(self.providers, self.implementations)
    }
}
