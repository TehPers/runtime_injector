use crate::{InjectResult, Injector, Request, RequestInfo};
use std::marker::PhantomData;

/// Lazy request factory allowing requests to be made outside of service
/// creation.
///
/// This lets a service make requests to the injector outside of the service's
/// constructor. For instance, the service can make requests for one of its
/// dependencies at any time, and as many times as it wants, outside of the
/// service's constructor.
///
/// [`Factory`] can be cloned, making it easy to specialize each request made
/// by the factory as needed.
///
/// ## Example
///
/// ```
/// use runtime_injector::{
///     Factory, Injector, IntoSingleton, IntoTransient, Svc,
/// };
///
/// #[derive(Default)]
/// struct Foo;
/// struct Bar(Factory<Box<Foo>>);
///
/// let mut builder = Injector::builder();
/// builder.provide(Foo::default.transient());
/// builder.provide(Bar.singleton());
///
/// let injector = builder.build();
/// let bar: Svc<Bar> = injector.get().unwrap();
/// let _foo1 = bar.0.get().unwrap();
/// let _foo2 = bar.0.get().unwrap();
/// // ...
/// ```
#[derive(Clone)]
pub struct Factory<R: Request> {
    injector: Injector,
    request_info: RequestInfo,
    marker: PhantomData<fn(&Injector, RequestInfo) -> R>,
}

impl<R: Request> Factory<R> {
    /// Performs the factory's inner request.
    pub fn get(&self) -> InjectResult<R> {
        R::request(&self.injector, &self.request_info)
    }

    /// Gets this factory's inner [`RequestInfo`]. This request info is used by
    /// all requests the factory makes.
    pub fn request_info(&self) -> &RequestInfo {
        &self.request_info
    }

    /// Mutably gets this factory's inner [`RequestInfo`]. This request info is
    /// used by all requests the factory makes.
    ///
    /// Modifying this request info affects future requests the factory makes,
    /// meaning additional arguments can be added to requests prior to them
    /// being executed. Since the factory can be cloned, requests can be
    /// specialized by first cloning the factory, then modifying the
    /// [`RequestInfo`] on the clone and using it to make the request instead.
    pub fn request_info_mut(&mut self) -> &mut RequestInfo {
        &mut self.request_info
    }
}

impl<R: Request> Request for Factory<R> {
    fn request(injector: &Injector, info: &RequestInfo) -> InjectResult<Self> {
        Ok(Factory {
            injector: injector.clone(),
            request_info: info.clone(),
            marker: PhantomData,
        })
    }
}
