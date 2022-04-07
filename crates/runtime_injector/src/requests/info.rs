use crate::{RequestParameter, ServiceInfo};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

/// Information about an active request.
#[derive(Clone)]
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
    /// ## Example
    ///
    /// ```
    /// use runtime_injector::{
    ///     Injector, IntoTransient, RequestInfo, ServiceInfo, Svc,
    /// };
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
    #[rustfmt::skip]
    /// assert_eq!(1, foo.0.0);
    /// assert_eq!(2, bar.0.0);
    /// assert_eq!(0, baz.0);
    /// ```
    #[must_use]
    pub fn service_path(&self) -> &[ServiceInfo] {
        &self.service_path
    }

    /// Sets the value of a request parameter for the request. If a parameter
    /// has already been set to a value, then that value is returned.
    pub fn insert_parameter(
        &mut self,
        key: &str,
        value: impl RequestParameter,
    ) -> Option<Box<dyn RequestParameter>> {
        self.insert_parameter_boxed(key, Box::new(value))
    }

    pub(crate) fn insert_parameter_boxed(
        &mut self,
        key: &str,
        value: Box<dyn RequestParameter>,
    ) -> Option<Box<dyn RequestParameter>> {
        self.parameters.insert(key.to_owned(), value)
    }

    /// Removes and returns the value of a parameter if it has been set.
    pub fn remove_parameter(
        &mut self,
        key: &str,
    ) -> Option<Box<dyn RequestParameter>> {
        self.parameters.remove(key)
    }

    /// Gets the value of a parameter if it has been set.
    #[must_use]
    pub fn get_parameter(&self, key: &str) -> Option<&dyn RequestParameter> {
        self.parameters.get(key).map(AsRef::as_ref)
    }

    /// Mutably gets the value of a parameter if it has been set.
    #[must_use]
    pub fn get_parameter_mut(
        &mut self,
        key: &str,
    ) -> Option<&mut dyn RequestParameter> {
        self.parameters.get_mut(key).map(AsMut::as_mut)
    }
}

impl Default for RequestInfo {
    fn default() -> Self {
        RequestInfo::new()
    }
}

impl Debug for RequestInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: maybe use finish_non_exhaustive when 1.53 hits stable
        f.debug_struct("RequestInfo")
            .field("service_path", &self.service_path)
            .finish()

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_is_inserted_then_removed() {
        let mut info = RequestInfo::new();
        info.insert_parameter("foo", "bar".to_string());
        assert_eq!(
            Some(&"bar".to_string()),
            info.get_parameter("foo")
                .map(|p| p.downcast_ref::<String>())
                .flatten()
        );
        assert_eq!(
            Some(Box::new("bar".to_string())),
            info.remove_parameter("foo")
                .map(|p| p.downcast::<String>().ok())
                .flatten()
        );
        assert!(info.get_parameter("foo").is_none());
    }

    #[test]
    fn missing_parameter_is_not_removed() {
        let mut info = RequestInfo::new();
        assert!(info.remove_parameter("foo").is_none());
    }
}
