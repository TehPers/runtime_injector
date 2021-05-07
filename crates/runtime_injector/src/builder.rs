use crate::{Injector, Module, Provider, ProviderMap};

/// A builder for an [`Injector`].
#[derive(Default)]
pub struct InjectorBuilder {
    providers: ProviderMap,
}

impl InjectorBuilder {
    /// Assigns the provider for a service type. Multiple providers can be
    /// registered for a service.
    #[allow(clippy::missing_panics_doc)]
    pub fn provide<P: Provider>(&mut self, provider: P) {
        // Should never panic
        self.providers
            .entry(provider.result())
            .or_insert_with(|| Some(Vec::new()))
            .as_mut()
            .unwrap()
            .push(Box::new(provider));
    }

    /// Adds all the providers registered in a module. This may cause multiple
    /// providers to be registered for the same service.
    #[allow(clippy::missing_panics_doc)]
    pub fn add_module(&mut self, module: Module) {
        for (result, module_providers) in module.providers {
            // Should never panic
            let mut module_providers = module_providers.unwrap();
            self.providers
                .entry(result)
                .and_modify(|providers| {
                    // Should never panic
                    providers.as_mut().unwrap().append(&mut module_providers)
                })
                .or_insert_with(|| Some(module_providers));
        }
    }

    /// Builds the injector.
    #[must_use]
    pub fn build(self) -> Injector {
        Injector::new(self.providers)
    }
}
