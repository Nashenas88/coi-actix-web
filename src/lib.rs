//! This crate provides a simple Dependency Injection framework for `actix-web`.
//! 
//! ## Example
//! 
//! In `Cargo.toml`:
//! ```toml
//! [dependencies]
//! coi = { package = "coi-actix-web", version = "0.4.0" }
//! actix-web = "2.0.0"
//! ```
//! 
//! Note that the following example is heavily minified. Files names don't really matter. For a
//! more involved example, please see the [`coi-actix-sample`] repository.
//! 
//! [`coi-actix-sample`]: https://github.com/Nashenas88/coi-actix-sample
//! 
//! In your main binary:
//! ```rust,ignore
//! use crate::infrastructure::{RepositoryProvider, PoolProvider};
//! use crate::service::ServiceProvider;
//! use coi::container;
//! use actix_web::{App, HttpServer};
//! 
//! mod traits;
//! mod infrastructure;
//! mod routes;
//! mod service;
//! 
//! fn main() {
//!     // container! only expects identifiers, so construct this provider outside
//!     let postgres_pool = PoolProvider::<NoTls>::new(/* construct actual pool */);
//! 
//!     // Build your container
//!     let container = container! {
//!         pool => postgres_pool.singleton,
//!         service => ServiceProvider.scoped,
//!         repository => RepositoryProvider.scoped,
//!     };
//!
//!     HttpServer::new(move || {
//!         App::new()
//!              // Make sure to assign it to `app_data` and not `data`
//!             .app_data(container.clone())
//!             .configure(routes::data::route_config)
//!     })
//!     ...
//! }
//! ```
//! 
//! `traits.rs`:
//! ```rust,ignore
//! use coi::Inject;
//! 
//! // Ensure all of your traits inherit from `Inject`
//! pub trait IService: Inject {
//!     ...
//! }
//! 
//! pub trait IRepository: Inject {
//!     ...
//! }
//! ```
//! 
//! `service.rs`
//! ```rust,ignore
//! use crate::traits::IService;
//! use coi::Inject;
//! use std::sync::Arc;
//! 
//! // derive `Inject` for all structs that will provide the injectable traits.
//! #[derive(Inject)]
//! #[provides(pub dyn IService with Service::new(repository))]
//! struct Service {
//!     #[inject]
//!     repository: Arc<dyn IRepository>,
//! }
//! 
//! impl IService for Service {
//!     ...
//! }
//! ```
//! 
//! > **Note**: See [`coi::Inject`] for more examples on how to use `#[derive(Inject)]`
//! 
//! [`coi::Inject`]: derive.Inject.html
//!
//! `infrastructure.rs`
//! ```rust,ignore
//! use crate::traits::IRepository;
//! use coi::Inject;
//! use ...::PostgresPool;
//! #[cfg(feature = "notls")]
//! use ...::NoTls;
//! #[cfg(not(feature = "notls"))]
//! use ...::Tls;
//! 
//! #[derive(Inject)]
//! #[provides(pub dyn IRepository with Repository::new(pool))]
//! struct Repository {
//!     #[cfg(feature = "notls")]
//!     #[inject]
//!     pool: PostgresPool<NoTls>,
//!
//!     #[cfg(not(feature = "notls"))]
//!     #[inject]
//!     pool: PostgresPool<Tls>,
//! }
//! 
//! impl IRepository for Repository {
//!     ...
//! }
//! 
//! #[derive(Inject)]
//! struct Pool<T> where T: ... {
//!     pool: PostgresPool<T>
//! }
//! 
//! #[derive(Provide)]
//! #[provides(pub Pool<T> with Pool::new(self.0.pool))]
//! struct PoolProvider<T> where T: ... {
//!     pool: PostgresPool<T>
//! }
//! 
//! impl<T> PoolProvider<T> where T: ... {
//!     fn new(PostgresPool<T>) -> Self { ... }
//! }
//! ```
//! 
//! `routes.rs`
//! ```rust,ignore
//! use crate::service::IService;
//! use actix_web::{
//!     web::{self, HttpResponse, ServiceConfig},
//!     Responder,
//! };
//! use coi::inject;
//! use std::sync::Arc;
//! 
//! #[inject]
//! async fn get(
//!     id: web::Path<i64>,
//!     #[inject] service: Arc<dyn IService>,
//! ) -> Result<impl Responder, ()> {
//!     let name: String = service.get(*id).await.map_err(|e| log::error!("{}", e))?;
//!     Ok(HttpResponse::Ok().json(name))
//! }
//! 
//! #[inject]
//! async fn get_all(#[inject] service: Arc<dyn IService>) -> Result<impl Responder, ()> {
//!     let data: Vec<String> = service.get_all().await.map_err(|e| log::error!("{}", e))?;
//!     Ok(HttpResponse::Ok().json(data))
//! }
//! 
//! pub fn route_config(config: &mut ServiceConfig) {
//!     config.service(
//!         web::scope("/data")
//!             .route("", web::get().to(get_all))
//!             .route("/", web::get().to(get_all))
//!             .route("/{id}", web::get().to(get)),
//!     );
//! }
//! ```

