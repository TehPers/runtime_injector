use actix_web::{
    dev::Payload, error::ErrorInternalServerError, FromRequest, HttpRequest,
};
use futures_util::future::{err, ok, Ready};
use runtime_injector::{Injector, Request};
use std::{fmt::Display, ops::Deref};

/// An injected request. Any request to the [`Injector`] can be injected by
/// wrapping it in this type and providing it as a parameter to your request
/// handler.
///
/// ## Example
///
/// ```no_run
/// use actix_web::{get, App, HttpResponse, HttpServer, Responder};
/// use runtime_injector_actix::{
///     constant, define_module, Injected, Injector, Svc,
/// };
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let mut builder = Injector::builder();
///     builder.add_module(define_module! {
///         services = [constant(4i32)],
///     });
///
///     let injector = builder.build();
///     HttpServer::new(move || {
///         App::new().app_data(injector.clone()).service(index)
///     })
///     .bind(("127.0.0.1", 8080))?
///     .run()
///     .await
/// }
///
/// #[get("/")]
/// async fn index(my_service: Injected<Svc<i32>>) -> impl Responder {
///     HttpResponse::Ok().body(format!("injected value is {}", *my_service))
/// }
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Injected<R>(R)
where
    R: Request;

impl<R> Injected<R>
where
    R: Request,
{
    /// Converts an [`Injected<R>`] to its inner value.
    pub fn into_inner(value: Injected<R>) -> R {
        value.0
    }
}

impl<R> Deref for Injected<R>
where
    R: Request,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R> Display for Injected<R>
where
    R: Request + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<R> FromRequest for Injected<R>
where
    R: Request,
{
    type Error = actix_web::Error;
    type Future = Ready<actix_web::Result<Self>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let injector: &Injector = match req.app_data() {
            Some(app_data) => app_data,
            None => {
                return err(ErrorInternalServerError(
                    "no injector is present in app_data",
                ));
            }
        };

        let inner = match injector.get() {
            Ok(inner) => inner,
            Err(error) => return err(ErrorInternalServerError(error)),
        };

        ok(Injected(inner))
    }
}
