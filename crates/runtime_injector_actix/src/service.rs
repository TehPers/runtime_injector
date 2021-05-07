use actix_web::{
    dev::Payload, error::ErrorInternalServerError, FromRequest, HttpRequest,
};
use futures_util::future::{err, ok, Ready};
use runtime_injector::{Injector, Request};
use std::ops::Deref;

/// An injected request.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Injected<R: Request>(R);

impl<R: Request> Injected<R> {
    /// Converts an [`Injected<R>`] to its inner value.
    pub fn into_inner(value: Injected<R>) -> R {
        value.0
    }
}

impl<R: Request> Deref for Injected<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: Request> FromRequest for Injected<R> {
    type Error = actix_web::Error;
    type Future = Ready<actix_web::Result<Self>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let injector: &Injector = match req.app_data() {
            Some(app_data) => app_data,
            None => return err(ErrorInternalServerError("missing app_data")),
        };

        let inner = match injector.get() {
            Ok(inner) => inner,
            Err(error) => return err(ErrorInternalServerError(error)),
        };

        ok(Injected(inner))
    }
}
