# coi-actix-web

[![Build Status](https://travis-ci.org/Nashenas88/coi-actix-web.svg?branch=master)](https://travis-ci.org/Nashenas88/coi-actix-web)
[![docs.rs](https://docs.rs/coi-actix-web/badge.svg)](https://docs.rs/coi-actix-web)
[![crates.io](https://img.shields.io/crates/v/coi-actix-web.svg)](https://crates.io/crates/coi-actix-web)

Dependency Injection in Rust

This crate provides integration support between `coi` and `actix-web`. It
exposes an `inject` procedural attribute macro to generate the code for
retrieving your dependencies from a `Container` registered with `actix-web`.

## Example

```rust
// What this crate provides
use coi_actix_web::inject;

// What's needed for the example fn below
use actix_web::{get, web::{self, HttpResponse}, Responder};
use std::sync::Arc;

// Add the `inject` attribute to the function you want to inject
#[get("/{id}")]
#[coi_actix_web::inject]
async fn get(
    id: web::Path<u64>,
    // Add the `inject` field attribute to each attribute you want
    // injected
    #[inject] service: Arc<dyn IService>
) -> Result<impl Responder, ()> {
    let data = service.get(*id).await?;
    Ok(HttpResponse::Ok().json(DataDto::from(data)))
}

// Just data models for the above fn
use serde::Serialize;

#[derive(Serialize)]
struct DataDto {
    name: String,
}

impl DataDto {
    fn from(data: Data) -> Self {
        Self {
            name: data.name
        }
    }
}


// An example of what's usually needed to make effective use of this
// crate is below
use coi::Inject;
use futures::future::{ready, BoxFuture};

// This section shows coi being put to use
// It's very important that the version of coi and the version
// of coi-actix-web used match since coi-actix-web implements
// some coi traits

// Here we're marking a trait as injectable
trait IService: Inject {
    fn get(&self, id: u64) -> BoxFuture<Result<Data, ()>>;
}

// And here we're marking a type that's capable of providing the
// above trait
#[derive(Inject)]
#[coi(provides dyn IService with ServiceImpl::new(repo))]
struct ServiceImpl {
    // Here we're injecting a dependency. `ServiceImpl` does
    // not need to know how to get this value.
    #[coi(inject)]
    repo: Arc<dyn IRepo>
}

// Normal impl for struct
impl ServiceImpl {
    fn new(repo: Arc<dyn IRepo>) -> Self {
        Self { repo }
    }
}

// Normal impl of trait for struct
impl IService for ServiceImpl {
    fn get(&self, id: u64) -> BoxFuture<Result<Data, ()>> {
        self.repo.read_from_db(id)
    }
}

// The data that will be passed between services
struct Data {
    id: u64,
    name: String,
}

// Here's the trait from above
trait IRepo: Inject {
    fn read_from_db(&self, id: u64) -> BoxFuture<Result<Data, ()>>;
}

// And it's setup below
#[derive(Inject)]
#[coi(provides dyn IRepo with RepoImpl)]
struct RepoImpl;

impl IRepo for RepoImpl {
    fn read_from_db(&self, id: u64) -> BoxFuture<Result<Data, ()>> {
        Box::pin(ready(Ok(Data {
            id,
            name: format!("{}'s name...", id)
        })))
    }
}

// This is for the register_container function
use coi_actix_web::AppExt as _;

// Your general server setup in "main". The name here is different
#[actix_rt::main]
async fn like_main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    use coi::container;

    // Construct your coi container with your keys and providers
    // See the coi crate for more details
    let container = container!{
        repo => RepoImplProvider; scoped,
        service => ServiceImplProvider; scoped
    };

    HttpServer::new(move || {
        App::new()
        // Don't forget to register the container or else
        // the injections will fail on every request!
        .register_container(container.clone())
        .service(get)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

See the repo [`coi-actix-sample`] for a more involved example.

[`coi-actix-sample`]: https://github.com/Nashenas88/coi-actix-sample

#### License

<sup>
Licensed under either of <a href="LICENSE.Apache-2.0">Apache License, Version
2.0</a> or <a href="LICENSE.MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

`SPDX-License-Identifier: MIT OR Apache-2.0`
