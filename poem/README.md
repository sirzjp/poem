<h1 align="center">Poem Framework</h1>

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/poem">
    <img src="https://img.shields.io/crates/v/poem.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/poem">
    <img src="https://img.shields.io/crates/d/poem.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/poem">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <a href="https://github.com/rust-secure-code/safety-dance/">
    <img src="https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square"
      alt="Unsafe Rust forbidden" />
  </a>
  <a href="https://blog.rust-lang.org/2021/11/01/Rust-1.56.1.html">
    <img src="https://img.shields.io/badge/rustc-1.56.1+-ab6000.svg"
      alt="rustc 1.56.1+" />
  </a>
</div>
<p align="center"><code>A program is like a poem, you cannot write a poem without writing it. --- Dijkstra</code></p>
<p align="center"> A full-featured and easy-to-use web framework with the Rust programming language.</p>

***

* [Book](https://poem-web.github.io/poem/)
* [Docs](https://docs.rs/poem)
* [Cargo package](https://crates.io/crates/poem)

## Features

- Both _Ease_ of use and performance.
- Minimizing the use of generics.
- Blazing fast and flexible routing.
- `tower::Service` and `tower::Layer` compatibility.
- Use [poem-openapi](https://crates.io/crates/poem-openapi) to write APIs that comply with [OAS3](https://github.com/OAI/OpenAPI-Specification) specifications and automatically generate documents.

## Crate features

To avoid compiling unused dependencies, Poem gates certain features, all of
which are disabled by default:

|Feature           |Description                     |
|------------------|--------------------------------|
|compression       | Support decompress request body and compress response body |
|cookie            | Support for Cookie             |
|multipart         | Support for Multipart          |
|native-tls        | Support for HTTP server over TLS with [`native-tls`](https://crates.io/crates/native-tls)  |
|opentelemetry     | Support for opentelemetry    |
|prometheus        | Support for Prometheus       |
|redis-session     | Support for RedisSession     |
|rustls            | Support for HTTP server over TLS with [`rustls`](https://crates.io/crates/rustls)  |
|session           | Support for session    |
|sse               | Support Server-Sent Events (SSE)       |
|staticfiles       | Support for serve static files       |
|tempfile          | Support for [`tempfile`](https://crates.io/crates/tempfile) |
|template          | Support for [`askama`](https://crates.io/crates/askama)       |
|tower-compat      | Adapters for `tower::Layer` and `tower::Service`. |
|websocket         | Support for WebSocket          |

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.

## Example

```rust, no_run
use poem::{get, handler, listener::TcpListener, web::Path, Route, Server};

#[handler]
fn hello(Path(name): Path<String>) -> String {
    format!("hello: {}", name)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/hello/:name", get(hello));
    Server::new(TcpListener::bind("127.0.0.1:3000"))
      .run(app)
      .await
}
```

More examples can be found [here][examples]. 

[examples]: https://github.com/poem-web/poem/tree/master/examples

## MSRV

The minimum supported Rust version for this crate is `1.56.1`.

## Contributing

:balloon: Thanks for your help improving the project! We are so happy to have you! 


## License

Licensed under either of

* Apache License, Version 2.0,([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Poem by you, shall be licensed as Apache, without any additional terms or conditions.
