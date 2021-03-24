use crate::{
    DynSvc, InjectError, InjectResult, InjectorBuilder, Interface, Provider, Service, ServiceInfo,
    Svc,
};
use std::collections::HashMap;

/// A runtime dependency injection container. This holds all the bindings
/// between service types and their providers, as well as all the mappings from
/// interfaces to their implementations (if they differ).
pub struct Injector {
    providers: HashMap<ServiceInfo, Option<Box<dyn Provider>>>,
    implementations: HashMap<ServiceInfo, ServiceInfo>,
}

impl Injector {
    pub fn builder() -> InjectorBuilder {
        InjectorBuilder::default()
    }

    pub fn new(
        providers: HashMap<ServiceInfo, Option<Box<dyn Provider>>>,
        implementations: HashMap<ServiceInfo, ServiceInfo>,
    ) -> Self {
        Injector {
            providers,
            implementations,
        }
    }

    /// Gets an implementation of the given type. If the type is a sized type,
    /// then this will attempt to activate an instance of that type using a
    /// registered provider. If the type is a dynamic type (`dyn Trait`), then
    /// an instancethe type registered as the implementation of that trait will
    /// be activated instead.
    pub fn get<T: ?Sized + Interface>(&mut self) -> InjectResult<Svc<T>> {
        T::resolve(
            self,
            self.implementations.get(&ServiceInfo::of::<T>()).copied(),
        )
    }

    /// Gets an instance of the service with exactly the type that was
    /// requested. This will not attempt to find the type registered as an
    /// implementation of a particular trait. In fact, dynamic types (`dyn
    /// Trait`) cannot be used with this function.
    pub fn get_exact<T: Service>(&mut self) -> InjectResult<Svc<T>> {
        let service_info = ServiceInfo::of::<T>();
        self.get_dyn_exact(service_info)?.downcast().map_err(|_| {
            InjectError::InternalError(format!(
                "request for {} yielded the wrong type of service",
                service_info.name()
            ))
        })
    }

    /// Similar to `get_exact`, but returns an instance of `dyn Any` instead,
    /// and does not need the type passed in via a type parameter.
    pub fn get_dyn_exact(&mut self, service_info: ServiceInfo) -> InjectResult<DynSvc> {
        let provider = self
            .providers
            .get_mut(&service_info)
            .ok_or(InjectError::MissingProvider { service_info })?;

        let mut provider = provider.take().ok_or(InjectError::CycleDetected {
            service_info,
            cycle: vec![service_info],
        })?;

        let result = match provider.provide(self) {
            Ok(result) => result,
            Err(InjectError::CycleDetected { mut cycle, .. }) => {
                cycle.push(service_info);
                return Err(InjectError::CycleDetected {
                    service_info,
                    cycle,
                });
            }
            Err(e) => return Err(e),
        };

        // Need to get the entry again since it could have been removed by a provider (it shouldn't have though)
        let provider_entry = self.providers.get_mut(&service_info).ok_or_else(|| {
            InjectError::InternalError(format!(
                "activated provider for {} is no longer registered",
                service_info.name()
            ))
        })?;
        let old_value = provider_entry.replace(provider);
        if let Some(_) = old_value {
            return Err(InjectError::InternalError(format!(
                "another provider for {} was added during its activation",
                service_info.name()
            )));
        }

        Ok(result)
    }
}
