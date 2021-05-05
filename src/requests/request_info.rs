use crate::ServiceInfo;

/// Information about an active request.
#[derive(Clone, Debug)]
pub struct RequestInfo {
    service_path: Vec<ServiceInfo>,
}

impl RequestInfo {
    /// Creates a new, empty instance of [`RequestInfo`].
    #[must_use]
    pub fn new() -> Self {
        RequestInfo {
            service_path: Vec::new(),
        }
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
}

impl Default for RequestInfo {
    fn default() -> Self {
        RequestInfo::new()
    }
}
