use crate::{
    Injector, Module, Provider, ProviderMap, RequestInfo, ServiceInfo,
};

/// A builder for an [`Injector`].
#[derive(Default)]
pub struct InjectorBuilder {
    providers: ProviderMap,
    root_info: RequestInfo,
}

impl InjectorBuilder {
    /// Assigns the provider for a service type. Multiple providers can be
    /// registered for a service.
    pub fn provide<P: Provider>(&mut self, provider: P) {
        self.add_provider(Box::new(provider))
    }

    /// Adds a provider to the injector.
    #[allow(clippy::missing_panics_doc)]
    pub fn add_provider(&mut self, provider: Box<dyn Provider>) {
        // Should never panic
        self.providers
            .entry(provider.result())
            .or_insert_with(|| Some(Vec::new()))
            .as_mut()
            .unwrap()
            .push(provider)
    }

    /// Removes all providers for a service type.
    pub fn remove_providers(
        &mut self,
        service_info: ServiceInfo,
    ) -> Option<Vec<Box<dyn Provider>>> {
        self.providers.remove(&service_info).flatten()
    }

    /// Borrows the root [`RequestInfo`] that will be used by calls to
    /// [`Injector::get()`].
    #[must_use]
    pub fn root_info(&self) -> &RequestInfo {
        &self.root_info
    }

    /// Mutably borrows the root [`RequestInfo`] that will be used by calls to
    /// [`Injector::get()`].
    #[must_use]
    pub fn root_info_mut(&mut self) -> &mut RequestInfo {
        &mut self.root_info
    }

    /// Adds all the providers registered in a module. This may cause multiple
    /// providers to be registered for the same service.
    ///
    /// If any conflicting request parameters have been set before adding this
    /// module, they are overridden.
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

        for (key, value) in module.parameters {
            drop(self.root_info_mut().insert_parameter_boxed(&key, value));
        }
    }

    /// Builds the injector.
    #[must_use]
    pub fn build(self) -> Injector {
        Injector::new_from_parts(self.providers, self.root_info)
    }
}
