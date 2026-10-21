# musli-web

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
[<img alt="crates.io" src="https://img.shields.io/crates/v/musli-web.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-web)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--web-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-web)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/musli/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/musli/actions?query=branch%3Amain)

This crate provides a set of utilities for working with various web-based
APIs and [Müsli].

It includes support for:
- [`axum`] [`Json`] integration, allowing you to use Müsli for serialization
  and deserialization in your Axum applications.
- [`axum`] [`ws::Server`] integration, allowing you to build the server side
  of the websocket protocol this crate implements.
- [`yew`] integration, allowing you to use Müsli for communicating with
  websocket clients using a well-defined API.
- [`tokio-tungstenite`] integration through [`tungstenite029`] and
  [`tungstenite030`], allowing non-browser clients to talk to the same
  websocket API.

Note that the organization of the modules include the version of the corresponding
crate. Unstable versions are prefixed with `0`, such as [`yew023`].

See the following modules for how to use:
* [`axum08`] for [`axum`] `0.8.x` integration.
* [`tungstenite029`] for [`tokio-tungstenite`] `0.29.x` integration.
* [`tungstenite030`] for [`tokio-tungstenite`] `0.30.x` integration.
* [`yew022`] for [`yew`] `0.22.x` integration.
* [`yew023`] for [`yew`] `0.23.x` integration.
* [`web03`] for [`web-sys`] `0.3.x` integration.

<br>

## Features

* A wasm-compatible and convenient websocket protocol with support for
  request-reply and broadcasts.
* [Channel support] allowing the server to identify the source of a message
  and clients to correlate messages.
* [Typed request errors]. A handler can fail a request with an error the
  client decodes back into its type, rather than a message it has to parse.
* A [negotiable body format], so a client can pick how much schema evolution
  it needs and the server adapts to it. Message envelopes never depend on
  the outcome, so negotiation itself is always readable.
* A [selectable socket mode], so a connection can be switched from binary
  frames to text ones whose envelope is a block of `<key>: <value>` lines.
  Together with [`Format::Json`] this makes the whole conversation readable
  in a proxy or a browser inspector, which is what it is for.
* An [explicitly versioned] protocol. Every message states the [`VERSION`]
  it was written against and a header which is not recognized is refused
  rather than skipped, so peers which do not speak the same protocol part
  ways before either has acted on what the other said.
* An optional [application version] in the handshake, so a page which was
  loaded before the server was upgraded learns that it has to reload instead
  of talking to an API which has moved on.

The available formats, from most compact to most capable, are
[`Format::Packed`], [`Format::Storage`], [`Format::Wire`] (the default, and
the least capable one which is fully upgrade safe), [`Format::Descriptive`]
and [`Format::Json`]. Each is gated behind a `format-*` feature.

<br>

## Choosing a format

The body of every message is encoded with one format per connection. The
envelope around it never depends on the choice, so the two sides can always
talk long enough to settle it.

* The client asks. Call `format` on the service builder (`ServiceBuilder`
  in the browser and `client::ServiceBuilder` for native clients) with a
  `Format`. A client which says nothing speaks `Format::Wire`, which tolerates
  fields being added, removed and unknown, so client and server can be
  deployed separately. `Format` implements `FromStr` and `Display`, so a
  choice can come from a configuration value or a query string, as in
  `"json".parse::<Format>()`.
* The server accepts. By default it accepts every format it was built with.
  Call `with_formats` on the server to narrow that to a list, such as
  `.with_formats(&[Format::Wire, Format::Json])`. This can never add a format
  which was not built in.
* A request for a format the server does not accept is reported to the client
  through its `on_error` callback and the connection continues with
  `Format::Wire`. The format in use can be read back with `format()` on the
  client's handle. Typed request errors are encoded in the connection's
  format too.
* A client that requests `Format::Json` can also ask for text frames with
  `mode(Mode::Text)`, which makes the whole conversation readable in a
  browser inspector. Text frames require a human readable format.
* The application version of the handshake is independent of the format. It
  is what tells an old page that the server has moved on.

A format is only available if its `format-*` feature is enabled, in the
crate that performs the encoding. `format-wire` is on wherever `api` is, so
`Format::Wire` is always available.

| Role | Features to enable | Notes |
|-|-|-|
| Shared api crate | `api` | Enables `format-wire`. Nothing else is needed to define requests and broadcasts. |
| Server | `axum08` | Enables `format-storage` and `format-wire`. Add `format-descriptive`, `format-json` or `format-packed` for each further format it should accept. |
| Browser client | `yew022`, `yew023` or `web03` | Enables `format-storage` and `format-wire`. Add the `format-*` feature of any other format it selects. |
| Native client | `tungstenite029` or `tungstenite030` | Enables `format-storage` and `format-wire`. Add the `format-*` feature of any other format it selects. |

Only enable what is used. A browser client which never calls `format` speaks
`Format::Wire` and needs no `format-*` feature of its own, and a server which
only serves such clients does not need `format-descriptive` or `format-json`.
Cargo unifies features across a build, so in a workspace a `format-*` feature
enabled by one crate is on for every crate that uses `musli-web`. Check what
each crate really selects.

The `json` feature is unrelated: it provides the `Json` extractor for `axum`
and does not enable `format-json`. The `formats` feature enables every
`format-*` feature at once.

[Channel support]: https://docs.rs/musli-web/latest/musli_web/web/struct.Handle.html#method.channel
[negotiable body format]: https://docs.rs/musli-web/latest/musli_web/api/index.html#wire-format
[selectable socket mode]: https://docs.rs/musli-web/latest/musli_web/api/index.html#the-text-envelope
[explicitly versioned]: https://docs.rs/musli-web/latest/musli_web/api/index.html#the-protocol-version
[application version]: https://docs.rs/musli-web/latest/musli_web/api/index.html#application-versions
[Typed request errors]: https://docs.rs/musli-web/latest/musli_web/ws/index.html#errors

<br>

## Examples

* [`api`] is the example crate which defines API types shared between server
  and client.
* [`server`] is the axum-based server implementation.
* [`client`] is the yew client communicating with the server.

You can run the client like this:

```sh
cd examples/client && trunk serve
```

You can run the server like this:

```sh
cd examples/server && cargo run
```

[`api`]: <https://github.com/udoprog/musli/tree/main/crates/musli-web/examples/api/>
[`axum`]: <https://docs.rs/axum>
[`client`]: <https://github.com/udoprog/musli/tree/main/crates/musli-web/examples/client/>
[`Json`]: <https://docs.rs/musli-web/latest/musli-web/Json.struct.html>
[`server`]: <https://github.com/udoprog/musli/tree/main/crates/musli-web/examples/server/>
[`tokio-tungstenite`]: <https://docs.rs/tokio-tungstenite>
[`web-sys`]: <https://docs.rs/web-sys>
[`ws::Server`]: <https://docs.rs/musli-web/latest/musli_web/ws/struct.Server.html>
[`yew`]: <https://yew.rs>
[Müsli]: <https://github.com/udoprog/musli>
