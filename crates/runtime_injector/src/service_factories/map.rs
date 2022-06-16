use crate::{Service, ServiceFactory};
use std::marker::PhantomData;

pub struct MappedServiceFactory<R, I, D, F>
where
    I: ServiceFactory<D>,
    F: Fn(I::Result) -> R,
    R: Service,
{
    inner: I,
    map: F,
    marker: PhantomData<fn(D) -> I::Result>,
}

impl<R, I, D, F> MappedServiceFactory<R, I, D, F>
where
    I: ServiceFactory<D>,
    F: Fn(I::Result) -> R,
    R: Service,
{
    pub(crate) fn new(inner: I, map: F) -> Self {
        Self {
            inner,
            map,
            marker: PhantomData,
        }
    }
}

impl<R, I, D, F> ServiceFactory<D> for MappedServiceFactory<R, I, D, F>
where
    I: ServiceFactory<D>,
    F: Fn(I::Result) -> R,
    R: Service,
{
    type Result = R;

    fn invoke(
        &self,
        injector: &crate::Injector,
        request_info: &crate::RequestInfo,
    ) -> crate::InjectResult<Self::Result> {
        self.inner.invoke(injector, request_info).map(&self.map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Injector, IntoSingleton};
    use std::sync::Mutex;

    #[test]
    fn foo() {
        #[derive(Default)]
        struct Foo;

        let mut builder = Injector::builder();
        builder.provide(Foo::default.map(Mutex::new).singleton())
    }
}
