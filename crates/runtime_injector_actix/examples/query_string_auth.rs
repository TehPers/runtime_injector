//! Spawns a web server that listens on localhost. A password must be sent to
//! access the index page via the query string. Try connecting to
//! <http://localhost:8080/> without any query strings, then connect with the
//! query string `?code=my_secret_password`. The authenticator service is
//! injected via dependency injection into the request handler.

use actix_web::{
    get, web::Query, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use runtime_injector_actix::{
    define_module, Injected, Injector, IntoSingleton, Svc,
};
use serde::Deserialize;

#[derive(Default)]
pub struct QueryRequestAuthenticator;

impl QueryRequestAuthenticator {
    fn is_allowed(&self, request: &HttpRequest) -> bool {
        #[derive(Deserialize)]
        struct QueryData {
            code: String,
        }

        let query = match Query::<QueryData>::from_query(request.query_string())
        {
            Ok(query) => query,
            Err(_) => return false,
        };

        query.code == "my_secret_password"
    }
}

fn configure_services() -> Injector {
    let module = define_module! {
        services = [QueryRequestAuthenticator::default.singleton()]
    };

    let mut builder = Injector::builder();
    builder.add_module(module);
    builder.build()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let injector = configure_services();

    HttpServer::new(move || {
        App::new().app_data(injector.clone()).service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index(
    request: HttpRequest,
    auth: Injected<Svc<QueryRequestAuthenticator>>,
) -> impl Responder {
    if auth.is_allowed(&request) {
        HttpResponse::Ok().body("You got the password right!")
    } else {
        HttpResponse::Forbidden().body("Incorrect password")
    }
}
