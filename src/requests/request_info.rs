use crate::ServiceInfo;
use std::{any::Any, collections::HashMap, fmt::Debug};

/// A parameter for configuring requested services.
pub trait RequestParameter: Any + Debug {
    /// Clones this parameter into a boxed trait object.
    fn clone_dyn(&self) -> Box<dyn RequestParameter>;

    /// Casts this parameter into a [`&dyn Any`](Any), making it easier to
    /// downcast into other types.
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Debug + Clone> RequestParameter for T {
    fn clone_dyn(&self) -> Box<dyn RequestParameter> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl dyn RequestParameter {
    /// Tries to downcast this request parameter to a concrete type.
    pub fn downcast_ref<T: RequestParameter>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
}

impl Clone for Box<dyn RequestParameter> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

/// Information about an active request.
#[derive(Clone, Debug)]
pub struct RequestInfo {
    service_path: Vec<ServiceInfo>,
    parameters: HashMap<String, Box<dyn RequestParameter>>,
}

impl RequestInfo {
    /// Creates a new, empty instance of [`RequestInfo`].
    #[must_use]
    pub fn new() -> Self {
        RequestInfo {
            service_path: Vec::new(),
            parameters: HashMap::new(),
        }
    }

    /// Sets a request parameter for the request. If a parameter has already
    /// been set to a value, then that value is returned.
    pub fn insert_parameter(
        &mut self,
        key: &str,
        value: impl RequestParameter,
    ) -> Option<Box<dyn RequestParameter>> {
        self.parameters.insert(key.to_owned(), Box::new(value))
    }

    /// Creates a new child instance of [`RequestInfo`] with the given service
    /// appended to the end of the request path.
    #[must_use]
    pub fn with_request(&self, service: ServiceInfo) -> Self {
        let mut child = self.clone();
        child.service_path.push(service);
        child
    }

    /// Gets the current request path. This can be used to configure a service
    /// based on what it's being injected into.
    ///
    /// # Example
    ///
    /// ```
    /// use runtime_injector::{Injector, IntoTransient, RequestInfo, ServiceInfo, Svc};
    ///
    /// struct Foo(pub Svc<Baz>);
    /// struct Bar(pub Svc<Baz>);
    /// struct Baz(pub i32);
    ///
    /// impl Baz {
    ///     pub fn new(request_info: RequestInfo) -> Self {
    ///         let service_path = request_info.service_path();
    ///         let value = match service_path.get(0) {
    ///             Some(root) if root == &ServiceInfo::of::<Foo>() => 1,
    ///             Some(root) if root == &ServiceInfo::of::<Bar>() => 2,
    ///             _ => 0,
    ///         };
    ///         
    ///         Baz(value)
    ///     }
    /// }
    ///
    /// let mut builder = Injector::builder();
    /// builder.provide(Foo.transient());
    /// builder.provide(Bar.transient());
    /// builder.provide(Baz::new.transient());
    ///
    /// let injector = builder.build();
    /// let foo: Svc<Foo> = injector.get().unwrap();
    /// let bar: Svc<Bar> = injector.get().unwrap();
    /// let baz: Svc<Baz> = injector.get().unwrap();
    ///
    /// assert_eq!(1, foo.0.0);
    /// assert_eq!(2, bar.0.0);
    /// assert_eq!(0, baz.0);
    /// ```
    #[must_use]
    pub fn service_path(&self) -> &[ServiceInfo] {
        &self.service_path
    }

    /// Gets the value of a parameter if it has been set.
    pub fn get_parameter(&self, key: &str) -> Option<&dyn RequestParameter> {
        self.parameters
            .get(key)
            .map(|parameter| parameter.as_ref())
    }
}

impl Default for RequestInfo {
    fn default() -> Self {
        RequestInfo::new()
    }
}