// re-export coi for convenience
pub use coi::*;
pub use coi_actix_web_derive::*;

use actix_web::{
    dev::Payload,
    error::{Error as WebError, ErrorInternalServerError, Result as WebResult},
    FromRequest, HttpRequest,
};
use futures::future::{err, ok, ready, Ready};
use std::{marker::PhantomData, sync::Arc};

#[doc(hidden)]
pub trait ContainerKey<T>
where
    T: Inject + ?Sized,
{
    const KEY: &'static str;
}

#[doc(hidden)]
pub struct Injected<T, K>(pub T, pub PhantomData<K>);

impl<T, K> Injected<T, K> {
    pub fn new(injected: T) -> Self {
        Self(injected, PhantomData)
    }
}

impl<T, K> FromRequest for Injected<Arc<T>, K>
where
    T: Inject + ?Sized,
    K: ContainerKey<T>,
{
    type Error = WebError;
    type Future = Ready<WebResult<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.app_data::<Container>() {
            Some(container) => {
                let container = container.scoped();
                ready(
                    container
                        .resolve::<T>(K::KEY)
                        .map(Injected::new)
                        .map_err(|e| {
                            log::error!("{}", e);
                            ErrorInternalServerError("huh")
                        }),
                )
            }
            None => {
                log::error!("Container not registered");
                err(ErrorInternalServerError("huh2"))
            }
        }
    }
}

macro_rules! injected_tuples {
    ($(($T:ident, $K:ident)),+) => {
        impl<$($T, $K),+> FromRequest for Injected<($(Arc<$T>),+), ($($K),+)>
        where $(
            $T: Inject + ?Sized,
            $K: ContainerKey<$T>,
        )+
        {
            type Error = WebError;
            type Future = Ready<WebResult<Self, Self::Error>>;
            type Config = ();

            fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
                match req.app_data::<Container>() {
                    Some(container) => {
                        let container = container.scoped();
                        ok(Injected::new(($(
                            {
                                let resolved = container.resolve::<$T>(<$K as ContainerKey<$T>>::KEY)
                                    .map_err(ErrorInternalServerError);
                                match resolved {
                                    Ok(r) => r,
                                    Err(e) => return err(e),
                                }
                            },
                        )+)))
                    },
                    None => err(ErrorInternalServerError("Container not registered"))
                }
            }
        }
    }
}

injected_tuples!((TA, KA), (TB, KB));
injected_tuples!((TA, KA), (TB, KB), (TC, KC));
injected_tuples!((TA, KA), (TB, KB), (TC, KC), (TD, KD));
injected_tuples!((TA, KA), (TB, KB), (TC, KC), (TD, KD), (TE, KE));
injected_tuples!((TA, KA), (TB, KB), (TC, KC), (TD, KD), (TE, KE), (TF, KF));
injected_tuples!(
    (TA, KA),
    (TB, KB),
    (TC, KC),
    (TD, KD),
    (TE, KE),
    (TF, KF),
    (TG, KG)
);
injected_tuples!(
    (TA, KA),
    (TB, KB),
    (TC, KC),
    (TD, KD),
    (TE, KE),
    (TF, KF),
    (TG, KG),
    (TH, KH)
);
injected_tuples!(
    (TA, KA),
    (TB, KB),
    (TC, KC),
    (TD, KD),
    (TE, KE),
    (TF, KF),
    (TG, KG),
    (TH, KH),
    (TI, KI)
);
injected_tuples!(
    (TA, KA),
    (TB, KB),
    (TC, KC),
    (TD, KD),
    (TE, KE),
    (TF, KF),
    (TG, KG),
    (TH, KH),
    (TI, KI),
    (TJ, KJ)
);
injected_tuples!(
    (TA, KA),
    (TB, KB),
    (TC, KC),
    (TD, KD),
    (TE, KE),
    (TF, KF),
    (TG, KG),
    (TH, KH),
    (TI, KI),
    (TJ, KJ),
    (TK, KK)
);
