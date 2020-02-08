# coi-actix-web

[![Build Status](https://travis-ci.org/Nashenas88/coi-actix-web.svg?branch=master)](https://travis-ci.org/Nashenas88/coi-actix-web)
[![docs.rs](https://docs.rs/coi-actix-web/badge.svg)](https://docs.rs/coi-actix-web)
[![crates.io](https://img.shields.io/crates/v/coi-actix-web.svg)](https://crates.io/crates/coi-actix-web)

Dependency Injection in Rust

This crate provides integration support between `coi` and `actix-web`.

## Example

In your `Cargo.toml`
```toml
[dependencies]
coi = { package = "coi-actix-web", version = "0.4.0" }
```

> ### Note
> It's important to rename the package to `coi` since it re-exports proc-macros from the `coi` crate, which expects the crate to be named `coi`.

and in your code:

```rust
use coi::inject;
...

#[inject]
async get_all(#[inject] service: Arc<dyn IService>) -> Result<impl Responder, ()> {
    let name = service.get(*id).await.map_err(|e| log::error!("{}", e))?;
    Ok(HttpResponse::Ok().json(DataDto::from(name)))
}
```

See [`coi-actix-sample`] for a more involved example.

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
