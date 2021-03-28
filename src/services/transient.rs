use crate::{
    Dependencies, InjectResult, Injector, ProviderFunction, Service, Svc,
    TypedProvider,
};
use std::marker::PhantomData;

pub struct TransientProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    func: F,
    marker: PhantomData<fn(D) -> InjectResult<R>>,
}

impl<D, R, F> TransientProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    pub fn new(func: F) -> Self {
        TransientProvider {
            func,
            marker: PhantomData,
        }
    }
}

impl<D, R, F> TypedProvider for TransientProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    type Result = R;

    fn provide_typed(
        &mut self,
        injector: &mut Injector,
    ) -> InjectResult<Svc<Self::Result>> {
        self.func.invoke(injector)
    }
}

pub trait IntoTransient<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    fn transient(self) -> TransientProvider<D, R, F>;
}

impl<D, R, F> IntoTransient<D, R, F> for F
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    fn transient(self) -> TransientProvider<D, R, F> {
        TransientProvider::new(self)
    }
}

impl<D, R, F> From<F> for TransientProvider<D, R, F>
where
    D: Dependencies,
    R: Service,
    F: ProviderFunction<D, R>,
{
    fn from(func: F) -> Self {
        func.transient()
    }
}
