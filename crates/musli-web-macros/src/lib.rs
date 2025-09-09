//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-web-macros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-web-macros)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--web--macros-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-web-macros)
//!
//! This crate provides the macros used in [Müsli web].
//!
//! Please refer to <https://docs.rs/musli> for documentation.
//!
//! [Müsli web]: <https://docs.rs/musli-web>

use proc_macro::TokenStream;

mod define;

/// Define API types.
///
/// Defining an `endpoint` causes a type to be generated which is a marker type
/// for the endpoint, which binds together the request and response types.
///
/// Defining a broadcast simply associated a broadcast with a marker type.
///
/// The marker type is used with the various types used when interacting with an
/// API endpoint or broadcast, such as:
///
/// * [`web::Request`]
/// * [`web::Listener`]
///
/// These are in turn extended with the relevant API using them:
///
/// * [`yew021`] for yew `0.21.x`.
///
/// <br>
///
/// # Macro usage
///
/// The macro defines a set of endpoints and broadcasts, each of which is
/// represented by an uninhabitable type-level marker declared through a `type`
/// declaration.
///
/// On top of the API types, this macro also generates a `debug_id` function
/// with the following signature:
///
/// ```rust
/// use musli_web::api::MessageId;
///
/// fn debug_id(id: MessageId) -> impl core::fmt::Debug {
///     # "fake"
/// }
/// ```
///
/// This method can be used to debug a message id, unknown message ids will be
/// identified with an `Unknown(<number>)` debug printing.
///
/// Each type-level marker will implement either [`api::Endpoint`] or
/// [`api::Broadcast`]. And they will have an associated constant named `ID`
/// which matches the kind that are assocaited with them.
///
/// These roughly follow the structure of:
///
/// ```text
/// pub type <name>;
///
/// impl Endpoint for <name> {
///     <definition>
/// }
///
/// impl Broadcast for <name> {
///     <definition>
/// }
/// ```
///
/// Implementing an `Endpoint` can define requests and responses. The first
/// response defined is required and is the default response that certain APIs
/// will expected the endpoint to return. Any number of requests can be
/// specified, this is allows for different "sender types" to be defined, but
/// their over the wire format has to be the same.
///
/// Types specified as request types have to implement [`musli::Encode`] and
/// types sets as response types must implemente [`musli::Decode`].
///
/// ```text
/// (#[musli(..)])?
/// pub type Hello;
///
/// impl Endpoint for Hello (where <bounds>)? {
///     impl<'de> Request for HelloRequest<'de> (where <bounds>)?;
///     type Response<'de> = HelloResponse<'de> (where <bounds>)?;
/// }
/// ```
///
/// Implementing a `Broadcast` can define events, which are messages sent from
/// the server to the client. At least one event type is required, which will be
/// used as the default. Any number of events can be specified which allows for
/// different "sender types" to be defined, but their over the wire format has
/// to be the same.
///
/// ```text
/// (#[musli(..)])?
/// pub type Tick;
///
/// impl Broadcast for Tick (where <bounds>)? {
///     impl<'de> Event TickEvent<'de> (where <bounds>)?;
///     impl Event for OwnedTickEvent (where <bounds>)?;
/// }
/// ```
///
/// <br>
///
/// # Attributes
///
/// * `#[musli(kind = "...")]` - Explicitly sets the kind of an endpoint or
///   broadcast. Without it a string variant of the name of it will be used.
///
/// ```text
/// #[musli(kind = "tock")]
/// pub type Tick;
///
/// impl Broadcast for Tick {
///     impl<'de> Event for TickEvent<'de>;
///     impl Event for OwnedTickEvent;
/// }
/// ```
///
/// <br>
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli_web::api;
///
/// #[derive(Encode, Decode)]
/// pub struct HelloRequest<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct HelloResponse<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct TickEvent<'de> {
///     pub message: &'de str,
///     pub tick: u32,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct OwnedTickEvent {
///     pub message: String,
///     pub tick: u32,
/// }
///
/// api::define! {
///     pub type Hello;
///
///     impl Endpoint for Hello {
///         impl<'de> Request for HelloRequest<'de>;
///         type Response<'de> = HelloResponse<'de>;
///     }
///
///     #[musli(id = 100)]
///     pub type Tick;
///
///     impl Broadcast for Tick {
///         impl<'de> Event for TickEvent<'de>;
///         impl Event for OwnedTickEvent;
///     }
/// }
///
/// assert_eq!(format!("{:?}", debug_id(Hello::ID)), "Hello");
/// assert_eq!(format!("{:?}", debug_id(Tick::ID)), "Tick");
/// ```
///
/// [`api::Broadcast`]: <https://docs.rs/musli-web/latest/musli_web/api/trait.Broadcast.html>
/// [`api::Endpoint`]: <https://docs.rs/musli-web/latest/musli_web/api/trait.Endpoint.html>
/// [`musli::Decode`]: <https://docs.rs/musli/latest/musli/trait.Decode.html>
/// [`musli::Encode`]: <https://docs.rs/musli/latest/musli/trait.Encode.html>
/// [`web::Listener`]: <https://docs.rs/musli-web/latest/musli_web/web/struct.Listener.html>
/// [`web::Request`]: <https://docs.rs/musli-web/latest/musli_web/web/struct.Request.html>
/// [`yew021`]: <https://docs.rs/musli-web/latest/musli_web/yew021/>
#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    let path = syn::parse_quote!(::musli_web);
    let cx = define::cx(&path);

    let stream = define::expand(&cx, input.into());

    if let Some(stream) = cx.into_compile_errors() {
        return stream.into();
    };

    stream.into()
}
